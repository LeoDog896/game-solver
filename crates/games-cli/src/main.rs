use clap::{Parser, Subcommand};
use games::{reversi::cli::main as reversi_main, reversi::cli::ReversiArgs};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands
}

#[derive(Subcommand)]
enum Commands {
    Reversi(ReversiArgs)
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Reversi(args) => reversi_main(args)
    }
}
