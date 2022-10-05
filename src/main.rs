use clap::{Parser, Subcommand};

mod build;
mod cache;
mod config;
mod dev_server;
mod post_format;
mod post_metadata;
mod scaffold;
mod theme;

use build::Engine;
use config::Config;
use dev_server::DevServer;
use post_format::PostFormat;
use scaffold::Scaffold;

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
    Dev,
    #[clap(about = "Build and bundle the blog")]
    Build,
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
        Command::Dev => {
            let server = DevServer::new();
            server.serve()?;
        }
        Command::Build => {
            let config = Config::load()?;
            let theme = config.load_theme()?;
            let bundle = {
                let mut engine = Engine::new(config, theme, false)?;
                engine.build()?
            };
            bundle.write_to_disk()?;
        }
    }
    Ok(())
}
