use std::collections::HashMap;

use serde::Serialize;

use crate::{build::Post, config::Config};

#[derive(Serialize)]
pub struct BlogRenderData {
    name: String,
}

#[derive(Serialize)]
pub struct PostRenderData {
    title: String,
    content: String,
}

#[derive(Serialize)]
pub struct PageRenderData {
    content: String,
}

#[derive(Serialize)]
struct PostIndex {
    title: String,
    link: String,
}

#[derive(Serialize)]
pub struct HomeRenderData {
    posts: Vec<PostIndex>,
}

#[derive(Serialize)]
pub struct RenderData {
    blog: Option<BlogRenderData>,
    post: Option<PostRenderData>,
    page: Option<PageRenderData>,
    home: Option<HomeRenderData>,
}

impl RenderData {
    pub fn for_post(config: &Config, post: &Post) -> anyhow::Result<Self> {
        let post = Some(PostRenderData {
            title: post.metadata.title.clone(),
            content: post.format.to_html(&post.contents)?,
        });
        let blog = Some(BlogRenderData {
            name: config.name.clone(),
        });
        let page = None;
        let home = None;
        Ok(Self {
            post,
            blog,
            page,
            home,
        })
    }

    pub fn for_index(config: &Config, output_map: &HashMap<String, Post>) -> Self {
        let post = None;
        let blog = Some(BlogRenderData {
            name: config.name.clone(),
        });
        let page = None;
        let home = Some(Self::build_home_data(&output_map));
        Self {
            post,
            blog,
            page,
            home,
        }
    }

    pub fn extend_with_page(&mut self, content: String) {
        self.page = Some(PageRenderData { content });
    }

    fn build_home_data(map: &HashMap<String, Post>) -> HomeRenderData {
        let mut post_index_data = map
            .iter()
            .filter(|(_, post)| post.metadata.published)
            .map(|(key, post)| (key, post))
            .collect::<Vec<_>>();
        post_index_data.sort_by(|a, b| {
            let a: chrono::DateTime<chrono::Utc> = a.1.metadata.published_at.parse().unwrap();
            let b: chrono::DateTime<chrono::Utc> = b.1.metadata.published_at.parse().unwrap();
            b.cmp(&a)
        });
        let post_index_data = post_index_data
            .into_iter()
            .map(|(key, post)| PostIndex {
                title: post.metadata.title.clone(),
                link: format!("/posts/{}", key),
            })
            .collect::<Vec<_>>();
        HomeRenderData {
            posts: post_index_data,
        }
    }
}

#[derive(Debug)]
pub struct Theme {
    pub css: String,
    base_template: String,
    home_template: String,
    post_template: String,
}

impl Theme {
    pub fn render_index(&self, mut data: RenderData) -> anyhow::Result<String> {
        use handlebars::Handlebars;
        let renderer = Handlebars::new();
        let output = renderer.render_template(&self.home_template, &data)?;
        data.extend_with_page(output);
        let output = renderer.render_template(&self.base_template, &data)?;
        Ok(output)
    }

    pub fn render_post(&self, mut data: RenderData) -> anyhow::Result<String> {
        use handlebars::Handlebars;
        let renderer = Handlebars::new();
        let output = renderer.render_template(&self.post_template, &data)?;
        data.extend_with_page(output);
        let output = renderer.render_template(&self.base_template, &data)?;
        Ok(output)
    }

    pub fn load<S>(name: S) -> anyhow::Result<Self>
    where
        S: AsRef<str>,
    {
        let theme_dir = std::env::current_dir()?.join("themes").join(name.as_ref());
        use std::fs::read_to_string;
        let name = name.as_ref();
        let css_path = theme_dir.join(format!("{}.css", name));
        let base_path = theme_dir.join(format!("{}.base.html", name));
        let home_path = theme_dir.join(format!("{}.home.html", name));
        let post_path = theme_dir.join(format!("{}.post.html", name));
        let css = read_to_string(css_path)?;
        let base_template = read_to_string(base_path)?;
        let home_template = read_to_string(home_path)?;
        let post_template = read_to_string(post_path)?;
        Ok(Self {
            css,
            base_template,
            home_template,
            post_template,
        })
    }
}
