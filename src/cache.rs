use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct HashCache {
    config: u32,
    posts: HashMap<String, u32>,
    templates: HashMap<String, u32>,
}

impl HashCache {
    pub fn new() -> anyhow::Result<Self> {
        let base_path = std::env::current_dir()?;
        let config = Self::hash_file(base_path.join("config.toml"))?;
        let posts = Self::hash_directory(base_path.join("posts"))?;
        let templates = Self::hash_directory(base_path.join("templates"))?;
        Ok(Self {
            config,
            posts,
            templates,
        })
    }

    fn hash_file<P>(path: P) -> anyhow::Result<u32>
    where
        P: AsRef<Path>,
    {
        use std::io::Read;
        let mut buf = Vec::<u8>::new();
        std::fs::File::open(path)?.read_to_end(&mut buf)?;
        Ok(crc::Crc::<u32>::new(&crc::CRC_32_CKSUM).checksum(&buf))
    }

    fn hash_directory<P>(path: P) -> anyhow::Result<HashMap<String, u32>>
    where
        P: AsRef<Path>,
    {
        let mut map = HashMap::<String, u32>::new();
        for dir_entry in path.as_ref().read_dir()? {
            let dir_entry = dir_entry?;
            let path = dir_entry.path().display().to_string();
            let hash = Self::hash_file(dir_entry.path())?;
            map.insert(path, hash);
        }
        Ok(map)
    }
}
