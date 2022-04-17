use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{build::Post, config::Config};

#[derive(Serialize, Deserialize, Debug)]
pub struct HashCache {
    config: u32,
    posts: HashMap<PathBuf, u32>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FileDiffMode {
    Unchanged,
    Added,
    Updated,
    Removed,
}

#[derive(Debug)]
pub struct HashDiff {
    config: FileDiffMode,
    posts: Vec<(PathBuf, FileDiffMode)>,
}

impl HashCache {
    pub fn empty() -> Self {
        Self {
            config: 0,
            posts: HashMap::new(),
        }
    }

    pub fn read_from_file() -> anyhow::Result<Self> {
        let path = std::env::current_dir()?.join(".cache.toml");
        let str = std::fs::read_to_string(&path)?;
        Ok(toml::from_str(&str)?)
    }

    pub fn save_to_file(&self) -> anyhow::Result<()> {
        use std::io::Write;
        let cache_toml = toml::to_string_pretty(&self)?;
        let path = std::env::current_dir()?.join(".cache.toml");
        let mut file = std::fs::File::create(&path)?;
        let _ = file.write(cache_toml.as_bytes())?;
        Ok(())
    }

    pub fn mix_post(&mut self, post: &Post) {
        self.posts.insert(
            post.filename.clone(),
            Self::hash_contents(post.contents.as_str()),
        );
    }

    pub fn mix_config(&mut self, config: &Config) -> anyhow::Result<()> {
        Ok(self.config = Self::hash_contents(config.to_toml_string()?))
    }

    pub fn diff(&self, hashes: &HashCache) -> HashDiff {
        fn diff_entries(
            a: &HashMap<PathBuf, u32>,
            b: &HashMap<PathBuf, u32>,
        ) -> Vec<(PathBuf, FileDiffMode)> {
            let mut diffs = Vec::<(PathBuf, FileDiffMode)>::new();
            for (key, value) in a {
                diffs.push((
                    key.to_owned(),
                    if b.contains_key(key) {
                        if value == &b[key] {
                            FileDiffMode::Unchanged
                        } else {
                            FileDiffMode::Updated
                        }
                    } else {
                        FileDiffMode::Removed
                    },
                ))
            }
            for key in b.keys().filter(|k| !a.contains_key(k.clone())) {
                diffs.push((key.to_owned(), FileDiffMode::Added));
            }
            diffs
        }
        let post_diffs = diff_entries(&self.posts, &hashes.posts);
        let config_diff = if self.config == hashes.config {
            FileDiffMode::Unchanged
        } else {
            FileDiffMode::Updated
        };
        HashDiff {
            config: config_diff,
            posts: post_diffs,
        }
    }

    fn hash_contents<S>(str: S) -> u32
    where
        S: AsRef<str>,
    {
        crc::Crc::<u32>::new(&crc::CRC_32_CKSUM).checksum(str.as_ref().as_bytes())
    }
}

impl HashDiff {
    pub fn sync(&self) -> anyhow::Result<()> {
        for (path, _) in self
            .posts
            .iter()
            .filter(|(_, mode)| mode == &FileDiffMode::Removed)
        {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }

    pub fn changed_post_paths(&self) -> Vec<PathBuf> {
        self.posts
            .iter()
            .filter(|(_, mode)| [FileDiffMode::Added, FileDiffMode::Updated].contains(mode))
            .map(|(path, _)| path.clone())
            .collect::<Vec<_>>()
    }

    pub fn should_rerender_index_page(&self) -> bool {
        let config_changed = self.config == FileDiffMode::Updated;
        let posts_changed = self
            .posts
            .iter()
            .any(|(_, mode)| mode != &FileDiffMode::Unchanged);
        config_changed || posts_changed
    }
}
