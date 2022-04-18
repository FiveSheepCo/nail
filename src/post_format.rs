use anyhow::{bail, Context};
use std::str::FromStr;

use crate::post_metadata::PostMetadata;

#[derive(Debug, Clone)]
pub enum PostFormat {
    Markdown,
    Html,
}

impl FromStr for PostFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "md" | "markdown" => Ok(PostFormat::Markdown),
            "html" => Ok(PostFormat::Html),
            _ => Err(anyhow::anyhow!("Invalid post format: `{}`", s)),
        }
    }
}

impl PostFormat {
    #[inline]
    pub fn make_kvp<K, V>(&self, key: K, value: V) -> String
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        match self {
            PostFormat::Markdown => format!("[//]: # ({}: {})\n", key.as_ref(), value.as_ref()),
            PostFormat::Html => format!("<!-- {}: {} -->", key.as_ref(), value.as_ref()),
        }
    }

    pub fn extract_kvp<S>(&self, line: S) -> anyhow::Result<(String, String)>
    where
        S: AsRef<str>,
    {
        let line = line.as_ref();
        let err_str = "Unable to parse post metadata.";
        match self {
            PostFormat::Markdown => {
                if !line.starts_with("[//]: # (") {
                    bail!(err_str)
                }
                let meta_part = line
                    .split_once('(')
                    .context(err_str)?
                    .1
                    .trim_end()
                    .chars()
                    .rev()
                    .skip(1)
                    .collect::<Vec<_>>()
                    .iter()
                    .rev()
                    .collect::<String>();
                let (key, value) = meta_part.split_once(':').context(err_str)?;
                let key = key.trim();
                let value = value.trim();
                Ok((key.to_string(), value.to_string()))
            }
            PostFormat::Html => {
                if !line.starts_with("<!-- !") {
                    bail!(err_str)
                }
                let meta_part = line
                    .split_once("<!-- !")
                    .context(err_str)?
                    .1
                    .trim_end()
                    .chars()
                    .rev()
                    .skip("-->".len())
                    .collect::<Vec<_>>()
                    .iter()
                    .rev()
                    .collect::<String>();
                let (key, value) = meta_part.split_once(':').context(err_str)?;
                let key = key.trim();
                let value = value.trim();
                Ok((key.to_string(), value.to_string()))
            }
        }
    }

    #[inline]
    pub fn to_file_extension(&self) -> String {
        match self {
            Self::Markdown => "md",
            Self::Html => "html",
        }
        .to_string()
    }

    #[inline]
    pub fn from_file_extension<S>(ext: S) -> anyhow::Result<Self>
    where
        S: AsRef<str>,
    {
        match ext.as_ref().to_ascii_lowercase().as_str() {
            "md" => Ok(Self::Markdown),
            "html" => Ok(Self::Html),
            _ => Err(anyhow::anyhow!("Unknown file extension: {}", ext.as_ref())),
        }
    }

    #[inline]
    pub fn as_template<S>(&self, name: S) -> String
    where
        S: AsRef<str>,
    {
        PostMetadata {
            title: name.as_ref().to_string(),
            published: false,
            published_at: chrono::offset::Local::now().to_rfc3339(),
        }
        .format(self)
    }

    pub fn extract_metadata<S>(&self, contents: S) -> anyhow::Result<PostMetadata>
    where
        S: AsRef<str>,
    {
        let mut title: Option<String> = None;
        let mut published: Option<bool> = None;
        let mut published_at: Option<String> = None;
        for line in contents.as_ref().lines() {
            if let Ok((key, value)) = self.extract_kvp(line) {
                match key.as_str() {
                    "title" => title = Some(value),
                    "published" => published = if value == "true" { Some(true) } else { None },
                    "published_at" => published_at = Some(value),
                    _ => (),
                }
            }
        }
        Ok(PostMetadata {
            title: title.unwrap_or_else(|| "N/A".to_string()),
            published: published.unwrap_or(false),
            published_at: published_at.unwrap_or_else(|| "N/A".to_string()),
        })
    }

    pub fn to_html<S>(&self, content: S) -> anyhow::Result<String>
    where
        S: AsRef<str>,
    {
        let content = content.as_ref();
        match self {
            PostFormat::Markdown => {
                use comrak::{format_html, parse_document, Arena, ComrakOptions};
                let arena = Arena::new();
                let options = ComrakOptions::default();
                let ast = parse_document(&arena, content, &options);
                let mut buf = std::io::BufWriter::new(Vec::new());
                format_html(ast, &options, &mut buf)?;
                Ok(String::from_utf8(buf.into_inner()?)?)
            }
            PostFormat::Html => Ok(content.to_string()),
        }
    }
}
