use crate::Error;

use simple_asn1::{oid, ASN1Block, ASN1Class, BigUint, FromASN1};

#[derive(Clone, Debug, PartialEq)]
pub struct Certificate {
    pub tbs_certificate: TbsCertificate,
    pub signature_algorithm: AlgorithmIdentifier,
    pub signature_value: Vec<u8>,
}

impl FromASN1 for Certificate {
    type Error = Error;

    fn from_asn1(v: &[ASN1Block]) -> Result<(Self, &[ASN1Block]), Self::Error> {
        let (tbs_certificate, v) = TbsCertificate::from_asn1(v)?;
        let (signature_algorithm, v) = AlgorithmIdentifier::from_asn1(v)?;
        if let [ASN1Block::BitString(_, _, signature_value)] = v {
            Ok((
                Self {
                    tbs_certificate,
                    signature_algorithm,
                    // TODO don't clone
                    signature_value: signature_value.clone(),
                },
                &[],
            ))
        } else {
            Err(Error::Asn1)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TbsCertificate {
    pub version: u8,
    pub serial_number: BigUint,
    pub signature: AlgorithmIdentifier,
    pub validity: Validity,
    pub subject_public_key_info: SubjectPublicKeyInfo,
}

// TODO: surely there is a better way
macro_rules! to_u32 {
    ($i:ident) => {
        match $i.iter_u32_digits().next() {
            Some(i) => i,
            None => 0,
        }
    };
}

impl FromASN1 for TbsCertificate {
    type Error = Error;

    fn from_asn1(_v: &[ASN1Block]) -> Result<(Self, &[ASN1Block]), Self::Error> {
        todo!()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Version {
    V1,
    V2,
    V3,
}

impl FromASN1 for Version {
    type Error = Error;

    fn from_asn1(v: &[ASN1Block]) -> Result<(Self, &[ASN1Block]), Self::Error> {
        let mut result = Err(Error::Asn1);
        if let ASN1Block::Explicit(ASN1Class::ContextSpecific, _, tag, inner) = &v[0] {
            if to_u32!(tag) == 0 {
                if let ASN1Block::Integer(_, i) = &**inner {
                    result = match to_u32!(i) {
                        0 => Ok(Version::V1),
                        1 => Ok(Version::V2),
                        2 => Ok(Version::V3),
                        _ => Err(Error::Asn1),
                    };
                }
            }
        }
        result.map(|x| (x, &v[1..]))
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct AlgorithmIdentifier {
    algorithm: Algorithm,
}

impl FromASN1 for AlgorithmIdentifier {
    type Error = Error;

    fn from_asn1(v: &[ASN1Block]) -> Result<(Self, &[ASN1Block]), Self::Error> {
        let mut result = Err(Error::Asn1);

        if let ASN1Block::ObjectIdentifier(_, oid) = &v[0] {
            // TODO: Constify
            let ed25519 = oid!(1, 3, 101, 112);

            // TODO: Add more algorithms
            if oid == ed25519 {
                let algorithm = Algorithm::Ed25519;
                result = Ok(Self { algorithm });
            }
        }

        result.map(|x| (x, &v[2..]))
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Algorithm {
    Ed25519,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Validity;

impl FromASN1 for Validity {
    type Error = Error;

    fn from_asn1(_v: &[ASN1Block]) -> Result<(Self, &[ASN1Block]), Self::Error> {
        todo!()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SubjectPublicKeyInfo;

impl FromASN1 for SubjectPublicKeyInfo {
    type Error = Error;

    fn from_asn1(_v: &[ASN1Block]) -> Result<(Self, &[ASN1Block]), Self::Error> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // fn test() {
    //     // just octets
    //     let a = [0x4, 0x4, 0xa, 0xa, 0xa, 0xf];
    //     // tagged implicit
    //     let x = [0x87, 0x4, 0xa, 0xa, 0xa, 0xf];
    //     // ext
    //     let u = [
    //         0x30, 0xd, 0x6, 0x3, 0x55, 0x1d, 0x11, 0x4, 0x6, 0x87, 0x4, 0xa, 0xa, 0xa, 0xf,
    //     ];

    //     // actual
    //     let z = [
    //         0x30, 0xf, 0x6, 0x3, 0x55, 0x1d, 0x11, 0x4, 0x8, 0x30, 0x6, 0x87, 0x4, 0xa, 0xa, 0xa,
    //         0xf,
    //     ];
    //     let a = simple_asn1::from_der(&a);
    //     let x = simple_asn1::from_der(&x);
    //     let u = simple_asn1::from_der(&u);
    //     let z = simple_asn1::from_der(&z);
    //     eprintln!("just octets");
    //     eprintln!("{:#?}", a.unwrap());
    //     eprintln!("tagged implicit");
    //     eprintln!("{:#?}", x.unwrap());
    //     eprintln!("ext");
    //     eprintln!("{:#?}", u.unwrap());
    //     eprintln!("actual");
    //     eprintln!("{:#?}", z.unwrap());
    // }
}
