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
        self.len_value() % 4
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
