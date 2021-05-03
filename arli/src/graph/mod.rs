//! Set of traits for defining a graph abstraction.
//! 
//! The module contains traits for defining directed, edge-based and weighted graph.
//! 
mod graph;
mod spatial;
mod weighted;

pub use graph::*;
pub use spatial::*;
pub use weighted::*;
