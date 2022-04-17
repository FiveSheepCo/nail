use std::fs::File;

use serde::{Deserialize, Serialize};

static CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub name: String,
    pub title: String,
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

    pub fn save_to_file(&self) -> anyhow::Result<()> {
        use std::io::Write;
        let config_toml = toml::to_string_pretty(&self)?;
        let mut file = {
            let path = std::env::current_dir()?.join(CONFIG_FILE_NAME);
            File::create(path)?
        };
        Ok(file.write_all(config_toml.as_bytes())?)
    }
}
