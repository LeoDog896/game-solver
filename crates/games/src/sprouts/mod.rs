#![doc = include_str!("./README.md")]

use game_solver::game::Game;
use petgraph::matrix_graph::MatrixGraph;

#[derive(Clone)]
pub struct Sprouts(MatrixGraph<(), ()>);

// impl Game for Sprouts {

// }
