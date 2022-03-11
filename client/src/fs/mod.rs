mod hash;
mod index;
mod path;

pub use hash::hash;
pub use index::{Index, IndexDifference};
pub use path::HashedPath;

pub fn read_bin<P, T>(path: P) -> crate::Result<T>
where
    T: serde::de::DeserializeOwned,
    P: AsRef<std::path::Path>,
{
    bincode::deserialize_from(std::fs::File::open(path)?).map_err(|e| e.into())
}

pub fn write_bin<P, T>(path: P, value: &T, overwrite: bool) -> crate::Result<()>
where
    T: serde::Serialize,
    P: AsRef<std::path::Path>,
{
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(!overwrite)
        .open(path)?;
    bincode::serialize_into(file, &value).map_err(|e| e.into())
}

pub fn encrypt_file<P>(
    key: &memorage_core::PrivateKey,
    path: P,
) -> crate::Result<crate::crypto::Encrypted<Vec<u8>>>
where
    P: AsRef<std::path::Path>,
{
    crate::crypto::Encrypted::encrypt(key, &std::fs::read(path)?)
}
