use crate::graph::*;
use crate::spatial::{s2_cover, to_s2, BoundingBox, Position};
use super::compact_graph::*;
use super::common::*;

use s2;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use superslice::*;

const SPATIAL_INDEX_S2_LEVEL: u64 = 13;

/// Graph with geometry and spatial index which uses a compact memory layout for it's data. The graph is immutable.
#[derive(Serialize, Deserialize)]
pub struct CompactSpatialGraph<NodeData> {
  graph: CompactGraph<NodeData>,
  // For each node reference ot its geometry in the points array.
  geometry_refs: Vec<RangeRef>,
  // All geometry points are stored in this array.
  points: Vec<Position>,
  // S2-based spatial index, sorted list of tuples
  blocks: Vec<(s2::cellid::CellID, Idx)>,
}

impl<NodeData> GraphBase for CompactSpatialGraph<NodeData> {
  type NodeId = Idx;
}

impl<'a, Data> IntoGeometry for &'a CompactSpatialGraph<Data> {
  type P = Position;
  type Geometry = RefIterator<'a, Position>;

  fn geometry(self, id: Idx) -> Self::Geometry {
    RefIterator::from_range(&self.points, &self.geometry_refs[id as usize])
  }
}

impl<'a, Data> Spatial for CompactSpatialGraph<Data> {
  type Nodes = std::vec::IntoIter<Self::NodeId>;

  fn find_nodes(&self, bbox: &BoundingBox) -> Self::Nodes {
    let mut result = Vec::new();
    let cover = s2_cover(bbox, SPATIAL_INDEX_S2_LEVEL as u8);
    for cell_id in cover.0 {
      let rng = self.blocks.equal_range_by_key(&cell_id, |i| i.0);
      let ids = self.blocks.get(rng).unwrap_or(&[]);
      result.extend(ids.iter().map(|i| i.1));
    }
    result.into_iter()
  }
}

impl<'a, NodeData> IntoNeighbors<Forward> for &'a CompactSpatialGraph<NodeData> {
  type Neighbors = <&'a CompactGraph<NodeData> as IntoNeighbors<Forward>>::Neighbors;

  fn neighbors(self, node_id: Idx) -> Self::Neighbors {
    <&CompactGraph<NodeData> as IntoNeighbors<Forward>>::neighbors(&self.graph, node_id)
  }
}

impl<'a, NodeData> IntoNeighbors<Backward> for &'a CompactSpatialGraph<NodeData> {
  type Neighbors = <&'a CompactGraph<NodeData> as IntoNeighbors<Backward>>::Neighbors;

  fn neighbors(self, node_id: Idx) -> Self::Neighbors {
    <&CompactGraph<NodeData> as IntoNeighbors<Backward>>::neighbors(&self.graph, node_id)
  }
}

impl<NodeData> Extensible for CompactSpatialGraph<NodeData> {
  type Extension = MoreNodes;

  fn new_extension(&self) -> Self::Extension {
    MoreNodes::new(self.number_of_nodes() as Idx)
  }
}

impl<NodeData> GraphData for CompactSpatialGraph<NodeData> {
  type Data = NodeData;

  fn data(&self, node_id: Idx) -> &Self::Data {
    self.graph.data(node_id)
  }
}

impl<NodeData> CompactSpatialGraph<NodeData> {
  pub fn from_row_data(
    base_graph: CompactGraph<NodeData>,
    offsets: Vec<(usize, usize)>,
    points: Vec<Position>,
  ) -> Self {
    let geometry_refs: Vec<RangeRef> = offsets
      .iter()
      .map(|(start, end)| RangeRef(*start as Idx, *end as Idx))
      .collect();

    // Build spatial index
    let mut blocks = Vec::with_capacity(2 * base_graph.number_of_nodes());
    for (idx, geom_ref) in geometry_refs.iter().enumerate() {
      let cells = RefIterator::from_range(&points, geom_ref)
        .map(|p| to_s2(&p).parent(SPATIAL_INDEX_S2_LEVEL))
        .collect::<HashSet<_>>();
      for cell_id in cells {
        blocks.push((cell_id, idx as Idx));
      }
    }
    blocks.sort_unstable_by_key(|(cell_id, _)| *cell_id);

    CompactSpatialGraph {
      graph: base_graph,
      geometry_refs: geometry_refs,
      points: points,
      blocks: blocks,
    }
  }

  pub fn number_of_nodes(&self) -> usize {
    self.graph.number_of_nodes()
  }

  pub fn number_of_edges(&self) -> usize {
    self.graph.number_of_edges()
  }

  pub fn print_stats(&self) {
    self.graph.print_stats();
    print_vector_size("self.geom_refs", &self.geometry_refs);
    print_vector_size("self.points", &self.points);
    print_vector_size("self.blocks", &self.blocks);
  }

  pub fn shrink(&mut self) {
    self.graph.shrink();
    self.geometry_refs.shrink_to_fit();
    self.points.shrink_to_fit();
    self.blocks.shrink_to_fit();
  }
}

#[cfg(test)]
mod tests {
  use super::super::super::spatial::*;
  use super::*;
  use std::collections::HashSet;

  #[test]
  fn test_compact_spatial_graph() {
    let data = vec!["node0", "node1", "node1-", "node2"];

    let a = Position::from((13.3548259, 52.4947094));
    let b = Position::from((13.3596968, 52.4943175));
    let c = Position::from((13.3608126, 52.4949576));
    let d = Position::from((13.3642673, 52.4956239));
    let e = Position::from((13.3625292, 52.4924492));

    let points: Vec<Position> = vec![a, b, b, c, d, /*d, c, b,*/ b, e];

    let base_graph = CompactGraph::from_row_data(data, vec![0, 2, 3, 4], vec![1, 3, 2, 3]);

    let geom_offsets: Vec<(usize, usize)> = vec![(0, 2), (2, 5), (4, 1), (5, 7)];

    let graph = CompactSpatialGraph::from_row_data(base_graph, geom_offsets, points);

    let out_edges_0: HashSet<_> = neighbors_forward(&graph, 0).collect();
    assert_eq!(out_edges_0.len(), 2);
    assert!(out_edges_0.contains(&1));
    assert!(out_edges_0.contains(&3));

    let out_edges_2: HashSet<_> = neighbors_forward(&graph, 2).collect();
    assert_eq!(out_edges_2.len(), 1);
    assert!(out_edges_2.contains(&3));

    let in_edges_1: HashSet<_> = neighbors_backward(&graph, 1).collect();
    assert_eq!(in_edges_1.len(), 1);
    assert!(in_edges_1.contains(&0));

    let in_edges_3: HashSet<_> = neighbors_backward(&graph, 3).collect();
    assert_eq!(in_edges_3.len(), 2);
    assert!(in_edges_3.contains(&2));
    assert!(in_edges_3.contains(&0));

    assert_eq!((&graph).geometry(0).collect::<Vec<_>>(), vec![a, b]);
    assert_eq!((&graph).geometry(1).collect::<Vec<_>>(), vec![b, c, d]);
    assert_eq!((&graph).geometry(2).collect::<Vec<_>>(), vec![d, c, b]);
    assert_eq!((&graph).geometry(3).collect::<Vec<_>>(), vec![b, e]);
  }
}
