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

    fn from_disk(path: Option<&std::path::Path>) -> crate::Result<Self> {
        let path = match path {
            Some(p) => p,
            None => Self::default_path(),
        };
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    fn to_disk(&self, path: Option<&std::path::Path>) -> crate::Result<()> {
        let path = match path {
            Some(p) => p,
            None => Self::default_path(),
        };
        let toml = toml::to_string(&self)?;
        // TODO: Create directories
        std::fs::write(path, toml).map_err(|e| e.into())
    }
}

pub mod config;
pub mod data;
