use crate::{Error, Result, MAGIC_COOKIE};

use std::{convert::TryFrom, net};

/// An enum that contains all supported attributes that can be added to a STUN message.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Attribute {
    Software(Software),
    XorMappedAddress(XorMappedAddress),
}

impl Attribute {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Attribute::Software(a) => a.to_bytes(),
            Attribute::XorMappedAddress(a) => a.to_bytes(),
        }
    }

    /// Decodes bytes into an [`Attribute`]. The function requires the transaction id of the message
    /// as some attributes require it in order to successfully interpret the bytes. If given
    /// multiple attributes, it will only decode the top most one and return the decoded attribute
    /// and how many bytes were read while decoding it.
    #[allow(clippy::missing_panics_doc)]
    pub fn from_bytes(data: Vec<u8>, tid: [u8; 12]) -> Result<(Self, usize)> {
        // Ensure future indexing won't panic.
        if data.len() < 2 {
            return Err(Error::InvalidAttributeType);
        }

        match u16::from_be_bytes(<[u8; 2]>::try_from(&data[0..2]).unwrap()) {
            Software::TYPE => {
                let result = Software::from_bytes(data, tid)?;
                Ok((Attribute::Software(result.0), result.1))
            }
            XorMappedAddress::TYPE => {
                let result = XorMappedAddress::from_bytes(data, tid)?;
                Ok((Attribute::XorMappedAddress(result.0), result.1))
            }
            _ => Err(Error::InvalidAttributeType),
        }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        match self {
            Attribute::Software(a) => a.len(),
            Attribute::XorMappedAddress(a) => a.len(),
        }
    }
}

/// The trait implemented by all STUN attributes.
#[allow(clippy::len_without_is_empty)]
pub trait AttributeExt: Sized {
    /// The 2 byte value that STUN agents use to identify the type of attribute.
    #[doc(hidden)]
    const TYPE: u16;
    /// The encoded representation of the attribute, not including the attribute header or any
    /// padding.
    #[doc(hidden)]
    fn encode(&self) -> Vec<u8>;
    /// Should return a decoded version of `Self` from the data given. `data` will contain the
    /// value of the attribute, so it will not include the header or any padding.
    #[doc(hidden)]
    fn decode(data: Vec<u8>, tid: [u8; 12]) -> Result<Self>;
    /// The length of the [`Vec`](std::vec::Vec) returned by [`encode`](AttributeExt::encode).
    ///
    /// This is only included as there are often much more efficient ways of calculating the length
    /// instead of fully encoding it and getting the length (e.g. the length of a
    /// [`XorMappedAddress`] is constant).
    #[doc(hidden)]
    fn value_len(&self) -> usize;

    /// The total length of the attribute once encoded.
    ///
    /// The length includes 4 bytes for the header, the length of the internal value, and the length
    /// of padding required.
    #[doc(hidden)]
    fn len(&self) -> usize {
        4 + self.value_len() + self.padding_len()
    }

    /// The amount of padding needed when encoding this attribute.
    ///
    /// Since STUN aligns attributes on 32-bit boundaries, attributes whose content is not a
    /// multiple of 4 bytes are padded with 1, 2, or 3 bytes of padding so that its value contains
    /// a multiple of 4 bytes. The padding bits are ignored, and may be any value.
    #[doc(hidden)]
    fn padding_len(&self) -> usize {
        (4 - self.value_len() % 4) % 4
    }

    fn to_bytes(&self) -> Vec<u8> {
        // The size of the header is 4 bytes.
        let mut result = Vec::with_capacity(self.len());

        result.extend_from_slice(&Self::TYPE.to_be_bytes());
        result.extend_from_slice(&(self.value_len() as u16).to_be_bytes());
        result.extend(self.encode());
        result.extend(vec![0; self.padding_len()]);

        // TODO assert len of result?

        result
    }

    /// Decodes bytes into an [`Attribute`]. The function requires the transaction id of the message
    /// as some attributes require it in order to successfully interpret the bytes. If given
    /// multiple attributes, it will only decode the top most one and return the decoded attribute
    /// and how many bytes were read while decoding it.
    fn from_bytes(data: Vec<u8>, tid: [u8; 12]) -> Result<(Self, usize)> {
        // The two try_from unwraps are safe as we are taking a slice of length 2 from a Vec<u8>.

        let expected_type = u16::from_be_bytes(<[u8; 2]>::try_from(&data[0..2]).unwrap());
        if expected_type != Self::TYPE {
            return Err(Error::IncorrectAttributeType);
        }

        // Ensure that indexing further down won't panic.
        let expected_len = u16::from_be_bytes(<[u8; 2]>::try_from(&data[2..4]).unwrap());
        if expected_len as usize > data.len() - 4 {
            return Err(Error::IncorrectAttributeLength);
        }

        // TODO we don't need to clone the vec, but I don't know how to get a slice of a vec that is
        // itself a vec.
        // Give decode the data contained in the packet (i.e. the packet - the heading - the
        // padding).
        Ok((
            Self::decode(data[4..(4 + expected_len) as usize].to_vec(), tid)?,
            // The length of data used is equal to the header (4 bytes) + the length of data
            // + the padding
            (4 + expected_len + (4 - expected_len % 4) % 4) as usize,
        ))
    }
}

/// A textual description of the software being used by the agent sending the message.
///
/// Its value must be a valid UTF-8 string and have fewer than 128 characters.
/// The value should include manufacturer and version number. The attribute has no impact on
/// operation of the protocol, and serves only as a tool for diagnostic and debugging purposes.
///
/// Software is a comprehension-optional attribute, which means that it can be ignored by
/// the STUN agent if it does not understand it.
///
/// # Reference
/// [RFC 5389]
///
/// [RFC 5389]: https://datatracker.ietf.org/doc/html/rfc5389#section-15.10
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Software(String);

impl Software {
    pub fn value(&self) -> String {
        self.0.clone()
    }
}

impl std::convert::TryFrom<&str> for Software {
    type Error = Error;

    /// Attempts to create a new [`Software`] attribute. As required by [RFC 5389], the value for the
    /// attribute must be a valid UTF-8 string and have fewer than 128 characters.
    ///
    /// [RFC 5389]: https://datatracker.ietf.org/doc/html/rfc5389#section-15.10
    fn try_from(value: &str) -> Result<Self> {
        let utf8_len = value.as_bytes().len();
        if utf8_len >= 128 {
            Err(Error::AttributeTooLarge("Software"))
        } else {
            Ok(Software(value.to_owned()))
        }
    }
}

impl AttributeExt for Software {
    const TYPE: u16 = 0x8022;

    fn encode(&self) -> Vec<u8> {
        self.0.as_bytes().to_owned()
    }

    fn decode(data: Vec<u8>, _: [u8; 12]) -> Result<Self> {
        let string = String::from_utf8(data)?;
        Ok(Self(string))
    }

    fn value_len(&self) -> usize {
        self.0.as_bytes().len()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct XorMappedAddress {
    ip: net::IpAddr,
    /// If the address is ipv6 then the transaction id of the message is used in the xor function.
    #[doc(hidden)]
    tid: Option<[u8; 12]>,
    port: u16,
}

impl XorMappedAddress {
    pub fn from(ip: net::IpAddr, port: u16) -> Self {
        Self {
            ip,
            tid: None,
            port,
        }
    }

    pub fn ip(&self) -> net::IpAddr {
        self.ip
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn is_ipv4(&self) -> bool {
        self.ip.is_ipv4()
    }

    pub fn is_ipv6(&self) -> bool {
        self.ip.is_ipv6()
    }

    /// This function must only be called by the [`Message`](crate::stun::Message) that carries
    /// the [`Software`] attribute. It should not be called by the user.
    #[doc(hidden)]
    pub(crate) fn set_tid(&mut self, tid: [u8; 12]) {
        self.tid = Some(tid);
    }
}

impl std::convert::From<XorMappedAddress> for std::net::SocketAddr {
    fn from(a: XorMappedAddress) -> Self {
        std::net::SocketAddr::new(a.ip(), a.port())
    }
}

impl AttributeExt for XorMappedAddress {
    const TYPE: u16 = 0x0020;

    fn encode(&self) -> Vec<u8> {
        // The port is XORed with the 16 most significant bits of the `MAGIC_COOKIE`. The typecast
        // is safe as the bit shift guarantees that, at most, only the 16 right most bits of the u32
        // are set.
        let port_xor = (self.port ^ (MAGIC_COOKIE >> 16) as u16).to_be_bytes();

        match self.ip {
            net::IpAddr::V4(a) => {
                let mut result = Vec::new();
                // Special code denoting the address family (i.e. IPv4).
                result.extend_from_slice(&1u16.to_be_bytes());
                result.extend_from_slice(&port_xor);

                let addr = u32::from(a);
                result.extend_from_slice(&(addr ^ MAGIC_COOKIE).to_be_bytes());

                result
            }
            net::IpAddr::V6(a) => {
                let mut result = Vec::new();
                // Special code denoting the address family (i.e. IPv6).
                result.extend_from_slice(&2u16.to_be_bytes());
                result.extend_from_slice(&port_xor);

                let addr = u128::from(a);

                let mut xor = Vec::new();
                xor.extend_from_slice(&MAGIC_COOKIE.to_be_bytes());
                // Unwrap is ok as `Message` sets the value of `tid` when it pushes an
                // `XorMappedAddress`.
                xor.extend_from_slice(&self.tid.unwrap());
                // Unwrap is ok as the vector is guaranteed to be 16 bytes: `MAGIC_COOKIE` must be
                // of length 4 and `self.tid` must be of length 12.
                let xor = u128::from_be_bytes(<[u8; 16]>::try_from(xor).unwrap());

                result.extend_from_slice(&(addr ^ xor).to_be_bytes());

                result
            }
        }
    }

    fn decode(data: Vec<u8>, tid: [u8; 12]) -> Result<Self> {
        // Make sure that getting the family (bytes 1-2 of the message) and decoding the port
        // (bytes 3-4 of the message) don't panic.
        if data.len() < 4 {
            return Err(Error::IncorrectAttributeLength);
        }

        let encoded_port = u16::from_be_bytes(<[u8; 2]>::try_from(&data[2..4]).unwrap());
        let decoded_port = encoded_port ^ (MAGIC_COOKIE >> 16) as u16;

        // It's a slice of 2 so unwrap can't fail.
        match u16::from_be_bytes(<[u8; 2]>::try_from(&data[0..2]).unwrap()) {
            // IPv4
            1 => {
                // If the address family is IPv4, the address must be 32 bits.
                // 2 bytes (length) + 2 bytes (port) + 4 bytes (address) = 8 bytes
                if data.len() != 8 {
                    return Err(Error::InvalidAddress);
                }

                let encoded_address = u32::from_be_bytes(<[u8; 4]>::try_from(&data[4..8]).unwrap());
                let decoded_address = encoded_address ^ MAGIC_COOKIE;
                let decoded_address = net::IpAddr::V4(net::Ipv4Addr::from(decoded_address));

                Ok(Self::from(decoded_address, decoded_port))
            }
            // IPv6
            2 => {
                // If the address family is IPv6, the address must be 128 bits.
                // 2 bytes (length) + 2 bytes (port) + 16 bytes (address) = 20 bytes
                if data.len() != 20 {
                    return Err(Error::IncorrectAttributeLength);
                }

                let encoded_address =
                    u128::from_be_bytes(<[u8; 16]>::try_from(&data[4..20]).unwrap());

                let mut xor = Vec::new();
                xor.extend_from_slice(&MAGIC_COOKIE.to_be_bytes());
                xor.extend_from_slice(&tid);
                // Unwrap is ok as the vector is guaranteed to be 16 bytes: `MAGIC_COOKIE` must be
                // of length 4 and `self.tid` must be of length 12.
                let xor = u128::from_be_bytes(<[u8; 16]>::try_from(xor).unwrap());

                let decoded_address = encoded_address ^ xor;
                let decoded_address = net::IpAddr::V6(net::Ipv6Addr::from(decoded_address));

                let mut result = Self::from(decoded_address, decoded_port);
                result.set_tid(tid);
                Ok(result)
            }
            _ => Err(Error::InvalidAddressFamily),
        }
    }

    fn value_len(&self) -> usize {
        match self.ip {
            // 2 (family) + 2 (port) + 4 (address)
            net::IpAddr::V4(_) => 8,
            // 2 (family) + 2 (port) + 16 (address)
            net::IpAddr::V6(_) => 20,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::net::IpAddr;

    #[test]
    fn test_software_encode() {
        let name = "unicorn company";

        let software = Software::try_from(name).unwrap();
        let bytes = software.to_bytes();

        let ty = 0x8022u16;
        let unpadded_size = 0xfu16;

        let mut expected = Vec::new();
        expected.extend_from_slice(&ty.to_be_bytes());
        expected.extend_from_slice(&unpadded_size.to_be_bytes());
        expected.extend_from_slice(name.as_bytes()); // message contents
        expected.push(0); // padding

        // Making sure that putting it in an enum doesn't break the `to_bytes` function.
        let wrapper = Attribute::Software(software.clone());
        assert_eq!(bytes, wrapper.to_bytes());

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

        if let Err(Error::AttributeTooLarge("Software")) = software {
            // correct
        } else {
            panic!();
        }
    }

    #[test]
    fn test_software_decode() {
        let name = "lakad matatag";

        let ty = 0x8022u16;
        let unpadded_size = name.len() as u16;

        let mut packet = Vec::new();
        packet.extend_from_slice(&ty.to_be_bytes());
        packet.extend_from_slice(&unpadded_size.to_be_bytes());
        packet.extend_from_slice(name.as_bytes()); // message contents
        packet.push(0); // padding

        let expected = Software::try_from(name).unwrap();
        let result = Software::from_bytes(packet, [0; 12]).unwrap();

        assert_eq!(expected, result.0);
        assert_eq!(expected.len(), result.1);
    }

    #[test]
    fn test_software_len() {
        let software = Software::try_from("company name").unwrap();
        let wrapper = Attribute::Software(software.clone());
        assert_eq!(16, software.len());
        assert_eq!(16, wrapper.len());
        assert_eq!(0, software.padding_len());
        assert_eq!(12, software.value_len());

        let software = Software::try_from("ding ding ding mf").unwrap();
        let wrapper = Attribute::Software(software.clone());
        assert_eq!(24, software.len());
        assert_eq!(24, wrapper.len());
        assert_eq!(3, software.padding_len());
        assert_eq!(17, software.value_len());

        let software = Software::try_from("cliffteezy").unwrap();
        let wrapper = Attribute::Software(software.clone());
        assert_eq!(16, software.len());
        assert_eq!(16, wrapper.len());
        assert_eq!(2, software.padding_len());
        assert_eq!(10, software.value_len());

        let software = Software::try_from("python3").unwrap();
        let wrapper = Attribute::Software(software.clone());
        assert_eq!(12, software.len());
        assert_eq!(12, wrapper.len());
        assert_eq!(1, software.padding_len());
        assert_eq!(7, software.value_len());
    }

    #[test]
    fn test_ipv4_encode() {
        let ip_adress = net::Ipv4Addr::new(31, 41, 59, 26);
        let port = 28015;

        let address = XorMappedAddress::from(IpAddr::V4(ip_adress), port);
        let wrapper = Attribute::XorMappedAddress(address);

        let ty = 0x20u16;
        let unpadded_size = 8u16;
        let family = 1u16;

        let port_xor = port ^ (MAGIC_COOKIE >> 16) as u16;

        let address_xor = u32::from(ip_adress) ^ MAGIC_COOKIE;

        let mut expected = Vec::new();
        expected.extend_from_slice(&ty.to_be_bytes());
        expected.extend_from_slice(&unpadded_size.to_be_bytes());
        expected.extend_from_slice(&family.to_be_bytes());
        expected.extend_from_slice(&port_xor.to_be_bytes());
        expected.extend_from_slice(&address_xor.to_be_bytes());

        assert!(address.is_ipv4());
        assert_eq!(address.len(), 12);
        assert_eq!(wrapper.len(), 12);
        assert_eq!(address.to_bytes(), expected);
        assert_eq!(wrapper.to_bytes(), expected);
    }

    #[test]
    fn test_ipv6_encode() {
        let ip_address = net::Ipv6Addr::new(958, 1919, 4304, 14091, 21196, 32600, 34313, 44479);
        let port = 9150;
        let tid: [u8; 12] = [5; 12];

        let mut address = XorMappedAddress::from(IpAddr::V6(ip_address), port);
        address.set_tid(tid);
        let wrapper = Attribute::XorMappedAddress(address);

        let ty = 0x20u16;
        let unpadded_size = 20u16;
        let family = 2u16;

        let port_xor = port ^ (MAGIC_COOKIE >> 16) as u16;

        let mut xor_op = Vec::new();
        xor_op.extend_from_slice(&MAGIC_COOKIE.to_be_bytes());
        xor_op.extend_from_slice(&tid);
        let address_xor =
            u128::from(ip_address) ^ u128::from_be_bytes(<[u8; 16]>::try_from(xor_op).unwrap());

        let mut expected = Vec::new();
        expected.extend_from_slice(&ty.to_be_bytes());
        expected.extend_from_slice(&unpadded_size.to_be_bytes());
        expected.extend_from_slice(&family.to_be_bytes());
        expected.extend_from_slice(&port_xor.to_be_bytes());
        expected.extend_from_slice(&address_xor.to_be_bytes());

        assert!(address.is_ipv6());
        assert_eq!(address.len(), 24);
        assert_eq!(wrapper.len(), 24);
        assert_eq!(address.to_bytes(), expected);
        assert_eq!(wrapper.to_bytes(), expected);
    }

    #[test]
    #[should_panic]
    fn test_ipv6_no_tid() {
        let ip_address = net::Ipv6Addr::new(1, 2, 3, 4, 5, 6, 7, 8);
        let port = 9418;

        let address = XorMappedAddress::from(IpAddr::V6(ip_address), port);

        address.to_bytes();
    }

    #[test]
    fn test_ipv4_decode() {
        let ip_address = net::Ipv4Addr::new(0xab, 0xad, 0xba, 0xbe);
        let port = 25565;

        let ty = 0x20u16;
        let unpadded_size = 8u16;
        let family = 1u16;

        let port_xor = port ^ (MAGIC_COOKIE >> 16) as u16;
        let address_xor = u32::from(ip_address) ^ MAGIC_COOKIE;

        let mut packet = Vec::new();
        packet.extend_from_slice(&ty.to_be_bytes());
        packet.extend_from_slice(&unpadded_size.to_be_bytes());
        packet.extend_from_slice(&family.to_be_bytes());
        packet.extend_from_slice(&port_xor.to_be_bytes());
        packet.extend_from_slice(&address_xor.to_be_bytes());

        let expected = XorMappedAddress::from(IpAddr::V4(ip_address), port);
        // tid doesn't matter for ipv4 addresses
        assert_eq!(
            XorMappedAddress::from_bytes(packet.clone(), [0; 12]).unwrap(),
            (expected, 12)
        );
        assert_eq!(
            Attribute::from_bytes(packet, [0; 12]).unwrap(),
            (Attribute::XorMappedAddress(expected), 12),
        )
    }

    #[test]
    fn test_ipv6_decode() {
        let ip_address = net::Ipv6Addr::new(322, 17, 3, 0xe400, 250, 5050, 0xbe7a, 10000);
        let port = 27015;

        let ty = 0x20u16;
        let unpadded_size = 20u16;
        let family = 2u16;

        let port_xor = port ^ (MAGIC_COOKIE >> 16) as u16;

        let tid: [u8; 12] = [5; 12];
        let mut xor_op = Vec::new();
        xor_op.extend_from_slice(&MAGIC_COOKIE.to_be_bytes());
        xor_op.extend_from_slice(&tid);
        let address_xor =
            u128::from(ip_address) ^ u128::from_be_bytes(<[u8; 16]>::try_from(xor_op).unwrap());

        let mut packet = Vec::new();
        packet.extend_from_slice(&ty.to_be_bytes());
        packet.extend_from_slice(&unpadded_size.to_be_bytes());
        packet.extend_from_slice(&family.to_be_bytes());
        packet.extend_from_slice(&port_xor.to_be_bytes());
        packet.extend_from_slice(&address_xor.to_be_bytes());

        let mut expected = XorMappedAddress::from(IpAddr::V6(ip_address), port);
        expected.set_tid(tid);

        assert_eq!(
            XorMappedAddress::from_bytes(packet.clone(), tid).unwrap(),
            (expected, 24)
        );
        assert_eq!(
            Attribute::from_bytes(packet, tid).unwrap(),
            (Attribute::XorMappedAddress(expected), 24),
        );
    }
}
