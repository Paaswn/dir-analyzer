use crate::commands::loc::{Extension, LocScanner, PrintLayout, print_loc};
use crate::commands::size;
use clap::{Parser, Subcommand};
use std::cmp::Ordering;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "dira")]
#[command(version)]
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
        #[arg(long)]
        nested: bool,
        #[arg(long, conflicts_with = "nested")]
        one_line: bool,
        #[arg(short, long, conflicts_with = "descending")]
        ascending: bool,
        #[arg(short, long)]
        descending: bool,
    },
}

pub fn parsing() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Scan { path } => {
            if let Some(path) = path {
                println!("{:?}", path);
            }
        }
        Commands::Size { path, top } => {
            if let Some(path) = path {
                size::print_sizes(&path, top)?;
            } else {
                size::print_sizes(&PathBuf::from("."), top)?;
            }
        }
        Commands::Loc {
            path,
            top,
            only,
            ignore,
            nested,
            one_line,
            ascending,
            descending,
        } => {
            let path: &Path = path.as_deref().unwrap_or_else(|| Path::new("."));
            let extension = match (only, ignore) {
                (Some(o), None) => Extension::Only(o),
                (None, Some(i)) => Extension::Ignore(i),
                _ => Extension::Default,
            };
            let order = match (ascending, descending) {
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                _ => Ordering::Greater,
            };
            let layout = if nested && !one_line {
                PrintLayout::Nested
            } else {
                PrintLayout::OneLine
            };
            let scanner = LocScanner {
                extension,
                code_files: vec![],
                name_buffer: String::new(),
                limit: top,
                layout,
                order,
            };
            print_loc(&path, scanner)?;
        }
    }
    Ok(())
}
