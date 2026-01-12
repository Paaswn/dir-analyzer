mod cli;
mod commands;
mod utils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    cli::parsing()
}
