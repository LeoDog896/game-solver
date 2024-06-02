use clap::Parser;
use games::{
    chomp::cli::main as chomp_main, domineering::cli::main as domineering_main,
    nim::cli::main as nim_main, order_and_chaos::cli::main as order_and_chaos_main,
    reversi::cli::main as reversi_main, tic_tac_toe::cli::main as tic_tac_toe_main, Games,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Games,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Games::Reversi(args) => reversi_main(args),
        Games::TicTacToe(args) => tic_tac_toe_main(args),
        Games::OrderAndChaos(args) => order_and_chaos_main(args),
        Games::Nim(args) => nim_main(args),
        Games::Domineering(args) => domineering_main(args),
        Games::Chomp(args) => chomp_main(args),
    }
}
