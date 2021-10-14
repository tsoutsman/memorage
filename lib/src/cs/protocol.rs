use std::time::Duration;

use crate::cs::Code;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClientRequest {
    Register(String),
    GetKey(Code),
    EstablishConnection(String),
    Ping,
}

pub type ServerResponse = Result<SuccesfulResponse, Error>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuccesfulResponse {
    Register(Code),
    GetKey(String),
    EstablishConnection,
    Ping(Option<Duration>),
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
