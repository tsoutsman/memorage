use crate::PairingCode;

use serde::{Deserialize, Serialize};
use soter_core::{PublicKey, Verifiable};

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
    GetSigningBytes(GetSigningBytes),
    /// Request to connect to a given [`PublicKey`].
    RequestConnection(RequestConnection),
    CheckConnection(CheckConnection),
    /// Request any socket addresses that have replied to the request for connection.
    Ping(Ping),
}

impl crate::private::Sealed for crate::request::RequestType {}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Register(pub PublicKey);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetKey(pub PairingCode);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetSigningBytes;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestConnection {
    pub initiator_key: Verifiable<PublicKey>,
    pub target_key: PublicKey,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckConnection(pub Verifiable<PublicKey>);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ping;

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
    GetSigningBytes,
    RequestConnection,
    CheckConnection,
    Ping,
];
