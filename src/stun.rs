#![allow(dead_code)]
use bytes::BytesMut;
use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaCha20Rng,
};

/// The magic cookie field MUST contain the fixed value 0x2112A442 in network byte order.
static MAGIC_COOKIE: u64 = 0x2112A442;

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

impl Length {
    fn from(l: u16) -> Self {
        // Length is actually a u14 so it can't be larger than 16838.
        if l > 16383 {
            panic!("length cannot be greater than 16383; got value {}", l);
        }

        Length(l)
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

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Message {
    ty: Type,
    length: Length,
    transaction_id: Option<BytesMut>,
    message: BytesMut,
}

/// This function returns a randomly generated transaction ID. The transaction ID is a 96-bit
/// identifier, used to uniquely identify STUN transactions.
pub fn gen_transaction_id() -> BytesMut {
    let mut buf = BytesMut::with_capacity(12);
    buf.resize(12, 0);

    let mut rng = ChaCha20Rng::from_entropy();
    rng.fill_bytes(&mut buf);

    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_len() {
        let length = Length::from(423);

        assert_eq!(1692u16, length.into());
        assert_eq!([0x6, 0x9c], <[u8; 2]>::from(length));
    }

    #[test]
    #[should_panic]
    fn test_overflow_len() {
        Length::from(16384);
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

    #[test]
    fn test_transaction_id() {
        for _ in 0..10 {
            let transaction_id = gen_transaction_id();
            assert_eq!(transaction_id.len(), 12);
        }
    }
}
