lazy_static::lazy_static! {
    static ref CONFIG_PATH: std::path::PathBuf = {
        match directories_next::ProjectDirs::from("org", "", "Memorage") {
            Some(p) => p.config_dir().to_owned().join("config.toml"),
            None => panic!("Can't find suitable folder for app config")
        }
    };
    static ref DATA_PATH: std::path::PathBuf = {
        match directories_next::ProjectDirs::from("org", "", "Memorage") {
            Some(p) => p.data_dir().to_owned().join("data.toml"),
            None => panic!("Can't find suitable folder for app data")
        }
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
        std::fs::write(path, toml).map_err(|e| e.into())
    }
}

pub mod config;
pub mod data;
