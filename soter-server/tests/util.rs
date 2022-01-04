use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use soter_core::{KeyPair, PublicKey};
use soter_server::setup::Channels;

lazy_static::lazy_static! {
    pub static ref ID_1: Identity = {
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)), 1);
        let rand = soter_core::rand::SystemRandom::new();
        let public_key = KeyPair::generate(&rand).unwrap().public_key();

        Identity {
            public_key,
            address,
        }
    };
    pub static ref ID_2: Identity = {
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(2, 3, 4, 5)), 2);
        let rand = soter_core::rand::SystemRandom::new();
        let public_key = KeyPair::generate(&rand).unwrap().public_key();

        Identity {
            public_key,
            address,
        }
    };
}

pub struct Identity {
    pub public_key: PublicKey,
    pub address: SocketAddr,
}

pub async fn request<T>(
    request: T,
    identity: &Identity,
    channels: Channels,
) -> soter_cs::Result<T::Response>
where
    T: soter_cs::Serialize + soter_cs::request::Request,
{
    let request = soter_cs::serialize(request).unwrap();
    let output = soter_server::__test_handle_request(
        Ok(request),
        identity.public_key,
        identity.address,
        channels,
    )
    .await
    .unwrap();
    soter_cs::deserialize(output).unwrap()
}
