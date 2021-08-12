use crate::error::{Error, Result};

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Attribute {
    Software(Software),
}

impl Attribute {
    pub fn to_bytes(&self) -> Vec<u8> {
        match &self {
            Attribute::Software(s) => s.to_bytes(),
        }
    }

    pub fn len(&self) -> usize {
        match &self {
            // TODO do we really need to get the whole value to get len
            Attribute::Software(s) => s.value().len(),
        }
    }
}

pub trait AttributeExt {
    const TYPE: u16;
    fn value(&self) -> Vec<u8>;

    /// The total length of the attribute once encoded.
    ///
    /// The length includes 4 bytes for the header, the length of the internal value, and the length
    /// of padding required.
    fn len(&self) -> usize {
        4 + self.len_value() + self.len_padding()
    }

    /// The amount of padding needed when encoding this attribute.
    ///
    /// Since STUN aligns attributes on 32-bit boundaries, attributes whose content is not a
    /// multiple of 4 bytes are padded with 1, 2, or 3 bytes of padding so that its value contains
    /// a multiple of 4 bytes. The padding bits are ignored, and may be any value.
    fn len_padding(&self) -> usize {
        (4 - self.len_value() % 4) % 4
    }

    /// The length of the data stored in the attribute.
    fn len_value(&self) -> usize {
        self.value().len()
    }

    fn to_bytes(&self) -> Vec<u8> {
        // The size of the header is 4 bytes.
        let mut result = Vec::with_capacity(self.len());

        result.extend_from_slice(&Self::TYPE.to_be_bytes());
        result.extend_from_slice(&(self.len_value() as u16).to_be_bytes());
        result.extend(self.value());
        result.extend(vec![0; self.len_padding()]);

        // TODO assert len of result?

        result
    }
}

/// A textual description of the software being used by the agent sending the message.
///
/// Its value must be a valid UTF-8 string and have fewer than 128 characters.
/// The value should include manufacturer and version number. The attribute has no impact on
/// operation of the protocol, and serves only as a tool for diagnostic and debugging purposes.
///
/// # Reference
/// [RFC 5389]
///
/// [RFC 5389]: https://datatracker.ietf.org/doc/html/rfc5389#section-15.10
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Software(String);

impl std::convert::TryFrom<&str> for Software {
    type Error = Error;

    /// Attempts to create a new [`Software`] attribute. As required by [RFC 5389], the value for the
    /// attribute must be a valid UTF-8 string and have fewer than 128 characters.
    ///
    /// [RFC 5389]: https://datatracker.ietf.org/doc/html/rfc5389#section-15.10
    fn try_from(value: &str) -> Result<Self> {
        let utf8_len = value.as_bytes().len();
        if utf8_len >= 128 {
            Err(Error::AttrTooLarge("Software"))
        } else {
            Ok(Software(value.to_owned()))
        }
    }
}

impl AttributeExt for Software {
    const TYPE: u16 = 0x8022;

    fn value(&self) -> Vec<u8> {
        self.0.as_bytes().to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn test_software_encode() {
        let name = "unicorn company";
        let software = Software::try_from(name).unwrap();

        let bytes = software.to_bytes();

        // Making sure that putting it in an enum doesn't break the `to_bytes` function.
        let wrapper = Attribute::Software(software.clone());
        assert_eq!(bytes, wrapper.to_bytes());

        let mut expected = vec![
            0x80, // type
            0x22, // type
            0x00, // unpadded length
            0x0F, // unpadded length
        ];
        expected.extend_from_slice(name.as_bytes()); // message contents
        expected.extend_from_slice(&[0]); // padding

        assert_eq!(bytes.len(), software.len());
        assert_eq!(bytes, expected);
    }

    #[test]
    fn test_software_overflow() {
        let mut long_string = String::new();
        for _ in 0..128 {
            long_string.push(' ');
        }
        let software = Software::try_from(&long_string[..]);

        if let Err(Error::AttrTooLarge("Software")) = software {
            // correct
        } else {
            panic!();
        }
    }

    #[test]
    fn test_software_len() {
        let software = Software::try_from("company name").unwrap();
        assert_eq!(16, software.len());
        assert_eq!(0, software.len_padding());
        assert_eq!(12, software.len_value());

        let software = Software::try_from("a").unwrap();
        assert_eq!(8, software.len());
        assert_eq!(3, software.len_padding());
        assert_eq!(1, software.len_value());

        let software = Software::try_from("my company").unwrap();
        assert_eq!(16, software.len());
        assert_eq!(2, software.len_padding());
        assert_eq!(10, software.len_value());

        let software = Software::try_from("abc").unwrap();
        assert_eq!(8, software.len());
        assert_eq!(1, software.len_padding());
        assert_eq!(3, software.len_value());
    }
}
