use crate::Error;

use rustls::{
    client::{ServerCertVerified, ServerCertVerifier, ServerName},
    server::{ClientCertVerified, ClientCertVerifier},
    Certificate,
};
use soter_core::PublicKey;
use x509_parser::{certificate::X509Certificate, traits::FromDer, validate::Validate};

pub struct CertVerifier;

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
        // TODO
        if !intermediates.is_empty() {
            return Err(rustls::Error::InvalidCertificateSignature);
        }
        match verify_cert(end_entity.as_ref()) {
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
        if !intermediates.is_empty() {
            return Err(rustls::Error::InvalidCertificateSignature);
        }
        match verify_cert(end_entity.as_ref()) {
            Ok(_) => Ok(ClientCertVerified::assertion()),
            Err(_) => Err(rustls::Error::InvalidCertificateSignature),
        }
    }
}

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

#[inline]
fn verify_cert(cert: &[u8]) -> crate::Result<()> {
    let (rem, cert) = X509Certificate::from_der(cert)?;

    if !(rem.is_empty() || cert.validate_to_vec().0 || cert.validity().is_valid()) {
        return Err(Error::InvalidCertificate);
    }

    cert.verify_signature(Some(cert.public_key()))?;
    Ok(())
}
