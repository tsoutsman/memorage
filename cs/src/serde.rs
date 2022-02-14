pub use bincode::{Error, Result};

pub trait Serialize: crate::private::Sealed {
    type TransmissionType: serde::Serialize;

    fn transmission_form(self) -> Self::TransmissionType;
}

// Ensures that the enum `RequestType` is deserialized and not the request structs
// such as `Register`.
pub trait Deserialize: crate::private::Sealed + serde::de::DeserializeOwned {}

impl<T> crate::private::Sealed for crate::Result<T> where T: crate::response::Response {}

// Serializable Types
impl<T> Serialize for T
where
    T: crate::request::Request,
{
    type TransmissionType = crate::request::RequestType;

    fn transmission_form(self) -> Self::TransmissionType {
        self.to_enum()
    }
}
impl<T> Serialize for crate::Result<T>
where
    T: crate::response::Response,
{
    type TransmissionType = Self;

    fn transmission_form(self) -> Self::TransmissionType {
        self
    }
}

// Deserializable Types
impl<T> Deserialize for crate::Result<T> where T: crate::response::Response {}
impl Deserialize for crate::request::RequestType {}

// Functions
pub fn serialize<T>(r: T) -> bincode::Result<Vec<u8>>
where
    T: Serialize,
{
    bincode::serialize(&r.transmission_form())
}

pub fn deserialize<B, T>(r: B) -> bincode::Result<T>
where
    B: AsRef<[u8]>,
    T: Deserialize,
{
    bincode::deserialize(r.as_ref())
}
