use crate::persistent::{Persistent, CONFIG_PATH};

use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    time::Duration,
};

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub server_address: Vec<IpAddr>,
    /// Path to backup.
    pub backup_path: PathBuf,
    /// Path at which the peer's encrypted data is stored.
    pub backup_storage_path: PathBuf,
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

impl Default for Config {
    fn default() -> Self {
        Self {
            server_address: vec![],
            backup_path: PathBuf::new(),
            backup_storage_path: PathBuf::new(),
            peer_connection_schedule_delay: Duration::from_secs(600),
            register_response: RetryConfig::register_response(),
            request_connection: RetryConfig::request_connection(),
        }
    }
}

impl Persistent for Config {
    fn default_path() -> &'static std::path::Path {
        &CONFIG_PATH
    }
}

impl Config {
    pub fn server_socket_addresses(&self) -> Vec<SocketAddr> {
        self.server_address
            .iter()
            .map(|a| SocketAddr::new(*a, memorage_core::PORT))
            .collect()
    }
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
        let config = Config::default();

        let mut path = std::env::temp_dir();
        path.push("config.toml");

        assert!(config.to_disk(Some(&path)).is_ok());
        assert_eq!(Config::from_disk(Some(&path)).unwrap(), config);
    }
}
