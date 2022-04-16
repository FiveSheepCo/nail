use super::PostFormat;

#[derive(Debug)]
pub struct PostMetadata {
    pub title: String,
    pub published: bool,
    pub published_at: String,
}

impl PostMetadata {
    pub fn format(&self, format: &PostFormat) -> String {
        let mut buf = String::new();
        buf.push_str(&format.make_kvp("title", &self.title));
        buf.push_str(&format.make_kvp("published", if self.published { "true" } else { "false" }));
        buf.push_str(&format.make_kvp("published_at", &self.published_at));
        buf.push_str("\n\n");
        buf
    }
}
