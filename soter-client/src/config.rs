use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use soter_core::{KeyPair, PublicKey};

// TODO: Other oses
lazy_static::lazy_static! {
    static ref CONFIG_PATH: std::path::PathBuf = {
        let mut home_dir = dirs::home_dir().unwrap();
        home_dir.push(".config");
        home_dir.push("soter.config");
        home_dir
    };
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(
        serialize_with = "serialize_key_pair",
        deserialize_with = "deserialize_key_pair"
    )]
    pub key_pair: KeyPair,
    pub peer: Option<PublicKey>,
    pub server_address: IpAddr,
    pub request_connection: RequestConnectionConfig,
    pub peer_connection_schedule_delay: Duration,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct RequestConnectionConfig {
    pub ping_delay: Duration,
    pub tries: usize,
}

impl Default for RequestConnectionConfig {
    fn default() -> Self {
        Self {
            ping_delay: Duration::from_secs(5),
            tries: 4,
        }
    }
}

impl Config {
    pub fn from_disk() -> crate::Result<Self> {
        let content = std::fs::read_to_string(&*CONFIG_PATH)?;
        Ok(toml::from_str(&content)?)
    }

    pub fn with_key_pair(key_pair: KeyPair) -> Self {
        key_pair.into()
    }

    pub fn server_socket_address(&self) -> SocketAddr {
        SocketAddr::new(self.server_address, soter_core::PORT)
    }

    pub fn save_to_disk(&self) -> crate::Result<()> {
        let toml = toml::to_string(&self)?;
        std::fs::write(&*CONFIG_PATH, toml).map_err(|e| e.into())
    }
}

impl From<KeyPair> for Config {
    fn from(key_pair: KeyPair) -> Self {
        Self {
            key_pair,
            peer: None,
            // TODO
            server_address: IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
            request_connection: RequestConnectionConfig::default(),
            peer_connection_schedule_delay: Duration::from_secs(10 * 60),
        }
    }
}

// I really don't want to derive Serialize and Deserialize for PrivateKey (and by extension
// KeyPair).

fn serialize_key_pair<S>(key_pair: &KeyPair, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serde_bytes::Bytes::new(&key_pair.to_pkcs8()).serialize(serializer)
}

fn deserialize_key_pair<'de, D>(deserializer: D) -> Result<KeyPair, D::Error>
where
    D: Deserializer<'de>,
{
    let bytes = <serde_bytes::ByteBuf>::deserialize(deserializer)?;
    KeyPair::try_from_pkcs8(bytes.as_ref()).map_err(serde::de::Error::custom)
}
