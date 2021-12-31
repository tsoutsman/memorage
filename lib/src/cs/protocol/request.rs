use crate::{
    cs::{key::PublicKey, Code},
    Verifiable,
};

use serde::{Deserialize, Serialize};

pub trait Request {
    type Response: serde::de::DeserializeOwned;

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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Register(pub PublicKey);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetKey(pub Code);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetSigningBytes;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestConnection {
    pub initiator_key: Verifiable<PublicKey>,
    pub target_key: PublicKey,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckConnection(pub Verifiable<PublicKey>);

impl std::ops::Deref for CheckConnection {
    type Target = Verifiable<PublicKey>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ping;

macro_rules! impl_request {
    // IDK why this works with ident but not ty
    ($($t:ident),*) => {
        $(impl Request for $t {
            type Response = crate::cs::protocol::response::$t;

            fn to_enum(self) -> RequestType {
                crate::cs::protocol::request::RequestType::$t(self)
            }
        })*
    };
}

impl_request![
    Register,
    GetKey,
    GetSigningBytes,
    RequestConnection,
    CheckConnection,
    Ping
];
