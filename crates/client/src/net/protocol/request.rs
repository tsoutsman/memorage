use crate::{
    crypto::Encrypted,
    fs::{index::Index, HashedPath},
};

use serde::{Deserialize, Serialize};

pub trait Request: crate::net::protocol::private::Sealed {
    type Response: crate::net::protocol::response::Response + std::fmt::Debug;
    fn to_enum(&self) -> RequestType;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestType {
    Ping(Ping),
    GetIndex(GetIndex),
    GetFile(GetFile),
    Write(Write),
    Rename(Rename),
    Delete(Delete),
    SetIndex(SetIndex),
    Complete(Complete),
}

impl crate::net::protocol::private::Sealed for RequestType {}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ping;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetIndex;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetFile {
    pub name: HashedPath,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Write {
    pub name: HashedPath,
    pub contents_len: u64,
}

/// Rename a file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rename {
    pub from: HashedPath,
    pub to: HashedPath,
}

/// Delete the file at the given path.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Delete {
    pub name: HashedPath,
}

/// Set the index on the peer to the given index.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetIndex {
    pub index: Encrypted<Index>,
}

/// Signify that syncing is complete.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Complete {
    /// The index of the peer's files on this computer.
    ///
    /// The peer uses this to decide whether or not they need to update their
    /// backup.
    ///
    /// When the receiver has completed syncing, the index they send back is
    /// empty.
    pub index: Option<Encrypted<Index>>,
}

macro_rules! impl_request {
    // IDK why this works with ident but not ty
    ($($t:ident),*$(,)?) => {
        $(
            impl crate::net::protocol::private::Sealed for $t {}

            impl Request for $t {
                type Response = crate::net::protocol::response::$t;

                fn to_enum(&self) -> RequestType {
                    // TODO: Remove clone
                    crate::net::protocol::request::RequestType::$t(self.clone())
                }
            }
        )*
    };
}

impl_request![Ping, GetIndex, GetFile, Write, Rename, Delete, SetIndex, Complete];
