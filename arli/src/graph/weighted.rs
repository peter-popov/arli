use std::fmt::Debug;
use std::ops::Add;
use crate::graph::{GraphBase, GraphData};

/// Trait representing an edge weight(cost) in weighted graph
pub trait Weight<T = Self>: Default + Add<Output = T> + Ord + Copy + Debug {}
impl<T> Weight for T where T: Default + Add<Output = T> + Ord + Copy + Debug {}

/// Weighted graph
/// 
/// arli only uses graph nodes(we don't define an edge explicitly). The weight is 
/// defined as a cost of traversing from one node to another. 
pub trait Weighted: GraphBase {
  type Weight: Weight;
  fn transition_weight(&self, from: Self::NodeId, to: Self::NodeId) -> Self::Weight;
}

/// The tuple of a graph plus cost function can implement a weighted graph
impl<G: GraphData, W: Weight, C: Fn(&G::Data, &G::Data) -> W> Weighted for (G, C)
{
  type Weight = W;
  fn transition_weight(&self, from: Self::NodeId, to: Self::NodeId) -> Self::Weight {
    (self.1)(self.0.data(from), self.0.data(to))
  }
}

impl<'a, G: Weighted> Weighted for &'a G
{
  type Weight = G::Weight;
  fn transition_weight(&self, from: Self::NodeId, to: Self::NodeId) -> Self::Weight {
    (*self).transition_weight(from, to)
  }
}