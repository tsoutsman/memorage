use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use memorage_core::{KeyPair, PublicKey};
use memorage_server::setup::Channels;

lazy_static::lazy_static! {
    pub static ref ID_1: Identity = {
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)), 1);
        let public_key = KeyPair::from_entropy().public;

        Identity {
            public_key,
            address,
        }
    };
    pub static ref ID_2: Identity = {
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(2, 3, 4, 5)), 2);
        let public_key = KeyPair::from_entropy().public;

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
) -> memorage_cs::Result<T::Response>
where
    T: memorage_cs::Serialize + memorage_cs::request::Request,
{
    let request = memorage_cs::serialize(request).unwrap();
    let output = memorage_server::__test_handle_request(
        Ok(request),
        identity.public_key,
        identity.address,
        channels,
    )
    .await
    .unwrap();
    memorage_cs::deserialize(output).unwrap()
}
