use crate::net::protocol::{private::Sealed, request, response, Result};

pub trait Serialize: Sealed {
    type TransmissionType: serde::Serialize;
    fn transmission_form(&self) -> Self::TransmissionType;
}

// Ensures that the enum `RequestType` is deserialized and not the request structs
// such as `Register`.
pub trait Deserialize: Sealed + serde::de::DeserializeOwned {}

impl<T> Sealed for Result<T> where T: response::Response {}

// Serializable Types
impl<T> Serialize for T
where
    T: request::Request,
{
    type TransmissionType = request::RequestType;

    fn transmission_form(&self) -> Self::TransmissionType {
        self.to_enum()
    }
}
impl<T> Serialize for Result<T>
where
    T: response::Response + Clone,
{
    type TransmissionType = Self;

    fn transmission_form(&self) -> Self::TransmissionType {
        self.clone()
    }
}

// Deserializable Types
impl<T> Deserialize for Result<T> where T: response::Response {}
impl Deserialize for request::RequestType {}

// Functions
pub fn serialize<T>(r: &T) -> bincode::Result<Vec<u8>>
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
