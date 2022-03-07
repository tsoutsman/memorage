use crate::Error;

use memorage_core::PublicKey;
use rustls::{
    client::{ServerCertVerified, ServerCertVerifier, ServerName},
    server::{ClientCertVerified, ClientCertVerifier},
    Certificate,
};
use x509_parser::{
    certificate::X509Certificate,
    traits::FromDer,
    validate::{Validator, X509StructureValidator},
};

pub(crate) struct CertVerifier(Option<PublicKey>);

impl CertVerifier {
    pub(crate) fn new(permitted_key: Option<PublicKey>) -> Self {
        Self(permitted_key)
    }

    fn verify_cert(
        &self,
        end_entity: &Certificate,
        intermediates: &[Certificate],
    ) -> crate::Result<()> {
        if !intermediates.is_empty() {
            return Err(Error::IntermediatesNotEmpty);
        }

        let (rem, cert) = X509Certificate::from_der(end_entity.as_ref())?;

        if !(rem.is_empty()
            || X509StructureValidator.validate(&cert, &mut DummyLogger)
            || cert.validity().is_valid())
        {
            return Err(Error::InvalidCertificate);
        }

        // TODO: Check if cert uses ed25519?

        cert.verify_signature(Some(cert.public_key()))?;

        if let Some(permitted_key) = self.0 {
            match PublicKey::try_from(cert.public_key().subject_public_key.data) {
                Ok(client_key) => {
                    if client_key != permitted_key {
                        return Err(Error::KeyNotPermitted);
                    }
                }
                Err(_) => return Err(Error::KeyNotPermitted),
            }
        }

        Ok(())
    }
}

struct DummyLogger;

impl x509_parser::validate::Logger for DummyLogger {
    fn warn(&mut self, _: &str) {}

    fn err(&mut self, _: &str) {}
}

impl ServerCertVerifier for CertVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &Certificate,
        intermediates: &[Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        match self.verify_cert(end_entity, intermediates) {
            Ok(_) => Ok(ServerCertVerified::assertion()),
            Err(_) => Err(rustls::Error::InvalidCertificateSignature),
        }
    }
}

impl ClientCertVerifier for CertVerifier {
    fn client_auth_root_subjects(&self) -> Option<rustls::DistinguishedNames> {
        // Returning None aborts the connection, whereas Some(Vec::new()) just
        // gives the client an empty list.
        Some(Vec::new())
    }

    fn verify_client_cert(
        &self,
        end_entity: &Certificate,
        intermediates: &[Certificate],
        _now: std::time::SystemTime,
    ) -> Result<ClientCertVerified, rustls::Error> {
        match self.verify_cert(end_entity, intermediates) {
            Ok(_) => Ok(ClientCertVerified::assertion()),
            Err(_) => Err(rustls::Error::InvalidCertificateSignature),
        }
    }
}

/// Get the subject key of the certificate used to authenticate the provided connection.
///
/// # Safety
/// This function does not check the validity or authenticity of the certificate -
/// it is assumed that the certificate has already been verified. If the configuration
/// from [`gen_recv_config`](crate::gen_recv_config) is used, then this function
/// call safely be called on the incoming connection (assuming the correct parameters
/// were passed to `gen_recv_config`).
#[inline]
pub fn get_key_unchecked(connection: &quinn::Connection) -> crate::Result<PublicKey> {
    let certs = connection
        .peer_identity()
        .ok_or(Error::CertificateData)?
        .downcast::<Vec<rustls::Certificate>>()
        .map_err(|_| Error::CertificateData)?;
    if certs.len() == 1 {
        Ok(PublicKey::try_from(
            X509Certificate::from_der(certs[0].as_ref())?
                .1
                .public_key()
                .subject_public_key
                .data,
        )
        .map_err(|_| Error::InvalidCertificate)?)
    } else {
        Err(Error::CertificateData)
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use super::*;
    use memorage_core::KeyPair;

    #[test]
    fn cert_verifier_valid_wanted() {
        let ip_addr = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
        let cert_verifier = CertVerifier::new(None);

        let (cert, _) = crate::config::gen_cert(ip_addr, &KeyPair::from_entropy()).unwrap();
        cert_verifier
            .verify_client_cert(&cert, &[], std::time::SystemTime::now())
            .unwrap();
    }

    #[test]
    fn cert_verifier_valid_unwanted() {
        let ip_addr = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
        let cert_verifier = CertVerifier::new(Some(KeyPair::from_entropy().public));

        let (cert, _) = crate::config::gen_cert(ip_addr, &KeyPair::from_entropy()).unwrap();
        assert!(matches!(
            cert_verifier.verify_client_cert(&cert, &[], std::time::SystemTime::now()),
            Err(rustls::Error::InvalidCertificateSignature)
        ));
    }

    #[test]
    fn cert_verifier_invalid_unwanted() {
        let cert_verifier = CertVerifier::new(None);
        #[rustfmt::skip]
        let cert = vec![
            0x30, 0x82, 0x01, 0x0d, 0x30, 0x81, 0xc0, 0xa0, 0x03, 0x02, 0x01, 0x02, 0x02, 0x09,
            0x00, 0xa3, 0x57, 0x8b, 0x72, 0xea, 0x32, 0xbe, 0xcd, 0x30, 0x05, 0x06, 0x03, 0x2b,
            0x65, 0x70, 0x30, 0x21, 0x31, 0x1f, 0x30, 0x1d, 0x06, 0x03, 0x55, 0x04, 0x03, 0x0c,
            0x16, 0x72, 0x63, 0x67, 0x65, 0x6e, 0x20, 0x73, 0x65, 0x6c, 0x66, 0x20, 0x73, 0x69,
            0x67, 0x6e, 0x65, 0x64, 0x20, 0x63, 0x65, 0x72, 0x74, 0x30, 0x20, 0x17, 0x0d, 0x37,
            0x35, 0x30, 0x31, 0x30, 0x31, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x5a, 0x18, 0x0f,
            0x34, 0x30, 0x39, 0x36, 0x30, 0x31, 0x30, 0x31, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
            0x5a, 0x30, 0x21, 0x31, 0x1f, 0x30, 0x1d, 0x06, 0x03, 0x55, 0x04, 0x03, 0x0c, 0x16,
            0x72, 0x63, 0x67, 0x65, 0x6e, 0x20, 0x73, 0x65, 0x6c, 0x66, 0x20, 0x73, 0x69, 0x67,
            0x6e, 0x65, 0x64, 0x20, 0x63, 0x65, 0x72, 0x74, 0x30, 0x2a, 0x30, 0x05, 0x06, 0x03,
            0x2b, 0x65, 0x70, 0x03, 0x21,

            // Real key
            // 0x00, 0x70, 0x2c, 0xb7, 0x93, 0xd5, 0x0b, 0x8a, 0x23, 0x4b, 0x65, 0x10, 0xb0, 0x53,
            // 0x33, 0x8d, 0x8a, 0xb3, 0xd5, 0xcf, 0x5a, 0x63, 0x1a, 0x3a, 0x33, 0x07, 0xce, 0xbc,
            // 0xf4, 0x45, 0x5f, 0x14, 0xeb,

            // Fake key
            0x00, 0xa8, 0x01, 0xc0, 0xb0, 0x1a, 0xb2, 0x61, 0x89, 0xf9, 0x0d, 0x9c, 0x0c, 0x42,
            0x4d, 0x28, 0xcd, 0x00, 0x13, 0xeb, 0x78, 0x68, 0x86, 0xb0, 0x1d, 0x06, 0x84, 0x05,
            0xea, 0xc4, 0x43, 0xd8, 0xea,
            // End fake key

            0xa3, 0x13, 0x30, 0x11, 0x30, 0x0f, 0x06, 0x03, 0x55, 0x1d, 0x11, 0x04, 0x08, 0x30,
            0x06, 0x87, 0x04, 0x01, 0x01, 0x01, 0x01, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70,
            0x03, 0x41, 0x00, 0xeb, 0xa1, 0x09, 0x72, 0xfd, 0xf9, 0xbf, 0x9e, 0x8f, 0x01, 0x2d,
            0x83, 0x40, 0xbc, 0x01, 0x92, 0x60, 0x77, 0x54, 0xd1, 0x0f, 0x6b, 0x6a, 0xbd, 0xda,
            0xac, 0xb6, 0x91, 0x83, 0xd4, 0x4e, 0x4a, 0x76, 0xf4, 0xd2, 0xbd, 0x6e, 0x01, 0x96,
            0xb3, 0xa4, 0x93, 0x87, 0xe3, 0x97, 0xfb, 0x83, 0x33, 0xfb, 0x8a, 0x7e, 0x08, 0xef,
            0xcd, 0x8f, 0x91, 0x98, 0x6f, 0x3f, 0x72, 0x96, 0x89, 0x87, 0x09,
        ];

        assert!(
            X509StructureValidator.validate(
                &X509Certificate::from_der(&cert)
                    .expect("cert failed to parse")
                    .1,
                &mut DummyLogger
            ),
            "cert is not valid (signature not yet checked)"
        );

        assert!(matches!(
            cert_verifier.verify_client_cert(
                &rustls::Certificate(cert),
                &[],
                std::time::SystemTime::now()
            ),
            Err(rustls::Error::InvalidCertificateSignature)
        ))
    }
}
