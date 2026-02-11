use crate::commands::loc::{self, Extension};
use crate::commands::size;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "dira")]
#[command(version = "0.2.1")]
#[command(about = "A CLI tool to analyze directories", long_about = None)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// A basic scan command (currently a placeholder)
    Scan {
        /// The path to the directory to scan. Defaults to the current directory.
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    /// Calculates the size of files and subdirectories in a given path
    Size {
        /// Path to the directory to analyze. Defaults to the current directory
        #[arg(short, long)]
        path: Option<PathBuf>,
        /// The number of largest items to display
        #[arg(short, long, default_value_t = 100)]
        top: usize,
    },
    /// Count lines of code (LOC) for files within a project directory. Note that this command is considerably slower than `dira size` and is best suited for project-specific analysis.
    Loc {
        /// Path to the project directory. Defaults to the current directory
        #[arg(short, long)]
        path: Option<PathBuf>,
        /// The number of files with the most lines to display
        #[arg(short, long, default_value_t = 100)]
        top: usize,
        /// A comma-separated list of file extensions to exclusively scan, ignoring others
        #[arg(long, value_delimiter = ',')]
        only: Option<Vec<String>>,
        /// A comma-separated list of additional file extensions to ignore
        #[arg(long, value_delimiter = ',', conflicts_with = "only")]
        ignore: Option<Vec<String>>,
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
        Commands::Loc {
            path,
            top,
            only,
            ignore,
        } => {
            let path = if let Some(path) = path {
                path
            } else {
                &PathBuf::from(".")
            };
            if let Some(only) = only {
                loc::print_loc(path, *top, Extension::Only(only.clone()))?;
            } else if let Some(ignore) = ignore {
                loc::print_loc(path, *top, Extension::Ignore(ignore.clone()))?;
            } else {
                loc::print_loc(path, *top, Extension::Default)?;
            }
        }
    }
    Ok(())
}
