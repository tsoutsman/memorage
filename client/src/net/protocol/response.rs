use serde::{Deserialize, Serialize};

pub trait Response:
    crate::net::protocol::private::Sealed + serde::Serialize + serde::de::DeserializeOwned
{
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ping;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetIndex(pub crate::fs::index::Index);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Add;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edit;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rename;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Delete;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetIndex;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Complete(
    /// Whether the backup is complete.
    ///
    /// Returning true ends communication. Returning false switches the roles
    /// with the responder now sending requests.
    pub bool,
);

macro_rules! impl_response {
    ($($t:ident),*$(,)?) => {
        $(
            impl crate::net::protocol::private::Sealed for $t {}

            impl Response for $t {}
        )*
    };
}

impl_response![Ping, GetIndex, Add, Edit, Rename, Delete, SetIndex, Complete];
