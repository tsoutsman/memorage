use serde::{Deserialize, Serialize};

pub trait Response:
    crate::net::protocol::private::Sealed + serde::Serialize + serde::de::DeserializeOwned
{
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ping;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetIndex {
    // TODO: Wide copy into stream like GetFile?
    pub index: Option<crate::crypto::Encrypted<crate::fs::index::Index>>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetFile {
    pub len: Option<u64>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Write;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rename;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Delete;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetIndex;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Complete;

macro_rules! impl_response {
    ($($t:ident),*$(,)?) => {
        $(
            impl crate::net::protocol::private::Sealed for $t {}

            impl Response for $t {}
        )*
    };
}

impl_response![Ping, GetIndex, GetFile, Write, Rename, Delete, SetIndex, Complete];
