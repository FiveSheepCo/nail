use std::{collections::HashMap, fs::File, io::Write, path::PathBuf};

use crate::{
    config::Config,
    post_format::PostFormat,
    post_metadata::PostMetadata,
    theme::{RenderData, Theme},
};

#[derive(Debug)]
pub struct Post {
    pub format: PostFormat,
    pub metadata: PostMetadata,
    pub contents: String,
}

#[derive(Debug)]
pub struct Engine {
    config: Config,
    theme: Theme,
}

impl Engine {
    pub fn new(config: Config, theme: Theme) -> Self {
        Self { config, theme }
    }

    pub fn build(&mut self) -> anyhow::Result<()> {
        let posts = Self::gather_posts()?;
        let mut output_map = HashMap::<String, Post>::new();
        let output_dir = Self::create_output_directories()?;
        let posts_dir = output_dir.join("posts");
        // Generate posts
        for mut post in posts.into_iter() {
            let html = post.format.to_html(&post.contents)?;
            post.contents = html;
            let file_name = format!(
                "{}.html",
                post.metadata.title.replace(" ", "_").to_ascii_lowercase()
            );
            let file_path = posts_dir.join(&file_name);
            let data = RenderData::for_post(&self.config, &post);
            let post_page = self.theme.render_post(data)?;
            let mut file = File::create(file_path)?;
            file.write(post_page.as_bytes())?;
            output_map.insert(file_name, post);
        }
        // Generate index.html
        {
            let data = RenderData::for_index(&self.config, &output_map);
            let index_page = self.theme.render_index(data)?;
            let file_path = output_dir.join("index.html");
            let mut file = File::create(file_path)?;
            file.write(index_page.as_bytes())?;
        }
        // Create style.css
        {
            let file_path = output_dir.join("style.css");
            let mut file = File::create(file_path)?;
            file.write(self.theme.css.as_bytes())?;
        }
        Ok(())
    }

    fn create_output_directories() -> anyhow::Result<PathBuf> {
        let output_dir = std::env::current_dir()?.join("build");
        let posts_dir = output_dir.join("posts");
        std::fs::create_dir_all(&output_dir)?;
        std::fs::create_dir_all(&posts_dir)?;
        Ok(output_dir)
    }

    fn gather_posts() -> anyhow::Result<Vec<Post>> {
        let posts_dir = std::env::current_dir()?.join("posts");
        let mut posts = Vec::<Post>::new();
        for dir_entry in std::fs::read_dir(posts_dir)? {
            let dir_entry = dir_entry?;
            let path = dir_entry.path();
            if let Some(Some(extension)) = path.extension().map(|s| s.to_str()) {
                let format = PostFormat::from_file_extension(extension)?;
                let contents = std::fs::read_to_string(path)?;
                let metadata = format.extract_metadata(&contents)?;
                posts.push(Post {
                    format,
                    metadata,
                    contents,
                });
            }
        }
        Ok(posts)
    }
}
