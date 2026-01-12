use crate::commands::loc;
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
        #[arg(short, long)]
        top: Option<usize>,
    },
    Loc {
        #[arg(short, long)]
        path: Option<PathBuf>,
        #[arg(short, long)]
        top: Option<usize>,
    },
}

pub fn parsing() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match &cli.command {
        Some(command) => match command {
            Commands::Scan { path } => {
                if let Some(path) = path {
                    println!("{:?}", path);
                }
            }
            Commands::Size { path, top } => {
                if let Some(path) = path {
                    size::get_top_sizes(&path, *top)?;
                } else {
                    size::get_top_sizes(&PathBuf::from("."), *top)?;
                }
            }
            Commands::Loc { path, top } => {
                if let Some(path) = path {
                    loc::print_loc(&path, *top)?;
                }
            }
        },
        None => {
            eprintln!("Error: No command provided")
        }
    }
    Ok(())
}
