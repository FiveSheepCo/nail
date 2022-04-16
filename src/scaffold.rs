use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;

use anyhow::bail;

use crate::config::Config;
use crate::post_format::PostFormat;

#[derive(Debug)]
pub struct Scaffold;

impl Scaffold {
    pub fn create_post(name: String, format: PostFormat, force: bool) -> anyhow::Result<()> {
        let snake_case_name = name.replace(" ", "_").to_ascii_lowercase();
        let post_path = std::env::current_dir()?.join("posts").join(format!(
            "{}.{}",
            &snake_case_name,
            format.to_file_extension()
        ));

        // Check if post exists
        if !force && post_path.exists() {
            bail!("The post `{}` already exists!", post_path.display())
        }

        // Write post
        {
            let mut file = File::create(post_path)?;
            file.write(format.as_template(&name).as_bytes())?;
        }

        println!("Created post `{}` in `./posts/{}`", &name, &snake_case_name);

        Ok(())
    }

    pub fn create_project(name: String, force: bool) -> anyhow::Result<()> {
        let snake_case_name = name.replace(" ", "_").to_ascii_lowercase();
        let blog_dir = Path::new(&snake_case_name);

        // Check if directory exists
        if !force && blog_dir.exists() {
            bail!("The directory `{}` already exists!", blog_dir.display())
        }

        // Create directory structure
        create_dir_all(&blog_dir)?;
        create_dir_all(blog_dir.join("posts"))?;
        create_dir_all(blog_dir.join("themes"))?;

        // Write default config
        let config = Config::new(&name);
        let config_path = blog_dir.join("config.toml");
        {
            let mut file = File::create(config_path)?;
            config.save_to_file(&mut file)?;
        }

        println!(
            "Created project `{}` in directory `./{}`",
            &name, snake_case_name
        );

        Ok(())
    }
}
