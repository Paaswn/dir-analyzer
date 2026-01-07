use crate::commands::size;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Scan {
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    Size {
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
}

pub fn parsing() -> std::io::Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Some(command) => match command {
            Commands::Scan { path } => {
                if let Some(path) = path {
                    println!("{:?}", path);
                }
            }
            Commands::Size { path } => {
                if let Some(path) = path {
                    size::get_top_sizes(&path)?;
                }
            }
        },
        // TODO print top 10 file size
        None => {
            eprintln!("Error: No command provided")
        }
    }
    Ok(())
}
