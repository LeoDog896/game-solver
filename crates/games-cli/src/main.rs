use anyhow::Result;
use clap::Parser;
use games::{
    chomp::Chomp, domineering::Domineering, nim::Nim, order_and_chaos::OrderAndChaos,
    reversi::Reversi, tic_tac_toe::TicTacToe, util::cli::play, Games,
};

/// `game-solver` is a solving utility that helps analyze various combinatorial games.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Games,
    #[arg(short, long)]
    plain: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Games::Reversi(args) => play::<Reversi>(args.try_into().unwrap(), cli.plain),
        Games::TicTacToe(args) => play::<TicTacToe>(args.try_into().unwrap(), cli.plain),
        Games::OrderAndChaos(args) => {
            play::<OrderAndChaos<6, 6, 5, 6>>(args.try_into().unwrap(), cli.plain)
        }
        Games::Nim(args) => play::<Nim>(args.try_into().unwrap(), cli.plain),
        Games::Domineering(args) => play::<Domineering<5, 5>>(args.try_into().unwrap(), cli.plain),
        Games::Chomp(args) => play::<Chomp>(args.try_into().unwrap(), cli.plain),
    };

    Ok(())
}
