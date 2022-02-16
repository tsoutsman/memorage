use std::{
    net::{IpAddr, SocketAddr},
    path::Path,
    time::Duration,
};

use memorage_core::{KeyPair, PublicKey};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

// TODO: Other oses
lazy_static::lazy_static! {
    pub static ref CONFIG_PATH: std::path::PathBuf = {
        let mut home_dir = dirs::home_dir().unwrap();
        home_dir.push(".config");
        home_dir.push("memorage.config");
        home_dir
    };
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Config {
    #[serde(
        serialize_with = "serialize_key_pair",
        deserialize_with = "deserialize_key_pair"
    )]
    pub key_pair: KeyPair,
    pub server_address: IpAddr,
    pub peer: Option<PublicKey>,
    #[serde(
        serialize_with = "serialize_duration",
        deserialize_with = "deserialize_duration"
    )]
    pub peer_connection_schedule_delay: Duration,
    pub register_response: RetryConfig,
    pub request_connection: RetryConfig,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RetryConfig {
    pub tries: usize,
    #[serde(
        serialize_with = "serialize_duration",
        deserialize_with = "deserialize_duration"
    )]
    pub ping_delay: Duration,
}

impl RetryConfig {
    fn register_response() -> Self {
        Self {
            ping_delay: Duration::from_secs(3),
            tries: 20,
        }
    }

    fn request_connection() -> Self {
        Self {
            ping_delay: Duration::from_secs(5),
            tries: 4,
        }
    }
}

impl Config {
    pub fn from_disk(path: &Path) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    pub fn with_key_pair(key_pair: KeyPair) -> Self {
        key_pair.into()
    }

    pub fn server_socket_address(&self) -> SocketAddr {
        SocketAddr::new(self.server_address, memorage_core::PORT)
    }

    pub fn save_to_disk(&self, path: &Path) -> crate::Result<()> {
        let toml = toml::to_string(&self)?;
        std::fs::write(path, toml).map_err(|e| e.into())
    }
}

impl From<KeyPair> for Config {
    fn from(key_pair: KeyPair) -> Self {
        Self {
            key_pair,
            peer: None,
            // TODO
            server_address: IpAddr::V4(std::net::Ipv4Addr::new(172, 105, 182, 36)),
            peer_connection_schedule_delay: Duration::from_secs(10 * 60),
            register_response: RetryConfig::register_response(),
            request_connection: RetryConfig::request_connection(),
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

fn serialize_duration<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_f64(duration.as_secs_f64())
}

fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    struct DurationVisitor;

    impl de::Visitor<'_> for DurationVisitor {
        type Value = Duration;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a float")
        }

        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Duration::from_secs_f64(v))
        }
    }

    deserializer.deserialize_f64(DurationVisitor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_config() {
        let config = Config::from(KeyPair::from_entropy());
        let mut path = std::env::temp_dir();
        path.push("memorage.config");
        assert!(config.save_to_disk(&path).is_ok());
        assert_eq!(Config::from_disk(&path).unwrap(), config);
    }
}
