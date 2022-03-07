use crate::fs::{EncryptedFile, EncryptedPath};

use serde::{Deserialize, Serialize};

pub trait Request: crate::net::protocol::private::Sealed {
    type Response: crate::net::protocol::response::Response + std::fmt::Debug;

    fn to_enum(self) -> RequestType;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestType {
    Ping(Ping),
    GetIndex(GetIndex),
    Add(Add),
    Edit(Edit),
    Rename(Rename),
    Delete(Delete),
    SetIndex(SetIndex),
}

impl crate::net::protocol::private::Sealed for RequestType {}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ping;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetIndex;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Add {
    name: EncryptedPath,
    file: EncryptedFile,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edit {
    name: EncryptedPath,
    contents: EncryptedFile,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rename {
    from: EncryptedPath,
    to: EncryptedPath,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Delete(pub EncryptedPath);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetIndex(pub crate::fs::index::Index);

macro_rules! impl_request {
    // IDK why this works with ident but not ty
    ($($t:ident),*$(,)?) => {
        $(
            impl crate::net::protocol::private::Sealed for $t {}

            impl Request for $t {
                type Response = crate::net::protocol::response::$t;

                fn to_enum(self) -> RequestType {
                    crate::net::protocol::request::RequestType::$t(self)
                }
            }
        )*
    };
}

impl_request![Ping, GetIndex, Add, Edit, Rename, Delete, SetIndex];
