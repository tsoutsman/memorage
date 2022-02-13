use crate::PairingCode;

use serde::{Deserialize, Serialize};
use soter_core::{time::OffsetDateTime, PublicKey};

pub trait Request: crate::private::Sealed {
    type Response: crate::response::Response;

    fn to_enum(self) -> RequestType;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestType {
    /// Request to associate the [`PublicKey`] to a temporary randomised code.
    Register(Register),
    /// Request to get the [`PublicKey`] associated with a given code.
    GetKey(GetKey),
    /// Request any [`PublicKey`] that used the client's [`PairingCode`].
    RegisterResponse(RegisterResponse),
    /// Request to connect to a given [`PublicKey`].
    RequestConnection(RequestConnection),
    /// Request any socket addresses that have requested a connection.
    CheckConnection(CheckConnection),
    Ping(Ping),
}

impl crate::private::Sealed for crate::request::RequestType {}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Register;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetKey(pub PairingCode);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterResponse;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestConnection {
    pub target: PublicKey,
    pub time: OffsetDateTime,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckConnection;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ping(pub PublicKey);

macro_rules! impl_request {
    // IDK why this works with ident but not ty
    ($($t:ident),*$(,)?) => {
        $(
            impl crate::private::Sealed for $t {}

            impl Request for $t {
                type Response = crate::response::$t;

                fn to_enum(self) -> RequestType {
                    crate::request::RequestType::$t(self)
                }
            }
        )*
    };
}

impl_request![
    Register,
    GetKey,
    RegisterResponse,
    RequestConnection,
    CheckConnection,
    Ping,
];
