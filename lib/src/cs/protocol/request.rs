use crate::cs::{
    key::{PublicKey, VerifiablePublicKey},
    Code,
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Request {
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
