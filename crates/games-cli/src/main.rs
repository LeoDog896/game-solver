mod interactive;

use anyhow::Result;
use clap::Parser;
use games::{
    chomp::Chomp, domineering::Domineering, naive_nim::Nim, order_and_chaos::OrderAndChaos,
    reversi::Reversi, sprouts::Sprouts, tic_tac_toe::TicTacToe, zener::Zener,
    Games,
};
use games_cli::play;
use interactive::play_interactive;

/// `game-solver` is a solving utility that helps analyze various combinatorial games.
#[derive(Parser)]
#[command(version, about, long_about = None)]
enum Cli {
    Solve {
        #[command(subcommand)]
        command: Games,
        #[arg(short, long)]
        plain: bool,
    },
    Play {
        #[command(subcommand)]
        command: Games
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli {
        Cli::Solve { command, plain } => match command {
            Games::Reversi(args) => play::<Reversi>(args.try_into().unwrap(), plain).await,
            Games::TicTacToe(args) => play::<TicTacToe>(args.try_into().unwrap(), plain).await,
            Games::OrderAndChaos(args) => play::<OrderAndChaos<6, 6, 5, 6>>(args.try_into().unwrap(), plain).await,
            Games::NaiveNim(args) => play::<Nim>(args.try_into().unwrap(), plain).await,
            Games::Domineering(args) => play::<Domineering<5, 5>>(args.try_into().unwrap(), plain).await,
            Games::Chomp(args) => play::<Chomp>(args.try_into().unwrap(), plain).await,
            Games::Sprouts(args) => play::<Sprouts>(args.try_into().unwrap(), plain).await,
            Games::Zener(args) => play::<Zener>(args.try_into().unwrap(), plain).await,
        },
        Cli::Play { command } => match command {
            Games::Reversi(args) => play_interactive::<Reversi>(args.try_into().unwrap()),
            Games::TicTacToe(args) => play_interactive::<TicTacToe>(args.try_into().unwrap()),
            Games::OrderAndChaos(args) => play_interactive::<OrderAndChaos<6, 6, 5, 6>>(args.try_into().unwrap()),
            Games::NaiveNim(args) => play_interactive::<Nim>(args.try_into().unwrap()),
            Games::Domineering(args) => play_interactive::<Domineering<5, 5>>(args.try_into().unwrap()),
            Games::Chomp(args) => play_interactive::<Chomp>(args.try_into().unwrap()),
            Games::Sprouts(args) => play_interactive::<Sprouts>(args.try_into().unwrap()),
            Games::Zener(args) => play_interactive::<Zener>(args.try_into().unwrap()),
        }
    };

    Ok(())
}
