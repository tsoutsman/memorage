use crate::{Error, Result};

use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct RootDirectory(PathBuf);

impl RootDirectory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn file_name<P>(&self, path: P) -> Result<PathBuf>
    where
        P: AsRef<Path>,
    {
        let mut components = path.as_ref().components();

        if let Some(component) = components.next() {
            if let Component::Normal(_) = component {
                if components.next().is_none() {
                    let mut file_name = self.0.clone();
                    file_name.push(component);
                    return Ok(file_name);
                }
            }
        }

        tracing::error!("peer sent malicious file name");
        Err(Error::MaliciousFileName)
    }
}

impl From<PathBuf> for RootDirectory {
    fn from(p: PathBuf) -> Self {
        Self(p)
    }
}

impl<T> From<&T> for RootDirectory
where
    T: ?Sized + AsRef<std::ffi::OsStr>,
{
    fn from(t: &T) -> Self {
        Self(t.into())
    }
}

impl AsRef<Path> for RootDirectory {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl std::fmt::Display for RootDirectory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

impl std::str::FromStr for RootDirectory {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_file_name() {
        let root: RootDirectory = "/".into();
        let path: PathBuf = "foo".into();

        let file_name = root.file_name(path).unwrap();
        assert_eq!(PathBuf::from("/foo"), file_name);
    }

    #[test]
    fn invalid_file_name() {
        let root: RootDirectory = "/foo".into();

        let path: PathBuf = "/foo".into();
        assert!(matches!(
            root.file_name(path),
            Err(Error::MaliciousFileName)
        ));

        let path: PathBuf = "/bar".into();
        assert!(matches!(
            root.file_name(path),
            Err(Error::MaliciousFileName)
        ));

        let path: PathBuf = "..".into();
        assert!(matches!(
            root.file_name(path),
            Err(Error::MaliciousFileName)
        ));

        let path: PathBuf = "../foo".into();
        assert!(matches!(
            root.file_name(path),
            Err(Error::MaliciousFileName)
        ));
    }
}
