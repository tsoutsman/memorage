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

    fn to_disk(&self) -> crate::Result<()> {
        let toml = toml::to_string(&self)?;
        // TODO: Create directories
        std::fs::write(Self::default_path(), toml).map_err(|e| e.into())
    }
}

pub mod config;
pub mod data;
