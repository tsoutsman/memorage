use crate::{
    error::{Error, Result},
    stun::attribute::Attribute,
};

use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaCha20Rng,
};

/// The magic cookie field must contain the fixed value `0x2112A442` in network byte order.
pub const MAGIC_COOKIE: u32 = 0x2112A442;

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, enumn::N)]
pub enum Class {
    Request,
    Indication,
    Success,
    Error,
}

/// Enum containing the possible methods defined in [RFC 8489] and [RFC 8656].
///
/// # Implementation details
/// The struct is represented by a [`u16`] as theoretically, the method is defined by 12 bits in the
/// message, meaning a [`u8`] is too small. In reality, the method integer representation is never
/// greater than 9, however, we still chose to represent it as a [`u16`] for compatibility.
#[repr(u16)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, enumn::N)]
pub enum Method {
    Binding = 1,
    Allocate = 3,
    Refresh,
    Send = 6,
    Data,
    CreatePermission,
    ChannelBind,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Type {
    pub class: Class,
    pub method: Method,
}

impl std::convert::From<Type> for u16 {
    fn from(t: Type) -> Self {
        let mut result: u16 = t.method as u16;

        // +--+--+-+-+-+-+-+-+-+-+-+-+-+-+
        // |M |M |M|M|M|C|M|M|M|C|M|M|M|M|
        // |11|10|9|8|7|1|6|5|4|0|3|2|1|0|
        // +--+--+-+-+-+-+-+-+-+-+-+-+-+-+
        // The c bits denote the class of the message. So, if the first bit of class is true, we
        // set the 5th bit (i.e. add 16). Likewise, if the second bit of class is true, we set the
        // 9th bit (i.e. add 256).

        // if the 1st bit is set
        if (t.class as u8 & 1) != 0 {
            result += 1 << 4;
        }
        // if the second bit is set
        if (t.class as u8 >> 1 & 1) != 0 {
            result += 1 << 8;
        }

        result
    }
}

impl std::convert::From<Type> for [u8; 2] {
    fn from(t: Type) -> Self {
        u16::from(t).to_be_bytes()
    }
}

impl std::convert::TryFrom<u16> for Type {
    type Error = Error;

    fn try_from(value: u16) -> Result<Self> {
        // First 2 bytes must be 0.
        if (value & 0xC000) != 0 {
            return Err(Error::Decoding);
        }

        // For details on how `Type` works and what the weird magic that follows is, see the
        // `From<Type> for u16` `impl` above.

        let mut m: u16 = 0;
        let mut c: u8 = 0;

        m += value & 0xf;
        m += (value & 0xe0) >> 1;
        m += (value & 0x3e00) >> 2;

        c += ((value & 0x10) >> 4) as u8;
        c += ((value & 0x100) >> 7) as u8;

        if let (Some(class), Some(method)) = (Class::n(c), Method::n(m)) {
            Ok(Self { class, method })
        } else {
            Err(Error::Decoding)
        }
    }
}

impl std::convert::TryFrom<[u8; 2]> for Type {
    type Error = Error;

    fn try_from(data: [u8; 2]) -> Result<Self> {
        Type::try_from(u16::from_be_bytes(data))
    }
}

/// This struct represents a STUN message in its entirety.
/// # Semantics
/// The struct cannot be mutated as, from [RFC 5389](datatracker.ietf.org/doc/html/rfc5389), "resends
/// of the same request reuse the same transaction ID, but the client must choose a new transaction
/// ID for new transactions unless the new request is bit-wise identical to the previous request and
/// sent from the same transport address to the same IP address." If you would like to resend the
/// request then you can use the same instance of `Message`. Otherwise, you must generate a new
/// `Message` instance that will have a different transaction ID.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Message {
    /// The transaction ID is a 96-bit identifier, used to uniquely identify stun transactions.
    ///
    /// It must be uniformly and randomly chosen from the interval 0 .. 2**96-1, and
    /// should be cryptographically random.
    tid: [u8; 12],
    ty: Type,
    attrs: Vec<Attribute>,
}

impl Message {
    pub fn new(ty: Type) -> Self {
        let mut tid = [0; 12];

        let mut rng = ChaCha20Rng::from_entropy();
        rng.fill_bytes(&mut tid);

        Self {
            tid,
            ty,
            attrs: Vec::new(),
        }
    }

    /// The transaction ID of the message.
    pub fn tid(&self) -> [u8; 12] {
        self.tid
    }

    /// The type of message.
    pub fn ty(&self) -> Type {
        self.ty
    }

    /// The attributes of the message.
    pub fn attrs(&self) -> Vec<Attribute> {
        self.attrs.clone()
    }

    /// The total length of the message excluding the header, but including padding.
    pub fn len(&self) -> usize {
        let mut result = 0;
        for attr in self.attrs.iter() {
            result += attr.len();
        }
        result
    }

    /// Append an attribute to the end of a message.
    ///
    /// Any attribute type may appear more than once in a STUN message. Unless specified otherwise,
    /// the order of appearance is significant: only the first occurrence needs to be processed by a
    /// receiver, and any duplicates may be ignored by a receiver.
    pub fn push(&mut self, mut attr: Attribute) {
        #[allow(clippy::single_match)]
        // Certain attributes require information about the message in order to be correctly encoded.
        match attr {
            Attribute::XorMappedAddress(ref mut a) => a.set_tid(self.tid),
            _ => {}
        }
        self.attrs.push(attr);
    }
}

impl std::convert::From<Message> for Vec<u8> {
    fn from(m: Message) -> Self {
        // The total size of the message sent is the length of the header (20 bytes) + the length of
        // the contents.
        let size = (20 + m.len()) as usize;
        let mut result = Vec::with_capacity(size);

        result.extend_from_slice(&<[u8; 2]>::from(m.ty));
        result.extend_from_slice(&(m.len() as u16).to_be_bytes());
        result.extend_from_slice(&MAGIC_COOKIE.to_be_bytes());
        result.extend_from_slice(&m.tid);
        for attr in m.attrs {
            result.extend(attr.to_bytes());
        }

        result
    }
}

impl std::convert::TryFrom<&[u8]> for Message {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self> {
        // The message must at least contain a header which is 20 bytes in length. This check
        // prevents future indexing of value from panicking.
        if value.len() < 20 {
            return Err(Error::Decoding);
        }

        let encoded_type = <[u8; 2]>::try_from(&value[0..2]).unwrap();
        let ty = Type::try_from(encoded_type)?;

        let encoded_len = <[u8; 2]>::try_from(&value[2..4]).unwrap();
        let len = u16::from_be_bytes(encoded_len);
        // The message can contain an unspecified number of zeroes at the end as `UdpSocket` just
        // reads into a buffer.
        let actual_length = {
            let mut len = value.len();
            for (index, item) in value.iter().rev().enumerate() {
                if (value.len() - index) % 4 == 0 {
                    len = value.len() - index;
                }
                if *item != 0 {
                    break;
                }
            }
            // The message will be at least of length 20, checked above. If the last 4 (or more)
            // bytes of the transaction ID are 0 for some unlikely reason (i.e. my shoddy tests)
            // then the calculations above could set `actual_length` to 16 which would cause
            // errors further on.
            std::cmp::max(len, 20)
        };

        // The message length must contain the size of the message in bytes, not including the
        // 20-byte STUN header.
        if len != actual_length as u16 - 20 {
            return Err(Error::Decoding);
        }

        // The Magic Cookie field must contain the fixed value 0x2112A442 in network byte order.
        if u32::from_be_bytes(<[u8; 4]>::try_from(&value[4..8]).unwrap()) != MAGIC_COOKIE {
            return Err(Error::Decoding);
        }

        let tid = <[u8; 12]>::try_from(&value[8..20]).unwrap();

        let mut attrs = Vec::new();
        // Attributes start after the 20 byte header
        let mut attr_start_index = 20;

        while attr_start_index < actual_length {
            let data_remainder = &value[(attr_start_index)..(actual_length)];
            let attr_result = Attribute::from_bytes(data_remainder.to_vec(), tid)?;
            attrs.push(attr_result.0);
            attr_start_index += attr_result.1;
        }

        Ok(Self { ty, tid, attrs })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stun::attribute::Software;
    use std::convert::TryFrom;

    #[test]
    fn test_message() {
        let descriptions = vec![
            "James is an ass, and we won't be working with him again.",
            "The E400 Sedan model arrives this year, boasting a 3.0L V6 biturbo engine producing \
             329 hp and 354 lb-ft of torque",
            "— the same powertrain that currently drives its E400 Coupe, Cabriolet and 4MATIC \
             Wagon cousins.",
        ];

        let mut attrs = Vec::new();
        for d in descriptions.iter() {
            let software = Software::try_from(*d).unwrap();
            attrs.push(Attribute::Software(software));
        }

        let ty = Type {
            class: Class::Request,
            method: Method::Binding,
        };
        let mut message = Message::new(ty);
        for attr in attrs.clone() {
            message.push(attr);
        }

        let message_bytes: Vec<u8> = message.into();

        // Each attribute is the length of their value + padding + a 4 byte header.
        let expected_len = descriptions
            .iter()
            .map(|d| d.len() + ((4 - (d.len() % 4)) % 4) + 4)
            .sum::<usize>() as u16;

        // Type
        assert_eq!(&message_bytes[0..2], &[0, 1]);
        // Size
        assert_eq!(&message_bytes[2..4], expected_len.to_be_bytes());
        // Magic cookie
        assert_eq!(&message_bytes[4..8], MAGIC_COOKIE.to_be_bytes());
        // Transaction ID exists
        let tid = message_bytes[8..20].to_vec();

        let message = Message::try_from(&message_bytes[..]).unwrap();

        assert_eq!(message.ty(), ty);
        assert_eq!(message.attrs(), attrs);
        assert_eq!(message.tid().to_vec(), tid);
    }

    #[test]
    fn test_message_decoding_err() {
        let tid = [0u8; 12];

        // Message too short
        let message_bytes = vec![0; 10];
        match Message::try_from(&message_bytes[..]) {
            Err(Error::Decoding) => {}
            _ => panic!(),
        }

        // Incorrect length
        let ty = <[u8; 2]>::from(Type {
            class: Class::Request,
            method: Method::Binding,
        });
        let incorrect_length = 10u16;
        let attr = Attribute::Software(Software::try_from("engadine maccas").unwrap());

        // Ensure that the test is valid.
        assert_ne!(attr.len(), incorrect_length as usize);

        let mut message_bytes: Vec<u8> = Vec::new();
        message_bytes.extend_from_slice(&ty);
        message_bytes.extend_from_slice(&incorrect_length.to_be_bytes());
        message_bytes.extend_from_slice(&MAGIC_COOKIE.to_be_bytes());
        message_bytes.extend_from_slice(&tid);
        message_bytes.extend(attr.to_bytes());

        match Message::try_from(&message_bytes[..]) {
            Err(Error::Decoding) => {}
            _ => panic!(),
        }

        // Incorrect MAGIC_COOKIE

        let mut message_bytes: Vec<u8> = Vec::new();
        message_bytes.extend_from_slice(&ty);
        message_bytes.extend_from_slice(&0u16.to_be_bytes());
        message_bytes.extend_from_slice(&[0u8; 4]);
        message_bytes.extend_from_slice(&tid);

        match Message::try_from(&message_bytes[..]) {
            Err(Error::Decoding) => {}
            _ => panic!(),
        }
    }

    #[test]
    fn test_message_class() {
        assert_eq!(0, Class::Request as u8);
        assert_eq!(1, Class::Indication as u8);
        assert_eq!(2, Class::Success as u8);
        assert_eq!(3, Class::Error as u8);

        assert_eq!(Class::Request, Class::n(0).unwrap());
        assert_eq!(Class::Indication, Class::n(1).unwrap());
        assert_eq!(Class::Success, Class::n(2).unwrap());
        assert_eq!(Class::Error, Class::n(3).unwrap());
    }

    #[test]
    fn test_message_method() {
        assert_eq!(1, Method::Binding as u8);
        assert_eq!(3, Method::Allocate as u8);
        assert_eq!(4, Method::Refresh as u8);
        assert_eq!(6, Method::Send as u8);
        assert_eq!(7, Method::Data as u8);
        assert_eq!(8, Method::CreatePermission as u8);
        assert_eq!(9, Method::ChannelBind as u8);

        assert_eq!(Method::Binding, Method::n(1).unwrap());
        assert_eq!(Method::Allocate, Method::n(3).unwrap());
        assert_eq!(Method::Refresh, Method::n(4).unwrap());
        assert_eq!(Method::Send, Method::n(6).unwrap());
        assert_eq!(Method::Data, Method::n(7).unwrap());
        assert_eq!(Method::CreatePermission, Method::n(8).unwrap());
        assert_eq!(Method::ChannelBind, Method::n(9).unwrap());
    }

    #[test]
    fn test_tid_random() {
        let mut messages = Vec::new();
        for _ in 0..100 {
            messages.push(Message::new(Type {
                class: Class::Request,
                method: Method::Binding,
            }));
        }
        let mut unique_tids: Vec<[u8; 12]> = messages.iter().map(|m| m.tid).collect();
        unique_tids.sort_unstable();
        unique_tids.dedup();
        assert_eq!(unique_tids.len(), 100);
    }

    #[test]
    fn test_type_conversions() {
        let class_masks = [
            (0b0000000000000000u16, Class::Request),
            (0b0000000000010000u16, Class::Indication),
            (0b0000000100000000u16, Class::Success),
            (0b0000000100010000u16, Class::Error),
        ];
        let method_masks = [
            (0b0000000000000001u16, Method::Binding),
            (0b0000000000000011u16, Method::Allocate),
            (0b0000000000000100u16, Method::Refresh),
            (0b0000000000000110u16, Method::Send),
            (0b0000000000000111u16, Method::Data),
            (0b0000000000001000u16, Method::CreatePermission),
            (0b0000000000001001u16, Method::ChannelBind),
        ];

        for (cmask, class) in class_masks {
            for (mmask, method) in method_masks {
                // From
                let result = Type::try_from(cmask | mmask).unwrap();
                let expected = Type { class, method };

                assert_eq!(result, expected);

                // Into
                let result = Type { class, method };
                let expected = cmask | mmask;

                assert_eq!(u16::from(result), expected);
                assert_eq!(<[u8; 2]>::from(result), expected.to_be_bytes());
            }
        }

        // Should result in an error as the first 2 bits aren't 0.
        match Type::try_from(0xFF) {
            Err(Error::Decoding) => {}
            _ => panic!(),
        }
    }
}
