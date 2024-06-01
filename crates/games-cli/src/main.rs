use clap::{Parser, Subcommand};
use games::{
    chomp::cli::{main as chomp_main, ChompArgs},
    domineering::cli::{main as domineering_main, DomineeringArgs},
    nim::cli::{main as nim_main, NimArgs},
    order_and_chaos::cli::{main as order_and_chaos_main, OrderAndChaosArgs},
    reversi::cli::{main as reversi_main, ReversiArgs},
    tic_tac_toe::cli::{main as tic_tac_toe_main, TicTacToeArgs},
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Reversi(ReversiArgs),
    TicTacToe(TicTacToeArgs),
    OrderAndChaos(OrderAndChaosArgs),
    Nim(NimArgs),
    Domineering(DomineeringArgs),
    Chomp(ChompArgs),
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Reversi(args) => reversi_main(args),
        Commands::TicTacToe(args) => tic_tac_toe_main(args),
        Commands::OrderAndChaos(args) => order_and_chaos_main(args),
        Commands::Nim(args) => nim_main(args),
        Commands::Domineering(args) => domineering_main(args),
        Commands::Chomp(args) => chomp_main(args),
    }
}
