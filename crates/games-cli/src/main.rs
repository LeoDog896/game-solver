use anyhow::Result;
use clap::Parser;
use games::{
    chomp::Chomp, domineering::Domineering, nim::Nim, order_and_chaos::OrderAndChaos,
    reversi::Reversi, tic_tac_toe::TicTacToe, util::cli::play, Games,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Games,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Games::Reversi(args) => play::<Reversi>(args.try_into().unwrap()),
        Games::TicTacToe(args) => play::<TicTacToe>(args.try_into().unwrap()),
        Games::OrderAndChaos(args) => play::<OrderAndChaos>(args.try_into().unwrap()),
        Games::Nim(args) => play::<Nim>(args.try_into().unwrap()),
        Games::Domineering(args) => play::<Domineering<5, 5>>(args.try_into().unwrap()),
        Games::Chomp(args) => play::<Chomp>(args.try_into().unwrap()),
    };

    Ok(())
}
