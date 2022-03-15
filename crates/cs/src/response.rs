use std::net::SocketAddr;

use crate::PairingCode;

use memorage_core::{time::OffsetDateTime, PublicKey};
use serde::{Deserialize, Serialize};

pub trait Response:
    crate::private::Sealed + serde::Serialize + serde::de::DeserializeOwned
{
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Register(pub PairingCode);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetKey(pub PublicKey);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetRegisterResponse(pub PublicKey);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestConnection;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckConnection {
    pub initiator: PublicKey,
    #[serde(serialize_with = "crate::time::serialize_offset_date_time")]
    #[serde(deserialize_with = "crate::time::deserialize_offset_date_time")]
    pub time: OffsetDateTime,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ping(pub SocketAddr);

macro_rules! impl_response {
    ($($t:ident),*$(,)?) => {
        $(
            impl crate::private::Sealed for $t {}

            impl Response for $t {}
        )*
    };
}

impl_response![
    Register,
    GetKey,
    GetRegisterResponse,
    RequestConnection,
    CheckConnection,
    Ping,
];
