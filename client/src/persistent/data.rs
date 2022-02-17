use crate::persistent::{Persistent, DATA_PATH};

use memorage_core::{KeyPair, PublicKey};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Data {
    #[serde(
        serialize_with = "serialize_key_pair",
        deserialize_with = "deserialize_key_pair"
    )]
    pub key_pair: KeyPair,
    pub peer: Option<PublicKey>,
}

impl Persistent for Data {
    fn default_path() -> &'static std::path::Path {
        &DATA_PATH
    }
}

impl Data {
    pub fn from_key_pair(key_pair: KeyPair) -> Self {
        Self {
            key_pair,
            peer: None,
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
