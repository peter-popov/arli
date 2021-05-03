use std::fmt::Debug;
use std::hash::Hash;
use std::iter::Iterator;

// ====== Basic traits =====

/// Requirements for a node identifier
pub trait Identifier: Eq + Hash + Copy + Debug {}
impl<T> Identifier for T where T: Eq + Hash + Copy + Debug {}

/// Defines type of the identifier for the graph
pub trait GraphBase {
  type NodeId: Identifier;
}

/// Defines data associated with each node. The data can be used in order to calculate the weight of each edge in weighted graph.
pub trait GraphData: GraphBase {
  type Data;
  fn data(&self, node: Self::NodeId) -> &Self::Data;
}

// ====== Neighbors access =====
/// Forward direction marker used for specializing [`IntoNeighbors`]
pub struct Forward;
/// Backward direction marker used for specializing [`IntoNeighbors`]
pub struct Backward;

#[doc(hidden)]
pub trait ForwardOrBackward {}
#[doc(hidden)]
impl ForwardOrBackward for Forward {}
#[doc(hidden)]
impl ForwardOrBackward for Backward {}

/// Trait for defining graph connectivity
/// 
/// The trait can be implemented separately for returning neighbors for when traveling in both 
/// forward and backward direction:
/// - `IntoNeighbors<Forward>::neighbors()` for none with id `u` must return all nodes `v` such that there is an edge from `u` to `v` in the graph.
/// - `IntoNeighbors<Backward>::neighbors()` for none with id `u` must return all nodes `w` such that there is an edge from `w` to `u` in the graph.
/// 
/// For graphs which implement the trait in both directions it can be cumbersome to call this function. Helper function [`neighbors_forward`] 
/// and [`neighbors_backward`] can be used to receive neighbor nodes of graph. 
pub trait IntoNeighbors<Direction: ForwardOrBackward>: GraphBase {
  type Neighbors: Iterator<Item = Self::NodeId>;
  fn neighbors(self, node: Self::NodeId) -> Self::Neighbors;
}

/// Returns an iterator over all forward neighbors of the graph
pub fn neighbors_forward<G: IntoNeighbors<Forward>>(graph: G, node_id: G::NodeId) -> G::Neighbors {
  graph.neighbors(node_id)
}

/// Returns an iterator over all backward neighbors of the graph
pub fn neighbors_backward<G: IntoNeighbors<Backward>>(
  graph: G,
  node_id: G::NodeId,
) -> G::Neighbors {
  graph.neighbors(node_id)
}

// ====== Graph extension =====

/// A generator of valid node identifiers
pub trait NodesExtension<NodeId: Identifier> {
  fn new_node_id(&self) -> Option<NodeId>;
  fn contains(&self, id: NodeId) -> bool;
}

/// Allows "extending" the graph with more node identifiers 
pub trait Extensible: GraphBase {
  type Extension: NodesExtension<Self::NodeId>;

  fn new_extension(&self) -> Self::Extension;
}

// ====== Blanket implementations =====

impl<'a, G: GraphBase> GraphBase for &'a G {
  type NodeId = G::NodeId;
}

impl<'a, G: GraphData> GraphData for &'a G {
  type Data = G::Data;

  fn data(&self, node: Self::NodeId) -> &Self::Data {
    (*self).data(node)
  }
}

impl<'a, G: Extensible> Extensible for &'a G {
  type Extension = G::Extension;

  fn new_extension(&self) -> Self::Extension {
    (*self).new_extension()
  }
}

impl<G: GraphBase, T> GraphBase for (G, T)
{
  type NodeId = G::NodeId;
}

impl<G: IntoNeighbors<Forward>, T> IntoNeighbors<Forward> for (G, T) {
  type Neighbors = <G as IntoNeighbors<Forward>>::Neighbors;

  fn neighbors(self, node_id: G::NodeId) -> Self::Neighbors {
    <G as IntoNeighbors<Forward>>::neighbors(self.0, node_id)
  }
}

impl<G: IntoNeighbors<Backward>, T> IntoNeighbors<Backward> for (G, T) {
  type Neighbors = <G as IntoNeighbors<Backward>>::Neighbors;

  fn neighbors(self, node_id: G::NodeId) -> Self::Neighbors {
    <G as IntoNeighbors<Backward>>::neighbors(self.0, node_id)
  }
}

impl<G: Extensible, T> Extensible for (G, T) {
  type Extension = G::Extension;

  fn new_extension(&self) -> Self::Extension {
    self.0.new_extension()
  }
}
