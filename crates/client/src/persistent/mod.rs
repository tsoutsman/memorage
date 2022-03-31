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

#[async_trait::async_trait]
pub trait Persistent: serde::Serialize + serde::de::DeserializeOwned {
    fn default_path() -> &'static std::path::Path;

    async fn from_disk<P>(path: Option<P>) -> crate::Result<Self>
    where
        P: AsRef<std::path::Path> + std::marker::Send,
    {
        let content = match path {
            Some(p) => tokio::fs::read_to_string(p).await,
            None => tokio::fs::read_to_string(Self::default_path()).await,
        }?;
        Ok(toml::from_str(&content)?)
    }

    async fn to_disk<P>(&self, path: Option<P>) -> crate::Result<()>
    where
        P: AsRef<std::path::Path> + std::marker::Send,
    {
        let toml = toml::to_string(&self)?;

        let path = match path {
            Some(ref p) => p.as_ref(),
            None => Self::default_path(),
        };

        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(path, toml).await.map_err(|e| e.into())
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

    #[tokio::test]
    async fn persistent_create_dirs() {
        let mut root = tempfile::tempdir().unwrap().into_path();
        root.push("foo");
        root.push("bar");
        root.push("baz");

        assert!(matches!(Temp().to_disk(Some(root)).await, Ok(())));
    }
}
