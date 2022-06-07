mod path;
mod root;

pub mod index;

pub use path::HashedPath;
pub use root::RootDirectory;

pub fn hash<T>(reader: T) -> crate::Result<[u8; 32]>
where
    T: std::io::Read,
{
    let mut hasher = blake3::Hasher::new();
    crate::util::sync_wide_copy(reader, &mut hasher)?;
    Ok(hasher.finalize().into())
}
