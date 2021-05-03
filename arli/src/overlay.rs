use crate::graph::*;
use crate::spatial::{cut_geometry_after, cut_geometry_before, Position};
use crate::waypoint::SnappedPosition;

use std::collections::HashMap;

struct OverlayNode<N: Identifier> {
  pub base_id: N,
  pub out_edges: Vec<N>,
  pub in_edges: Vec<N>,
  pub geometry: Vec<Position>,
  pub snapped_position: SnappedPosition,
}

impl<N: Identifier> OverlayNode<N> {
  pub fn new(base_id: N, positions: Vec<Position>, snapped_position: SnappedPosition) -> Self {
    Self {
      base_id: base_id,
      out_edges: Vec::new(),
      in_edges: Vec::new(),
      geometry: positions,
      snapped_position: snapped_position,
    }
  }
}

pub struct OverlayGraph<G: Extensible> {
  base_graph: G,
  overlay_nodes: HashMap<G::NodeId, OverlayNode<G::NodeId>>,
  extended_ids: G::Extension,
}

impl<G: Copy + Extensible> OverlayGraph<G> {
  pub fn new(graph: G) -> Self {
    let ext = graph.new_extension();
    Self {
      base_graph: graph,
      overlay_nodes: HashMap::new(),
      extended_ids: ext,
    }
  }

  fn find_node(&self, node_id: G::NodeId) -> (G::NodeId, Option<SnappedPosition>) {
    if self.extended_ids.contains(node_id) {
      let overlay_node = self.overlay_nodes.get(&node_id).unwrap();
      (overlay_node.base_id, Some(overlay_node.snapped_position))
    } else {
      (node_id, None)
    }
  }
}

impl<G: Copy + IntoNeighbors<Forward> + IntoGeometry + Extensible> OverlayGraph<G> {
  pub fn add_origin(
    &mut self,
    base_node_id: G::NodeId,
    snapped_position: SnappedPosition,
  ) -> Option<G::NodeId> {
    let new_id = self.extended_ids.new_node_id();
    if let Some(id) = new_id {
      self
        .overlay_nodes
        .entry(id)
        .or_insert(OverlayNode::new(
          base_node_id,
          cut_geometry_before(
            (self.base_graph).geometry(base_node_id),
            snapped_position.snapped,
          ),
          // TODO: `1-factor` below is ugly, but needed for to calculate cost properly
          SnappedPosition {
            snapped: snapped_position.snapped,
            factor: 1.0 - snapped_position.factor,
            distance: snapped_position.distance,
          },
        ))
        .out_edges = neighbors_forward(self.base_graph, base_node_id).collect();
    }
    new_id
  }
}

impl<G: Copy + IntoNeighbors<Backward> + IntoGeometry + Extensible> OverlayGraph<G> {
  pub fn add_destination(
    &mut self,
    base_node_id: G::NodeId,
    snapped_position: SnappedPosition,
  ) -> Option<G::NodeId> {
    let new_id = self.extended_ids.new_node_id();
    if let Some(id) = new_id {
      self
        .overlay_nodes
        .entry(id)
        .or_insert(OverlayNode::new(
          base_node_id,
          cut_geometry_after(
            self.base_graph.geometry(base_node_id),
            snapped_position.snapped,
          ),
          snapped_position,
        ))
        .in_edges = neighbors_backward(self.base_graph, base_node_id).collect();
    }
    new_id
  }
}

impl<G: Extensible + GraphBase> GraphBase for OverlayGraph<G> {
  type NodeId = G::NodeId;
}

impl<'a, G: Copy + Extensible + IntoNeighbors<Forward>> IntoNeighbors<Forward>
  for &'a OverlayGraph<G>
{
  type Neighbors = OverlayIterator<
    <G as IntoNeighbors<Forward>>::Neighbors,
    std::iter::Cloned<std::slice::Iter<'a, G::NodeId>>,
  >;

  fn neighbors(self, node_id: G::NodeId) -> Self::Neighbors {
    if self.extended_ids.contains(node_id) {
      OverlayIterator::Overlay(
        self
          .overlay_nodes
          .get(&node_id)
          .unwrap()
          .out_edges
          .iter()
          .cloned(),
      )
    } else {
      OverlayIterator::Base(neighbors_forward(self.base_graph, node_id))
    }
  }
}

impl<'a, G: Copy + Extensible + IntoNeighbors<Backward>> IntoNeighbors<Backward>
  for &'a OverlayGraph<G>
{
  type Neighbors = OverlayIterator<
    <G as IntoNeighbors<Backward>>::Neighbors,
    std::iter::Cloned<std::slice::Iter<'a, G::NodeId>>,
  >;

  fn neighbors(self, node_id: G::NodeId) -> Self::Neighbors {
    if self.extended_ids.contains(node_id) {
      OverlayIterator::Overlay(
        self
          .overlay_nodes
          .get(&node_id)
          .unwrap()
          .in_edges
          .iter()
          .cloned(),
      )
    } else {
      OverlayIterator::Base(neighbors_backward(self.base_graph, node_id))
    }
  }
}

impl<
    'a,
    G: Copy + Extensible + GraphData,
    W: Weight,
    C: Fn(&G::Data, &G::Data, Option<SnappedPosition>) -> W,
  > Weighted for (&'a OverlayGraph<G>, C)
{
  type Weight = W;

  fn transition_weight(&self, from: Self::NodeId, to: Self::NodeId) -> Self::Weight {
    let (from_mapped, snapped_from) = self.0.find_node(from);
    let (to_mapped, _) = self.0.find_node(to);

    (self.1)(
      self.0.base_graph.data(from_mapped),
      self.0.base_graph.data(to_mapped),
      snapped_from,
    )
  }
}

impl<'a, G: Copy + Extensible + IntoGeometry<P = Position>> IntoGeometry for &'a OverlayGraph<G> {
  // TODO: I had to constraint G::P to be equal to Position, since overlay iterator currently require both iters to have the same item
  type P = G::P;
  type Geometry = OverlayIterator<G::Geometry, std::iter::Cloned<std::slice::Iter<'a, Position>>>;

  fn geometry(self, node_id: G::NodeId) -> Self::Geometry {
    if self.extended_ids.contains(node_id) {
      OverlayIterator::Overlay(
        self
          .overlay_nodes
          .get(&node_id)
          .unwrap()
          .geometry
          .iter()
          .cloned(),
      )
    } else {
      OverlayIterator::Base(self.base_graph.geometry(node_id))
    }
  }
}

pub enum OverlayIterator<BaseIter: Iterator, OverlayIter: Iterator> {
  Base(BaseIter),
  Overlay(OverlayIter),
}

impl<BaseIter: Iterator, OverlayIter: Iterator<Item = BaseIter::Item>> Iterator
  for OverlayIterator<BaseIter, OverlayIter>
{
  type Item = BaseIter::Item;

  fn next(&mut self) -> Option<Self::Item> {
    return match self {
      Self::Base(iterator) => iterator.next(),
      Self::Overlay(iterator) => iterator.next(),
    };
  }
}

#[cfg(test)]
mod tests {
  use super::super::test_utils::graph_from_intersections;
  use super::super::waypoint::SnappedPosition;
  use super::*;
  use std::collections::HashSet;

  const POSITIONS: [Position; 6] = [
    Position {
      x: 13.3331859,
      y: 52.4846880,
    },
    Position {
      x: 13.3331215,
      y: 52.4875758,
    },
    Position {
      x: 13.3331429,
      y: 52.4860078,
    },
    Position {
      x: 13.3351385,
      y: 52.4879351,
    },
    Position {
      x: 13.3352458,
      y: 52.4859163,
    },
    Position {
      x: 13.3352780,
      y: 52.4839889,
    },
  ];

  // Sample node-based graph converted into a segment-based graph
  //
  //   1                         4
  //   │                         ▲
  //   │                         │
  //   │                         │
  //   │                         │
  //   ▼                         │
  //   2────────x───────────────►3
  //   ▲        '                │
  //   │        * (snap)         │
  //   │                         │
  //   │                         │
  //   │                         ▼
  //   0                         5

  #[test]
  fn test_empty_overlay_not_affect_graph() {
    let graph = graph_from_intersections(
      Vec::from(POSITIONS),
      vec![(0, 2), (1, 2), (2, 3), (3, 4), (3, 5)],
    );

    let overlay = OverlayGraph::new(&graph);

    let n2_out_edges: HashSet<_> = neighbors_forward(&overlay, 2).collect();
    assert_eq!(n2_out_edges, [3, 4].iter().cloned().collect());

    let n2_in_edges: HashSet<_> = neighbors_backward(&overlay, 2).collect();
    assert_eq!(n2_in_edges, [0, 1].iter().cloned().collect());
  }

  #[test]
  fn test_overlay_split_after_preserves_connectivity() {
    let graph = graph_from_intersections(
      Vec::from(POSITIONS),
      vec![(0, 2), (1, 2), (2, 3), (3, 4), (3, 5)],
    );
    let mut overlay = OverlayGraph::new(&graph);

    let snapped_position = SnappedPosition {
      snapped: Position::from((13.3340375, 52.4859637)),
      distance: 0.0,
      factor: 0.4,
    };

    let new_node = overlay.add_origin(2, snapped_position).unwrap();

    let base_out_edges: HashSet<_> = neighbors_forward(&overlay, 2).collect();
    assert_eq!(base_out_edges, [3, 4].iter().cloned().collect());

    let overlay_out_edges: HashSet<_> = neighbors_forward(&overlay, new_node).collect();
    assert_eq!(overlay_out_edges, [3, 4].iter().cloned().collect());

    // We don't add incoming edges for the overlay... yet
    let overlay_in_edges: HashSet<_> = neighbors_backward(&overlay, new_node).collect();
    assert!(overlay_in_edges.is_empty());
  }

  #[test]
  fn test_overlay_split_adjusts_geometry() {
    let graph = graph_from_intersections(
      Vec::from(POSITIONS),
      vec![(0, 2), (1, 2), (2, 3), (3, 4), (3, 5)],
    );
    let mut overlay = OverlayGraph::new(&graph);

    let snapped_position = SnappedPosition {
      snapped: Position::from((13.3340375, 52.4859637)),
      distance: 0.0,
      factor: 0.4,
    };
    let new_node = overlay.add_origin(2, snapped_position).unwrap();

    let base_geometry: Vec<_> = (&overlay).geometry(2).collect();
    let overlay_geometry: Vec<_> = (&overlay).geometry(new_node).collect();

    assert_eq!(base_geometry[0], POSITIONS[2]);
    assert_eq!(base_geometry[1], POSITIONS[3]);
    assert_eq!(base_geometry[1], overlay_geometry[1]);
    assert_eq!(overlay_geometry[0], snapped_position.snapped);
  }
}
