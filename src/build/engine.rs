use std::{collections::HashMap, fs::File, io::Write, path::PathBuf};

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

#[derive(Debug)]
pub struct Post {
    pub format: PostFormat,
    pub filename: PathBuf,
    pub metadata: PostMetadata,
    pub contents: String,
}

#[derive(Debug)]
pub struct Engine {
    last_cache: HashCache,
    current_cache: HashCache,
    config: Config,
    theme: Theme,
}

impl Engine {
    pub fn new(config: Config, theme: Theme) -> anyhow::Result<Self> {
        let last_cache = HashCache::read_from_file()
            .ok()
            .unwrap_or_else(|| HashCache::empty());
        let current_cache = {
            let mut cache = HashCache::empty();
            cache.mix_config(&config)?;
            cache
        };
        Ok(Self {
            config,
            theme,
            last_cache,
            current_cache,
        })
    }

    pub fn build(&mut self) -> anyhow::Result<()> {
        // Build directory structure
        let dirs = Self::create_output_directories()?;
        // Collect posts
        let posts = self.gather_posts()?;
        // Generate difference between last and current build
        let diff = self.last_cache.diff(&self.current_cache);
        // Synchronize changes (actually delete removed files, etc.)
        diff.sync()?;
        // Save cache
        self.current_cache.save_to_file()?;
        // Collect post that actually have to be rendered
        let posts = {
            let paths = diff.changed_post_paths();
            posts
                .into_iter()
                .filter(|post| paths.contains(&post.filename))
                .collect::<Vec<_>>()
        };
        let mut output_map = HashMap::<String, Post>::new();
        // Generate posts
        for post in posts.into_iter() {
            let file_name = format!(
                "{}.html",
                post.metadata.title.replace(" ", "_").to_ascii_lowercase()
            );
            let file_path = dirs.build_post_dir.join(&file_name);
            let data = RenderData::for_post(&self.config, &post)?;
            let post_page = self.theme.render_post(data)?;
            let mut file = File::create(file_path)?;
            file.write(post_page.as_bytes())?;
            output_map.insert(file_name, post);
        }
        // Generate index.html
        if diff.should_rerender_index_page() {
            let data = RenderData::for_index(&self.config, &output_map);
            let index_page = self.theme.render_index(data)?;
            let file_path = dirs.build_dir.join("index.html");
            let mut file = File::create(file_path)?;
            file.write(index_page.as_bytes())?;
        }
        // Create style.css
        {
            let file_path = dirs.build_dir.join("style.css");
            let mut file = File::create(file_path)?;
            file.write(self.theme.css.as_bytes())?;
        }
        Ok(())
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
