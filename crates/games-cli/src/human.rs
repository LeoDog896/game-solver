use std::{
    fmt::Display, future::IntoFuture, sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Arc,
    }, time::Duration
};

use anyhow::Result;
use tokio::select;
use tokio_util::sync::CancellationToken;
use core::hash::Hash;
use game_solver::{
    game::Game,
    par_move_scores,
    player::TwoPlayer,
    stats::{Stats, TerminalEnds},
};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Alignment, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{
        block::{Position, Title},
        Block, Paragraph, Widget,
    },
    DefaultTerminal, Frame,
};
use std::fmt::Debug;

use super::report::{scores::show_scores, stats::show_stats};

#[derive(Debug)]
struct App<G: Game> {
    exit: CancellationToken,
    stats: Arc<Stats<G::Player>>,
}

impl<G: Game> App<G> {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit.is_cancelled() {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                // it's important to check that the event is a key press event as
                // crossterm also emits key release and repeat events on Windows.
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event)
                }
                _ => {}
            };
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if let KeyCode::Char('q') = key_event.code {
            self.exit()
        }
    }

    fn exit(&mut self) {
        self.exit.cancel();
    }
}

impl<G: Game> Widget for &App<G> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Title::from(" game-solver ".bold().green());
        let instructions = Title::from(Line::from(vec![" Quit ".into(), "<Q> ".blue().bold()]));
        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .border_set(border::THICK);

        let cache_text = Text::from(vec![
            Line::from(vec![
                "States Explored: ".into(),
                self.stats
                    .states_explored
                    .load(Ordering::Relaxed)
                    .to_string()
                    .yellow(),
            ]),
            Line::from(vec![
                "Cache Hits: ".into(),
                self.stats
                    .cache_hits
                    .load(Ordering::Relaxed)
                    .to_string()
                    .yellow(),
            ]),
            Line::from(vec![
                "Pruning Cutoffs: ".into(),
                self.stats
                    .pruning_cutoffs
                    .load(Ordering::Relaxed)
                    .to_string()
                    .yellow(),
            ]),
            Line::from(vec![
                "Terminal Nodes: (winning: ".into(),
                self.stats
                    .terminal_ends
                    .winning
                    .load(Ordering::Relaxed)
                    .to_string()
                    .yellow(),
                ", tie: ".into(),
                self.stats
                    .terminal_ends
                    .tie
                    .load(Ordering::Relaxed)
                    .to_string()
                    .yellow(),
                ", losing: ".into(),
                self.stats
                    .terminal_ends
                    .losing
                    .load(Ordering::Relaxed)
                    .to_string()
                    .yellow(),
                ")".into(),
            ]),
            // TODO: depth
            // Line::from(vec![
            //     "Max Depth: ".into(),
            //     self.stats
            //         .max_depth
            //         .load(Ordering::Relaxed)
            //         .to_string()
            //         .yellow(),
            // ])
        ]);

        Paragraph::new(cache_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

pub async fn human_output<
    T: Game<Player = impl TwoPlayer + Debug + Sync + Send + 'static>
        + Eq
        + Hash
        + Sync
        + Send
        + Display
        + Debug
        + 'static,
>(
    game: T,
) -> Result<()>
where
    T::Move: Sync + Send + Display,
    T::MoveError: Sync + Send + Debug,
{
    let mut terminal = ratatui::init();

    let stats = Arc::new(Stats {
        states_explored: AtomicU64::new(0),
        max_depth: AtomicUsize::new(0),
        cache_hits: AtomicU64::new(0),
        pruning_cutoffs: AtomicU64::new(0),
        terminal_ends: TerminalEnds::default(),
        original_player: game.player(),
        original_move_count: game.move_count(),
    });

    let exit = CancellationToken::new();

    let mut app: App<T> = App {
        stats: stats.clone(),
        exit: exit.clone(),
    };

    let internal_game = game.clone();
    let internal_stats = stats.clone();

    let game_thread = tokio::spawn(async move {
        let game_solving_thread = tokio::spawn(async move {
            par_move_scores(
                &internal_game,
                Some(internal_stats.as_ref()),
            )
        }).into_future();

        select! {
            score = game_solving_thread => {
                exit.cancel();
                Some(score)
            },
            _ = exit.cancelled() => None
        }
    });

    app.run(&mut terminal)?;
    ratatui::restore();
    let move_scores = game_thread.await?;

    show_stats::<T>(&stats);
    match move_scores {
        Some(move_scores) => show_scores(&game, move_scores?),
        None => eprintln!("Game solving was cancelled!"),
    }

    Ok(())
}
