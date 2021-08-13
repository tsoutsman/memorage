use crate::stun::attribute::Attribute;

use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaCha20Rng,
};

/// The magic cookie field must contain the fixed value 0x2112A442 in network byte order.
pub static MAGIC_COOKIE: u32 = 0x2112A442;

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

        let mut message = Message::new(Type {
            class: Class::Request,
            method: Method::Binding,
        });
        for attr in attrs {
            message.push(attr);
        }

        let message: Vec<u8> = message.into();

        // Each attribute is the length of their value + padding + a 4 byte header.
        let expected_len = descriptions
            .iter()
            .map(|d| d.len() + ((4 - (d.len() % 4)) % 4) + 4)
            .sum::<usize>() as u16;

        // Type
        assert_eq!(&message[0..2], &[0, 1]);
        // Size
        assert_eq!(&message[2..4], expected_len.to_be_bytes());
        // Magic cookie
        assert_eq!(&message[4..8], MAGIC_COOKIE.to_be_bytes());
        // Transaction ID exists
        let _ = &message[8..20];

        // let message: Message = message.into();
        // for attr in message.attrs {
        //     //
        // }
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
}
