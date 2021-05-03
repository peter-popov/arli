use crate::graph::GraphBase;
use crate::spatial::{BoundingBox, Position};

/// Defines how to obtain a geometry of a graph node
///
/// arli only uses graph nodes(we don't define an edge explicitly). Node geometry
/// is a polyline because we each node corresponds to a segment in a road-network(traveled in a certain direction). See `edge-based graph` for more details.
pub trait IntoGeometry: GraphBase {
  type P: Into<Position>;
  type Geometry: Iterator<Item = Self::P>;
  fn geometry(self, node: Self::NodeId) -> Self::Geometry;
}

/// Defines a spatial index for graph nodes
pub trait Spatial: GraphBase {
  type Nodes: IntoIterator<Item = Self::NodeId>;
  fn find_nodes(&self, bbox: &BoundingBox) -> Self::Nodes;
}

impl<'a, G: Spatial> Spatial for &'a G {
  type Nodes = G::Nodes;

  fn find_nodes(&self, bbox: &BoundingBox) -> Self::Nodes {
    (*self).find_nodes(bbox)
  }
}

impl<G: IntoGeometry, T> IntoGeometry for (G, T) {
  type P = G::P;
  type Geometry = G::Geometry;

  fn geometry(self, node: Self::NodeId) -> Self::Geometry {
    self.0.geometry(node)
  }
}
