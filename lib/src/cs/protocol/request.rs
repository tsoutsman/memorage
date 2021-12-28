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
    /// Request to connect to a given [`VerifiablePublicKey`].
    RequestConnection {
        initiator: VerifiablePublicKey,
        target: PublicKey,
    },
    /// Request any [`SocketAddr`](std::net::SocketAddr) that have made a request to
    /// the given [`VerifiablePublicKey`].
    CheckConnection(VerifiablePublicKey),
    /// Request any socket addresses that have replied to the request for connection.
    Ping,
}
