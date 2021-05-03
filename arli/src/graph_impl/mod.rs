//! Graph data structures.
//! 
//! The module defined two types of in-memory graph data structures:
//! - [`DynamicGraph`] allows growing a graph by adding nodes or edges. It's recommended for smaller graphs and testing, since memory layout is not optimal.
//! - [`CompactGraph`] is a static graph which cannot be modified after creation. But it can store big graphs in a memory efficient way.

mod dynamic_graph;
mod dynamic_spatial_graph;
mod compact_graph;
mod compact_spatial_graph;
mod common;

pub use dynamic_graph::*;
pub use dynamic_spatial_graph::*;
pub use compact_graph::*;
pub use compact_spatial_graph::*;
pub use common::*;
