mod file;
mod hash;
mod path;

pub use file::EncryptedFile;
pub use hash::hash;
pub use path::EncryptedPath;

pub mod index;

pub fn read_bin<P, T>(path: P) -> crate::Result<T>
where
    T: serde::de::DeserializeOwned,
    P: AsRef<std::path::Path>,
{
    bincode::deserialize_from(std::fs::File::open(path)?).map_err(|e| e.into())
}

pub fn write_bin<P, T>(path: P, value: &T) -> crate::Result<()>
where
    T: serde::Serialize,
    P: AsRef<std::path::Path>,
{
    bincode::serialize_into(std::fs::File::create(path)?, &value).map_err(|e| e.into())
}
