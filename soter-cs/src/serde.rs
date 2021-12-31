pub trait SerializableRequestOrResponse: crate::private::Sealed {
    type TransmissionType: serde::Serialize;

    fn transmission_form(self) -> Self::TransmissionType;
}

// Ensures that the enum `RequestType` is deserialized and not the request structs
// such as `Register`.
pub trait DeserializableRequestOrResponse:
    crate::private::Sealed + serde::de::DeserializeOwned
{
}

impl<T> crate::serde::SerializableRequestOrResponse for T
where
    T: crate::request::Request,
{
    type TransmissionType = crate::request::RequestType;

    fn transmission_form(self) -> Self::TransmissionType {
        self.to_enum()
    }
}

impl<T> crate::private::Sealed for crate::Result<T> where T: crate::response::Response {}
impl<T> SerializableRequestOrResponse for crate::Result<T>
where
    T: crate::response::Response,
{
    type TransmissionType = Self;

    fn transmission_form(self) -> Self::TransmissionType {
        self
    }
}
impl<T> DeserializableRequestOrResponse for crate::Result<T> where T: crate::response::Response {}

impl DeserializableRequestOrResponse for crate::request::RequestType {}

// TODO error type
#[allow(clippy::result_unit_err)]
pub fn serialize<T>(r: T) -> Result<Vec<u8>, ()>
where
    T: SerializableRequestOrResponse,
{
    bincode::serialize(&r.transmission_form()).map_err(|_| ())
}

// TODO error type
#[allow(clippy::result_unit_err)]
pub fn deserialize<B, T>(r: B) -> Result<T, ()>
where
    B: AsRef<[u8]>,
    T: DeserializableRequestOrResponse,
{
    bincode::deserialize(r.as_ref()).map_err(|_| ())
}
