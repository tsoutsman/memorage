lazy_static::lazy_static! {
    static ref PROJECT_DIRS: directories_next::ProjectDirs = {
        match directories_next::ProjectDirs::from("org", "", "Memorage") {
            Some(p) => p,
            None => panic!("Can't find suitable folder for app data/config")
        }
    };
    pub static ref CONFIG_PATH: std::path::PathBuf = {
        PROJECT_DIRS.config_dir().to_owned().join("config.toml")
    };
    pub static ref DATA_PATH: std::path::PathBuf = {
        PROJECT_DIRS.data_dir().to_owned().join("data.toml")
    };
}

pub trait Persistent: serde::Serialize + serde::de::DeserializeOwned {
    fn default_path() -> &'static std::path::Path;

    fn from_disk<P>(path: Option<P>) -> crate::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        let content = match path {
            Some(p) => std::fs::read_to_string(p),
            None => std::fs::read_to_string(Self::default_path()),
        }?;
        Ok(toml::from_str(&content)?)
    }

    fn to_disk<P>(&self, path: Option<P>) -> crate::Result<()>
    where
        P: AsRef<std::path::Path>,
    {
        let toml = toml::to_string(&self)?;

        let path = match path {
            Some(ref p) => p.as_ref(),
            None => Self::default_path(),
        };

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(path, toml).map_err(|e| e.into())
    }
}

pub mod config;
pub mod data;

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::Path;

    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct Temp();

    impl Persistent for Temp {
        fn default_path() -> &'static std::path::Path {
            Path::new("/")
        }
    }

    #[test]
    fn persistent_create_dirs() {
        let mut root = tempfile::tempdir().unwrap().into_path();
        root.push("foo");
        root.push("bar");
        root.push("baz");

        assert!(matches!(Temp().to_disk(Some(root)), Ok(())));
    }
}
