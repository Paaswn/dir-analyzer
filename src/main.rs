mod cli;
mod commands;
mod utils;

fn main() -> Result<(), std::io::Error> {
    cli::parsing()
}
