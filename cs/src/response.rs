use std::net::SocketAddr;

use crate::PairingCode;

use serde::{Deserialize, Serialize};
use memorage_core::{time::OffsetDateTime, PublicKey};

pub trait Response:
    crate::private::Sealed + serde::Serialize + serde::de::DeserializeOwned
{
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Register(pub PairingCode);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetKey(pub PublicKey);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterResponse(pub PublicKey);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestConnection;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckConnection {
    pub initiator: PublicKey,
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
    RegisterResponse,
    RequestConnection,
    CheckConnection,
    Ping,
];
