use clap::{Parser, Subcommand};
use parking_lot::RwLock;
use tiny_http::{Response, Server};

use std::{io::Cursor, str::FromStr, sync::Arc, time::SystemTime};

mod build;
mod cache;
mod config;
mod post_format;
mod post_metadata;
mod scaffold;
mod theme;

use build::{Bundle, Engine};
use config::Config;
use notify::Watcher;
use post_format::PostFormat;
use scaffold::Scaffold;
use theme::Theme;

#[derive(Subcommand, Debug)]
enum PostCommand {
    #[clap(about = "Scaffold new post")]
    New {
        name: String,
        #[clap(long = "format")]
        format: Option<PostFormat>,
        #[clap(long = "force")]
        force: bool,
    },
}

#[derive(Subcommand, Debug)]
enum Command {
    #[clap(about = "Scaffold new project")]
    New {
        name: String,
        #[clap(long = "force")]
        force: bool,
    },
    #[clap(about = "Manage posts")]
    Post {
        #[clap(subcommand)]
        command: PostCommand,
    },
    #[clap(about = "Start development server")]
    Dev {
        #[clap(short = 't', long = "theme")]
        theme: String,
    },
    Build {
        #[clap(short = 't', long = "theme")]
        theme: String,
    },
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    match args.command {
        Command::New { name, force } => Scaffold::create_project(name, force)?,
        Command::Post { command } => match command {
            PostCommand::New {
                name,
                format,
                force,
            } => Scaffold::create_post(name, format.unwrap_or(PostFormat::Markdown), force)?,
        },
        Command::Dev { theme } => {
            let theme = Arc::new(RwLock::new(theme));
            let latest_bundle = Arc::new(RwLock::new(Bundle::new()));
            let watcher_bundle = latest_bundle.clone();

            // Initial build
            if let Ok(mut config) = Config::load() {
                config.__is_dev_mode = true;
                if let Ok(theme) = Theme::load(theme.read().clone()) {
                    if let Ok(mut engine) = Engine::new(config, theme, true) {
                        if let Ok(bundle) = engine.build() {
                            *latest_bundle.write() = bundle;
                        }
                    }
                }
            }

            // Rebuild bundle on file change
            let mut watcher = notify::recommended_watcher(move |res| {
                if let Ok(_) = res {
                    if let Ok(mut config) = Config::load() {
                        config.__is_dev_mode = true;
                        if let Ok(theme) = Theme::load(theme.read().clone()) {
                            if let Ok(mut engine) = Engine::new(config, theme, true) {
                                if let Ok(bundle) = engine.build() {
                                    *watcher_bundle.write() = bundle;
                                }
                            }
                        }
                    }
                }
            })?;

            println!(
                "Watching {} for changes.",
                std::env::current_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default()
            );
            watcher.watch(&std::env::current_dir()?, notify::RecursiveMode::Recursive)?;

            // Start local development server
            let server = Server::http("127.0.0.1:8080").unwrap();
            println!("Development server running on http://127.0.0.1:8080.");
            fn wrap(
                response: Response<Cursor<Vec<u8>>>,
                content_type: impl AsRef<str>,
                status: u32,
            ) -> Response<Cursor<Vec<u8>>> {
                use tiny_http::Header;
                response
                    .with_header(
                        Header::from_str(&format!(
                            "Content-Type: {}; charset=utf-8",
                            content_type.as_ref()
                        ))
                        .unwrap(),
                    )
                    .with_status_code(status)
            }
            fn filename_to_mime(filename: impl AsRef<str>) -> &'static str {
                match filename.as_ref() {
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
            for request in server.incoming_requests() {
                let _time = SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|t| t.as_millis())
                    .unwrap();
                let url = map_url(request.url());
                if let Some(file) = latest_bundle
                    .read()
                    .iter()
                    .find(|file| file.virtual_path() == url)
                {
                    println!("| 200 {}", request.url());
                    let mime_type = filename_to_mime(file.virtual_path());
                    request
                        .respond(wrap(Response::from_string(file.contents()), mime_type, 200))
                        .unwrap();
                } else {
                    println!("| 404 {}", request.url());
                    let error_page = format!("<div>Nail Dev Server</div><b>File not found: <span><code>{}</code></span></b>", request.url());
                    request
                        .respond(wrap(Response::from_string(error_page), "text/html", 404))
                        .unwrap();
                }
            }
        }
        Command::Build { theme } => {
            let config = Config::load()?;
            let theme = Theme::load(theme)?;
            let bundle = {
                let mut engine = Engine::new(config, theme, false)?;
                engine.build()?
            };
            bundle.write_to_disk()?;
        }
    }
    Ok(())
}
