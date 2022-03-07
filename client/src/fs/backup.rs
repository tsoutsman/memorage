//! TODO: Add to mod.rs (if needed)
use crate::{
    crypto::{decrypt, encrypt},
    fs::EncryptedFile,
};

use memorage_core::PrivateKey;
use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    ser::{self, SerializeStruct},
    Deserialize, Serialize,
};

#[derive(Clone, Debug)]
pub struct Backup {
    /// The version of serialization used to create the backup.
    ///
    /// This is not the same as the version of memorage used, as the serialization of backups
    /// rarely changes.
    version: u16,
    /// The files contained within the backup.
    files: Vec<File>,
}

impl Backup {
    pub fn new(files: Vec<EncryptedFile>) -> Self {
        Self { version: 1, files }
    }

    pub fn push(&mut self, file: File) {
        self.files.push(file);
    }

    pub fn encrypt(self, key: &PrivateKey) -> EncryptedBackup<'_> {
        EncryptedBackup {
            version: self.version,
            key,
            files: self.files,
        }
    }
}

#[derive(Debug)]
pub struct EncryptedBackup<'a> {
    version: u16,
    key: &'a PrivateKey,
    files: Vec<File>,
}

impl<'a> Serialize for EncryptedBackup<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Backup", 3)?;
        state.serialize_field("version", &self.version)?;
        // TODO: Do we have to use a specific serialization implementation here?
        let files = bincode::serialize(&self.files).map_err(ser::Error::custom)?;
        let (nonce, encrypted_data) = encrypt(self.key, &files).map_err(ser::Error::custom)?;
        state.serialize_field("nonce", &nonce)?;
        state.serialize_field("data", &encrypted_data)?;
        state.end()
    }
}

#[derive(Debug)]
pub struct BackupDeserializer<'a>(&'a PrivateKey);

impl<'de, 'a> serde::de::DeserializeSeed<'de> for BackupDeserializer<'a> {
    type Value = Backup;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Version,
            Nonce,
            Data,
        }

        struct BackupVisitor<'a>(&'a PrivateKey);

        impl<'de, 'a> Visitor<'de> for BackupVisitor<'a> {
            type Value = Backup;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct Backup")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let version = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let nonce = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let encrypted_data = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                let data = decrypt(self.0, nonce, encrypted_data).map_err(de::Error::custom)?;
                // TODO: See serialize impl
                let files = bincode::deserialize(&data).map_err(de::Error::custom)?;
                Ok(Backup { version, files })
            }

            // I don't think this function is strictly necessary as we only ever use bincode
            // but might as well implement it for the sake of completeness.
            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut version = None;
                let mut nonce = None;
                let mut encrypted_data = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Version => {
                            if version.is_some() {
                                return Err(de::Error::duplicate_field("version"));
                            }
                            version = Some(map.next_value()?);
                        }
                        Field::Nonce => {
                            if nonce.is_some() {
                                return Err(de::Error::duplicate_field("nonce"));
                            }
                            nonce = Some(map.next_value()?);
                        }
                        Field::Data => {
                            if encrypted_data.is_some() {
                                return Err(de::Error::duplicate_field("data"));
                            }
                            encrypted_data = Some(map.next_value()?);
                        }
                    }
                }
                let version = version.ok_or_else(|| de::Error::missing_field("version"))?;
                let nonce = nonce.ok_or_else(|| de::Error::missing_field("nonce"))?;
                let encrypted_data =
                    encrypted_data.ok_or_else(|| de::Error::missing_field("data"))?;
                let data = decrypt(self.0, nonce, encrypted_data).map_err(de::Error::custom)?;
                // TODO: See serialize impl
                let files = bincode::deserialize(&data).map_err(de::Error::custom)?;
                Ok(Backup { version, files })
            }
        }

        const FIELDS: &[&str] = &["version", "nonce", "data"];
        deserializer.deserialize_struct("Backup", FIELDS, BackupVisitor(self.0))
    }
}
