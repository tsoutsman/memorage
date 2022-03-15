mod hash;
mod path;
mod root;

pub mod index;

pub use hash::hash;
pub use path::HashedPath;
pub use root::RootDirectory;

pub fn read_bin<P, T>(path: P) -> crate::Result<T>
where
    T: serde::de::DeserializeOwned,
    P: AsRef<std::path::Path>,
{
    bincode::deserialize_from(std::io::BufReader::new(std::fs::File::open(path)?))
        .map_err(|e| e.into())
}

pub fn write_bin<P, T>(path: P, value: &T) -> crate::Result<()>
where
    T: serde::Serialize,
    P: AsRef<std::path::Path>,
{
    bincode::serialize_into(std::io::BufWriter::new(std::fs::File::create(path)?), value)
        .map_err(|e| e.into())
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
