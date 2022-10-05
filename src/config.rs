use std::{fs::File, path::Path};

use serde::{Deserialize, Serialize};

static CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub name: String,
    pub title: String,
    #[serde(default)]
    pub __is_dev_mode: bool,
}

impl Config {
    pub fn new<T>(name: T) -> Self
    where
        T: AsRef<str>,
    {
        let name = name.as_ref();
        Self {
            name: name.to_string(),
            title: name.to_string(),
            __is_dev_mode: false,
        }
    }

    pub fn load() -> anyhow::Result<Self> {
        let path = std::env::current_dir()?.join(CONFIG_FILE_NAME);
        let str = std::fs::read_to_string(&path)?;
        Ok(toml::from_str(&str)?)
    }

    pub fn to_toml_string(&self) -> anyhow::Result<String> {
        Ok(toml::to_string_pretty(&self)?)
    }

    pub fn save_to_file<P>(&self, path: P) -> anyhow::Result<()>
    where
        P: AsRef<Path>,
    {
        use std::io::Write;
        let config_toml = toml::to_string_pretty(&self)?;
        let mut file = {
            let path = path.as_ref().join(CONFIG_FILE_NAME);
            File::create(path)?
        };
        Ok(file.write_all(config_toml.as_bytes())?)
    }
}
