use std::{io::Cursor, str::FromStr, sync::Arc};

use notify::Watcher;
use parking_lot::RwLock;
use tiny_http::{Response, Server};

use crate::{
    build::{Bundle, Engine},
    config::Config,
};

#[derive(Debug)]
pub struct DevServer;

impl DevServer {
    pub fn new() -> Self {
        Self
    }

    pub fn serve(&self) -> anyhow::Result<()> {
        let latest_bundle = {
            let initial_bundle = Self::build_bundle()?;
            Arc::new(RwLock::new(initial_bundle))
        };

        // Rebuild bundle on file change
        let watcher_bundle = latest_bundle.clone();
        let mut watcher =
            notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
                if res.is_err() {
                    return;
                }
                if let Ok(bundle) = Self::build_bundle() {
                    *watcher_bundle.write() = bundle;
                }
            })?;
        watcher.watch(&std::env::current_dir()?, notify::RecursiveMode::Recursive)?;

        // Start local development server
        let server = Server::http("127.0.0.1:8080").unwrap();
        println!("Development server running on http://127.0.0.1:8080.");

        // Handle incoming requests
        for request in server.incoming_requests() {
            let url = Self::map_url(request.url());
            let bundle = latest_bundle.read();
            let file = bundle.iter().find(|&file| file.virtual_path() == url);
            if let Some(file) = file {
                println!("| 200 {}", request.url());
                request.respond(Self::make_response(
                    file.contents(),
                    Self::mime_from_url(file.virtual_path()),
                    200,
                ))?;
            } else {
                println!("| 404 {}", request.url());
                let error_page = format!(
                    "<div>Nail Development Server</div><b>File not found: <span><code>{}</code></span></b>",
                    request.url()
                );
                request.respond(Self::make_response(error_page, "text/html", 404))?;
            }
        }
        Ok(())
    }

    fn load_config() -> anyhow::Result<Config> {
        let mut config = Config::load()?;
        config.__is_dev_mode = true;
        Ok(config)
    }

    fn build_bundle() -> anyhow::Result<Bundle> {
        let config = Self::load_config()?;
        let theme = config.load_theme()?;
        let mut engine = Engine::new(config, theme, true)?;
        engine.build()
    }

    fn make_response(
        content: impl AsRef<str>,
        content_type: impl AsRef<str>,
        status: u32,
    ) -> Response<Cursor<Vec<u8>>> {
        use tiny_http::Header;
        Response::from_string(content.as_ref())
            .with_header(
                Header::from_str(&format!(
                    "Content-Type: {}; charset=utf-8",
                    content_type.as_ref()
                ))
                .unwrap(),
            )
            .with_status_code(status)
    }

    fn mime_from_url(url: impl AsRef<str>) -> &'static str {
        match url.as_ref() {
            "/" => "text/html",
            s if [".html", ".htm"].iter().any(|ext| s.ends_with(ext)) => "text/html",
            s if [".css"].iter().any(|ext| s.ends_with(ext)) => "text/css",
            s if [".js", ".mjs"].iter().any(|ext| s.ends_with(ext)) => "text/javascript",
            _ => "text/plain",
        }
    }

    fn map_url(url: impl AsRef<str>) -> String {
        match url.as_ref() {
            "/index.html" => "/",
            url => url,
        }
        .into()
    }
}
