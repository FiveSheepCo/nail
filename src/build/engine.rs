use std::{collections::HashMap, io::Write, path::PathBuf};

use crate::{
    cache::HashCache,
    config::Config,
    post_format::PostFormat,
    post_metadata::PostMetadata,
    theme::{RenderData, Theme},
};

struct DirectoryStructure {
    pub build_dir: PathBuf,
    pub build_post_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Post {
    pub format: PostFormat,
    pub filename: PathBuf,
    pub metadata: PostMetadata,
    pub contents: String,
}

impl Post {
    pub fn get_final_file_name(&self) -> String {
        let snake_case_name = self.metadata.title.replace(' ', "_").to_ascii_lowercase();
        format!("{}.html", snake_case_name)
    }
}

#[derive(Debug)]
pub struct Engine {
    last_cache: HashCache,
    current_cache: HashCache,
    config: Config,
    theme: Theme,
    bypass_cache: bool,
}

#[derive(Debug, Clone)]
pub struct BuildFile {
    path: PathBuf,
    virtual_path: String,
    contents: String,
}

impl BuildFile {
    pub fn new(path: PathBuf, virtual_path: impl ToString, contents: impl ToString) -> Self {
        Self {
            path,
            virtual_path: virtual_path.to_string(),
            contents: contents.to_string(),
        }
    }

    pub fn write_to_disk(&self) -> anyhow::Result<()> {
        use anyhow::Context;
        let mut file = std::fs::File::create(&self.path)?;
        file.write_all(self.contents.as_bytes())
            .context(format!("Unable to write file: {:?}", self.path))
    }

    pub fn virtual_path(&self) -> &str {
        &self.virtual_path
    }

    pub fn contents(&self) -> &str {
        &self.contents
    }
}

#[derive(Debug)]
pub struct Bundle {
    files: Vec<BuildFile>,
}

impl Bundle {
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    pub fn add_file(&mut self, file: BuildFile) {
        self.files.push(file)
    }

    pub fn write_to_disk(&self) -> anyhow::Result<()> {
        for file in &self.files {
            file.write_to_disk()?;
        }
        Ok(())
    }

    pub fn iter(&self) -> std::slice::Iter<'_, BuildFile> {
        self.files.iter()
    }
}

impl Engine {
    pub fn new(config: Config, theme: Theme, bypass_cache: bool) -> anyhow::Result<Self> {
        let last_cache = {
            if bypass_cache {
                HashCache::empty()
            } else {
                HashCache::read_from_file()
                    .ok()
                    .unwrap_or_else(HashCache::empty)
            }
        };
        let current_cache = {
            let mut cache = HashCache::empty();
            if !bypass_cache {
                cache.mix_config(&config)?;
            }
            cache
        };
        Ok(Self {
            config,
            theme,
            last_cache,
            current_cache,
            bypass_cache,
        })
    }

    pub fn build(&mut self) -> anyhow::Result<Bundle> {
        // Build directory structure
        let dirs = Self::create_output_directories()?;
        // Collect posts
        let posts = self.gather_posts()?;
        // Generate difference between last and current build
        let diff = self.last_cache.diff(&self.current_cache);
        if !self.bypass_cache {
            // Synchronize changes (actually delete removed files, etc.)
            diff.sync()?;
            // Save cache if different
            if diff.any_changed() {
                self.current_cache.save_to_file()?;
            }
        }
        // Populate output map
        let mut output_map = HashMap::<String, Post>::new();
        for post in &posts {
            output_map.insert(post.get_final_file_name(), post.clone());
        }
        // Collect post that actually have to be rendered
        let posts = {
            let paths = {
                if self.bypass_cache {
                    posts.iter().map(|post| post.filename.clone()).collect()
                } else {
                    diff.changed_post_paths()
                }
            };
            posts
                .into_iter()
                .filter(|post| {
                    let file_path = dirs.build_post_dir.join(post.get_final_file_name());
                    paths.contains(&post.filename) || !file_path.exists()
                })
                .collect::<Vec<_>>()
        };
        // Generate bundle
        let mut bundle = Bundle::new();
        // Generate posts
        for post in posts.into_iter() {
            let file_name = post.get_final_file_name();
            let file_path = dirs.build_post_dir.join(&file_name);
            let virtual_path = format!("/posts/{}", file_name);
            let data = RenderData::for_post(&self.config, &post)?;
            let post_page = self.theme.render_post(data)?;
            bundle.add_file(BuildFile::new(file_path, virtual_path, post_page));
        }
        // Generate index.html
        let index_file_path = dirs.build_dir.join("index.html");
        if self.bypass_cache || (diff.should_rerender_index_page() || !index_file_path.exists()) {
            let data = RenderData::for_index(&self.config, &output_map);
            let index_page = self.theme.render_index(data)?;
            bundle.add_file(BuildFile::new(index_file_path, "/", index_page));
        }
        // Create style.css
        {
            let file_path = dirs.build_dir.join("style.css");
            bundle.add_file(BuildFile::new(file_path, "/style.css", &self.theme.css));
        }
        Ok(bundle)
    }

    fn create_output_directories() -> anyhow::Result<DirectoryStructure> {
        let output_dir = std::env::current_dir()?.join("build");
        let posts_dir = output_dir.join("posts");
        std::fs::create_dir_all(&output_dir)?;
        std::fs::create_dir_all(&posts_dir)?;
        Ok(DirectoryStructure {
            build_dir: output_dir,
            build_post_dir: posts_dir,
        })
    }

    fn gather_posts(&mut self) -> anyhow::Result<Vec<Post>> {
        let posts_dir = std::env::current_dir()?.join("posts");
        let mut posts = Vec::<Post>::new();
        for dir_entry in std::fs::read_dir(posts_dir)? {
            let dir_entry = dir_entry?;
            let path = dir_entry.path();
            if let Some(Some(extension)) = path.extension().map(|s| s.to_str()) {
                let format = PostFormat::from_file_extension(extension)?;
                let contents = std::fs::read_to_string(&path)?;
                let metadata = format.extract_metadata(&contents)?;
                let post = Post {
                    format,
                    metadata,
                    contents,
                    filename: path,
                };
                self.current_cache.mix_post(&post);
                posts.push(post);
            }
        }
        Ok(posts)
    }
}
