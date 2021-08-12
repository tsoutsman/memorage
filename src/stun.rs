use crate::error::{Error, Result};

use std::convert::TryInto;

use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaCha20Rng,
};

/// The magic cookie field MUST contain the fixed value 0x2112A442 in network byte order.
static MAGIC_COOKIE: [u8; 4] = [0x21, 0x12, 0xA4, 0x42];

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Class {
    Request,
    Indication,
    Success,
    Error,
}

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Method {
    /// Binding method defined by [RFC 5389](https://tools.ietf.org/html/rfc5389).
    Binding = 1,
    /// Allocate method defined by [RFC 5766](https://tools.ietf.org/html/rfc5766).
    Allocate = 3,
    /// Refresh method defined by [RFC 5766](https://tools.ietf.org/html/rfc5766).
    Refresh,
    /// Send method defined by [RFC 5766](https://tools.ietf.org/html/rfc5766).
    Send = 6,
    /// Data method defined by [RFC 5766](https://tools.ietf.org/html/rfc5766).
    Data,
    /// CreatePermission method defined by [RFC 5766](https://tools.ietf.org/html/rfc5766).
    CreatePermission,
    /// ChannelBind method defined by [RFC 5766](https://tools.ietf.org/html/rfc5766).
    ChannelBind,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Length(u16);

impl Length {
    pub fn from(l: usize) -> Result<Self> {
        l.try_into()
    }
}

impl std::convert::From<Length> for u16 {
    fn from(l: Length) -> Self {
        // The last 2 bits of the length are padded.
        l.0 << 2
    }
}

impl std::convert::From<Length> for [u8; 2] {
    fn from(l: Length) -> Self {
        u16::from(l).to_be_bytes()
    }
}

impl std::convert::TryFrom<usize> for Length {
    type Error = Error;

    fn try_from(l: usize) -> Result<Self> {
        // Length is actually a u14 so it can't be larger than 16838.
        if l > 16383 {
            Err(Error::MessageTooLarge(l))
        } else {
            // The typecast is guaranteed to not panic as `l` is guaranteed to be less than
            // `u16::MAX`.
            Ok(Length(l as u16))
        }
    }
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
            result += 16;
        }
        // if the second bit is set
        if (t.class as u8 >> 1 & 1) != 0 {
            result += 256;
        }

        result
    }
}

impl std::convert::From<Type> for [u8; 2] {
    fn from(t: Type) -> Self {
        u16::from(t).to_be_bytes()
    }
}

/// This struct represents a STUN message in its entirety.
/// # Semantics
/// The struct cannot be mutated as, from [RFC 5389](datatracker.ietf.org/doc/html/rfc5389), "resends
/// of the same request reuse the same transaction ID, but the client MUST choose a new transaction
/// ID for new transactions unless the new request is bit-wise identical to the previous request and
/// sent from the same transport address to the same IP address." If you would like to resend the
/// request then you can use the same instance of `Message`. Otherwise, you must generate a new
/// `Message` instance that will have a different transaction ID.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Message {
    ty: Type,
    length: Length,
    /// The transaction ID is a 96-bit identifier, used to uniquely identify stun transactions.
    transaction_id: [u8; 12],
    message: Vec<u8>,
}

impl Message {
    pub fn new(ty: Type, message: &[u8]) -> Result<Self> {
        let mut transaction_id = [0; 12];

        // The transaction ID MUST be uniformly and randomly chosen from the interval 0 .. 2**96-1, and
        // SHOULD be cryptographically random.
        let mut rng = ChaCha20Rng::from_entropy();
        rng.fill_bytes(&mut transaction_id);

        Ok(Self {
            ty,
            length: Length::from(message.len())?,
            transaction_id,
            message: message.into(),
        })
    }
}

impl std::convert::From<Message> for Vec<u8> {
    fn from(m: Message) -> Self {
        // The total size of the package sent is the length of the header (20 bytes) + the length of
        // the message.
        let size = (20 + u16::from(m.length)) as usize;
        let mut result = Vec::with_capacity(size);

        result.extend_from_slice(&<[u8; 2]>::from(m.ty));
        result.extend_from_slice(&<[u8; 2]>::from(m.length));
        result.extend_from_slice(&MAGIC_COOKIE);
        result.extend_from_slice(&m.transaction_id);
        result.extend(m.message);

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message() {
        // TODO
    }

    #[test]
    fn test_from_len() {
        let length = Length::from(423).unwrap();

        assert_eq!(1692u16, length.into());
        assert_eq!([0x6, 0x9c], <[u8; 2]>::from(length));
    }

    #[test]
    #[should_panic]
    fn test_overflow_len() {
        Length::from(16384).unwrap();
    }

    #[test]
    fn test_from_type() {
        let ty = Type {
            class: Class::Request,
            method: Method::Binding,
        };

        assert_eq!(1u16, ty.into());
        assert_eq!([0, 1], <[u8; 2]>::from(ty));

        let ty = Type {
            class: Class::Indication,
            method: Method::Data,
        };

        assert_eq!(0x17u16, ty.into());
        assert_eq!([0, 0x17], <[u8; 2]>::from(ty));

        let ty = Type {
            class: Class::Success,
            method: Method::Refresh,
        };

        assert_eq!(0x104u16, ty.into());
        assert_eq!([1, 0x4], <[u8; 2]>::from(ty));

        let ty = Type {
            class: Class::Error,
            method: Method::ChannelBind,
        };

        assert_eq!(0x119u16, ty.into());
        assert_eq!([1, 0x19], <[u8; 2]>::from(ty));
    }

    #[test]
    fn test_message_class() {
        assert_eq!(0, Class::Request as u8);
        assert_eq!(1, Class::Indication as u8);
        assert_eq!(2, Class::Success as u8);
        assert_eq!(3, Class::Error as u8);
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
    }
}
