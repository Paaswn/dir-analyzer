mod cli;
mod commands;

fn main() -> Result<(), std::io::Error> {
    cli::parsing()
}
