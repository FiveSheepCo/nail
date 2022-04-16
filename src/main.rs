use build::Engine;
use clap::{Parser, Subcommand};

mod build;
mod cache;
mod config;
mod post_format;
mod post_metadata;
mod scaffold;
mod theme;

use config::Config;
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
    Dev,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
            todo!()
        }
        Command::Build { theme } => {
            let config = Config::load()?;
            let theme = Theme::load(theme)?;
            let mut engine = Engine::new(config, theme);
            engine.build()?;
        }
    }
    Ok(())
}
