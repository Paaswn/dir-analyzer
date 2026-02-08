use crate::commands::loc;
use crate::commands::size;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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
        Commands::Scan { path } => {
            if let Some(path) = path {
                println!("{:?}", path);
            }
        }
        Commands::Size { path, top } => {
            if let Some(path) = path {
                size::print_sizes(&path, *top)?;
            } else {
                size::print_sizes(&PathBuf::from("."), *top)?;
            }
        }
        Commands::Loc { path, top } => {
            if let Some(path) = path {
                loc::print_loc(path, *top)?;
            } else {
                loc::print_loc(&PathBuf::from("."), *top)?;
            }
        }
    }
    Ok(())
}
