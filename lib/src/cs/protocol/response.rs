use crate::cs::{
    key::{PublicKey, SigningBytes},
    Code,
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Register(pub Code);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetKey(pub PublicKey);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetSigningBytes(pub SigningBytes);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestConnection;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckConnection(pub Option<PublicKey>);

// #[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
// pub struct Ping(pub Option<SocketAddr>);
