use crate::commands::loc::{self, ExclusiveExt};
use crate::commands::size;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "dira")]
#[command(version = "0.2.6")]
#[command(about = "A CLI tool to analyze directories", long_about = None)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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
    match cli.command {
        Commands::Size { path, top } => {
            size::print_sizes(path.as_deref().unwrap_or_else(|| Path::new(".")), top)?;
        }
        Commands::Loc {
            path,
            top,
            only,
            ignore,
        } => {
            let path = path.as_deref().unwrap_or_else(|| Path::new("."));
            if let Some(only) = only {
                loc::print_loc(path, top, ExclusiveExt::Only(only))?;
            } else if let Some(ignore) = ignore {
                loc::print_loc(path, top, ExclusiveExt::Ignore(ignore))?;
            } else {
                loc::print_loc(path, top, ExclusiveExt::None)?;
            }
        }
    }
    Ok(())
}
