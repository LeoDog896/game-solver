#![doc = include_str!("./README.md")]

use petgraph::matrix_graph::MatrixGraph;

#[derive(Clone)]
pub struct Sprouts(MatrixGraph<(), ()>);
