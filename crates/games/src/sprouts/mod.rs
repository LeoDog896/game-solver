#![doc = include_str!("./README.md")]

use std::{
    fmt::{Debug, Display},
    hash::Hash,
    str::FromStr,
};

use anyhow::Error;
use clap::Args;
use game_solver::{
    game::{Game, Normal, NormalImpartial},
    player::ImpartialPlayer,
};
use itertools::Itertools;
use petgraph::{
    matrix_graph::{MatrixGraph, NodeIndex},
    visit::{IntoEdgeReferences, IntoNodeIdentifiers},
    Undirected,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::util::move_failable;

/// We aren't dealing with large sprout counts for now.
pub type SproutsIx = u8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct SproutsMove {
    from: NodeIndex<SproutsIx>,
    to: NodeIndex<SproutsIx>,
}

impl Display for SproutsMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {})", self.from.index(), self.to.index())
    }
}

type SproutsGraph = MatrixGraph<(), (), Undirected, Option<()>, SproutsIx>;

#[derive(Clone)]
pub struct Sprouts(SproutsGraph);

// SproutsGraph, given that its vertices and edges are unlabelled,
// doesn't implement equality as that requires isomorphism checks.
// since we don't want these operations for reordering to be expensive,
// we simply check for equality as is.

impl Hash for Sprouts {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.node_count().hash(state);
        for edge in self.0.edge_references() {
            edge.hash(state);
        }
    }
}

impl PartialEq for Sprouts {
    fn eq(&self, other: &Self) -> bool {
        self.0.node_count() == other.0.node_count()
            && self.0.edge_references().collect::<Vec<_>>()
                == other.0.edge_references().collect::<Vec<_>>()
    }
}

impl Eq for Sprouts {}

impl Sprouts {
    pub fn new(node_count: SproutsIx) -> Self {
        let mut graph = SproutsGraph::default();

        for _ in 0..node_count {
            graph.add_node(());
        }

        Self(graph)
    }
}

#[derive(Error, Debug, Clone)]
pub enum SproutsMoveError {
    #[error("chosen index {0} from move {1:?} is out of bounds.")]
    MoveOutOfBounds(SproutsIx, SproutsMove),
    #[error("chosen index {0} from move {1:?} references a dead sprout.")]
    DeadSprout(SproutsIx, SproutsMove),
    #[error("a move for {0:?} has already been made")]
    SproutsConnected(SproutsMove),
}

const MAX_SPROUTS: usize = 3;

impl Normal for Sprouts {}
impl NormalImpartial for Sprouts {}
impl Game for Sprouts {
    type Move = SproutsMove;
    type Iter<'a> = std::vec::IntoIter<Self::Move>;

    type Player = ImpartialPlayer;
    type MoveError = SproutsMoveError;

    fn max_moves(&self) -> Option<usize> {
        // TODO: i actually want to find what the proper paper is, but
        // https://en.wikipedia.org/wiki/Sprouts_(game)#Maximum_number_of_moves
        // is where this is from.
        // TODO: use MAX_SPROUTS?
        Some(3 * self.0.node_count() - 1)
    }

    fn move_count(&self) -> usize {
        self.0.edge_count()
    }

    fn make_move(&mut self, m: &Self::Move) -> Result<(), Self::MoveError> {
        // There already exists an edge here!
        if self.0.has_edge(m.from, m.to) {
            return Err(SproutsMoveError::SproutsConnected(*m));
        }

        // move index is out of bounds
        {
            if !self.0.node_identifiers().contains(&m.from) {
                return Err(SproutsMoveError::MoveOutOfBounds(
                    m.from.index().try_into().unwrap(),
                    *m,
                ));
            }

            if !self.0.node_identifiers().contains(&m.to) {
                return Err(SproutsMoveError::MoveOutOfBounds(
                    m.to.index().try_into().unwrap(),
                    *m,
                ));
            }
        }

        // sprouts to use are dead
        {
            if self.0.edges(m.from).count() >= MAX_SPROUTS {
                return Err(SproutsMoveError::DeadSprout(
                    m.from.index().try_into().unwrap(),
                    *m,
                ));
            }

            if self.0.edges(m.to).count() >= MAX_SPROUTS {
                return Err(SproutsMoveError::DeadSprout(
                    m.to.index().try_into().unwrap(),
                    *m,
                ));
            }
        }

        self.0.add_edge(m.from, m.to, ());

        Ok(())
    }

    fn player(&self) -> Self::Player {
        ImpartialPlayer::Next
    }

    fn possible_moves(&self) -> Self::Iter<'_> {
        let mut sprouts_moves = vec![];

        for id in self.0.node_identifiers() {
            let edge_count = self.0.edges(id).count();

            // TODO: use MAX_SPROUTS for all values
            match edge_count {
                0 | 1 => {
                    if !self.0.has_edge(id, id) {
                        sprouts_moves.push(SproutsMove { from: id, to: id });
                    }
                    for sub_id in self.0.node_identifiers() {
                        if id >= sub_id {
                            continue;
                        }
                        if self.0.edges(sub_id).count() >= MAX_SPROUTS {
                            continue;
                        }
                        if self.0.has_edge(id, sub_id) {
                            continue;
                        }
                        sprouts_moves.push(SproutsMove {
                            from: id,
                            to: sub_id,
                        })
                    }
                }
                2 => {
                    for sub_id in self.0.node_identifiers() {
                        if id >= sub_id {
                            continue;
                        }
                        if self.0.edges(sub_id).count() >= MAX_SPROUTS {
                            continue;
                        }
                        if self.0.has_edge(id, sub_id) {
                            continue;
                        }
                        sprouts_moves.push(SproutsMove {
                            from: id,
                            to: sub_id,
                        })
                    }
                }
                MAX_SPROUTS => (),
                _ => panic!("No node should have more than three edges"),
            }
        }

        sprouts_moves.into_iter()
    }

    fn state(&self) -> game_solver::game::GameState<Self::Player> {
        <Self as Normal>::state(&self)
    }
}

impl Debug for Sprouts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let references = self.0.edge_references().collect::<Vec<_>>();

        writeln!(f, "graph of vertices count {}", references.len())?;

        if references.is_empty() {
            return Ok(());
        }

        for (i, j, _) in references {
            write!(f, "{}-{} ", i.index(), j.index())?;
        }
        writeln!(f)?;

        Ok(())
    }
}

impl Display for Sprouts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

/// Analyzes Sprouts.
///
#[doc = include_str!("./README.md")]
#[derive(Args, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct SproutsArgs {
    /// The amount of sprouts (nodes)
    /// to start off with.
    starting_sprouts: SproutsIx,
    /// Sprouts moves, ordered as i1-j1 i2-j2 ...
    #[arg(value_parser = clap::value_parser!(SproutsMove))]
    moves: Vec<SproutsMove>,
}

impl Default for SproutsArgs {
    fn default() -> Self {
        Self {
            starting_sprouts: 6,
            moves: vec![],
        }
    }
}

impl TryFrom<SproutsArgs> for Sprouts {
    type Error = Error;

    fn try_from(args: SproutsArgs) -> Result<Self, Self::Error> {
        let mut game = Sprouts::new(args.starting_sprouts);

        for sprouts_move in args.moves {
            move_failable(&mut game, &sprouts_move)?;
        }

        Ok(game)
    }
}

impl FromStr for SproutsMove {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let components = s.split("-").collect::<Vec<_>>();

        assert_eq!(
            components.len(),
            2,
            "a move shouldn't connect more than two sprouts"
        );

        Ok(SproutsMove {
            from: str::parse::<SproutsIx>(components[0])?.into(),
            to: str::parse::<SproutsIx>(components[1])?.into(),
        })
    }
}
