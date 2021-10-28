use std::net::SocketAddr;

use crate::cs::{
    key::{PublicKey, VerifiablePublicKey},
    Code,
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClientRequest {
    /// Request to associate the [`PublicKey`] to a temporary randomised code.
    Register(PublicKey),
    /// Request to get the [`PublicKey`] associated with a given code.
    GetKey(Code),
    GetSigningBytes,
    /// Request to connect to a given [`PublicKey`].
    RequestConnection {
        initiator: VerifiablePublicKey,
        target: PublicKey,
    },
    CheckConnection(VerifiablePublicKey),
    Ping,
}

pub type ServerResponse = Result<SuccesfulResponse, Error>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuccesfulResponse {
    Register(Code),
    GetKey(PublicKey),
    RequestConnection,
    CheckConnection(Option<PublicKey>),
    Ping(Option<SocketAddr>),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Error {
    InvalidCode,
    Generic,
    Serialization,
}

impl From<bincode::Error> for Error {
    fn from(_: bincode::Error) -> Self {
        Self::Serialization
    }
}

impl From<tokio::sync::oneshot::error::RecvError> for Error {
    fn from(_: tokio::sync::oneshot::error::RecvError) -> Self {
        Self::Generic
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for Error {
    fn from(_: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Self::Generic
    }
}

impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self {
        Self::Generic
    }
}
