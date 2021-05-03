use crate::graph::*;
use crate::spatial::{bounding_box, BoundingBox, Position};
use super::common::*;
use super::dynamic_graph::*;
use rstar::{RTree, RTreeObject, AABB};

pub trait HasGeometry {
  type Points: Iterator<Item = Position>;
  fn geometry(&self) -> Self::Points;
}

#[derive(Debug)]
struct Entry {
  id: Idx,
  bbox: BoundingBox,
}

fn to_aabb(bbox: &BoundingBox) -> AABB<[f32; 2]> {
  let min = bbox.min().x_y();
  let max = bbox.max().x_y();
  AABB::from_corners([min.0, min.1], [max.0, max.1])
}

impl Entry {
  fn new<Data: HasGeometry>(id: Idx, data: &Data) -> Self {
    Self {
      id: id,
      bbox: bounding_box(data.geometry()).unwrap(),
    }
  }
}

impl RTreeObject for Entry {
  type Envelope = AABB<[f32; 2]>;
  fn envelope(&self) -> Self::Envelope {
    to_aabb(&self.bbox)
  }
}

/// Simple graph implementation which stores edge references and geometry in as an vector in each node. Not memory efficient. But allows adding nodes dynamically - useful for testing.
pub struct DynamicSpatialGraph<NodeData> {
  graph: DynamicGraph<NodeData>,
  rtree: RTree<Entry>,
}

impl<NodeData: HasGeometry> DynamicSpatialGraph<NodeData> {
  pub fn new() -> Self {
    Self {
      graph: DynamicGraph::new(),
      rtree: RTree::new(),
    }
  }

  pub fn new_with_data(data: Vec<NodeData>) -> Self {
    let entries = data
      .iter()
      .enumerate()
      .map(|id_and_data| Entry::new(id_and_data.0 as Idx, id_and_data.1))
      .collect();
    Self {
      graph: DynamicGraph::new_with_data(data),
      rtree: RTree::bulk_load(entries),
    }
  }

  pub fn add_node(&mut self, data: NodeData) -> Idx {
    self.graph.add_node(data)
  }

  pub fn add_edge(&mut self, from: Idx, to: Idx) -> &mut Self {
    self.graph.add_edge(from, to);
    self
  }

  pub fn number_of_nodes(&self) -> usize {
    self.graph.number_of_nodes()
  }

  pub fn number_of_edges(&self) -> usize {
    self.graph.number_of_edges()
  }
}

impl<NodeData> GraphBase for DynamicSpatialGraph<NodeData> {
  type NodeId = Idx;
}

impl<'a, Data: HasGeometry> IntoGeometry for &'a DynamicSpatialGraph<Data> {
  type P = Position;
  type Geometry = Data::Points;

  fn geometry(self, id: Idx) -> Self::Geometry {
    self.graph.data(id).geometry()
  }
}

impl<Data: HasGeometry> Spatial for DynamicSpatialGraph<Data> {
  type Nodes = Vec<Self::NodeId>;

  fn find_nodes(&self, bbox: &BoundingBox) -> Self::Nodes {
    let envelope = to_aabb(&bbox);
    let results_iter = self.rtree.locate_in_envelope_intersecting(&envelope);
    results_iter.map(|entry| entry.id).collect()
  }
}

impl<'a, NodeData> IntoNeighbors<Forward> for &'a DynamicSpatialGraph<NodeData> {
  type Neighbors = <&'a DynamicGraph<NodeData> as IntoNeighbors<Forward>>::Neighbors;

  fn neighbors(self, node_id: Idx) -> Self::Neighbors {
    <&DynamicGraph<NodeData> as IntoNeighbors<Forward>>::neighbors(&self.graph, node_id)
  }
}

impl<'a, NodeData> IntoNeighbors<Backward> for &'a DynamicSpatialGraph<NodeData> {
  type Neighbors = <&'a DynamicGraph<NodeData> as IntoNeighbors<Backward>>::Neighbors;

  fn neighbors(self, node_id: Idx) -> Self::Neighbors {
    <&DynamicGraph<NodeData> as IntoNeighbors<Backward>>::neighbors(&self.graph, node_id)
  }
}

impl<NodeData> GraphData for DynamicSpatialGraph<NodeData> {
  type Data = NodeData;

  fn data(&self, node_id: Idx) -> &Self::Data {
    self.graph.data(node_id)
  }
}

impl<NodeData> Extensible for DynamicSpatialGraph<NodeData> {
  type Extension = MoreNodes;

  fn new_extension(&self) -> Self::Extension {
    MoreNodes::new(self.graph.number_of_nodes() as Idx)
  }
}
#[cfg(test)]
mod tests {
  use super::super::super::spatial::*;
  use super::super::super::test_utils::graph_from_intersections;
  use super::*;
  use std::collections::HashSet;

  const POSITIONS: [Position; 5] = [
    Position { x: 1.0, y: 1.0 },
    Position { x: 1.0, y: 3.0 },
    Position { x: 3.0, y: 3.0 },
    Position { x: 3.0, y: 1.0 },
    Position { x: 2.5, y: 2.5 },
  ];

  #[test]
  fn test_wraps_normal_graph() {
    let graph = graph_from_intersections(
      Vec::from(POSITIONS),
      vec![(0, 1), (1, 2), (2, 3), (3, 0), (4, 2), (0, 4)],
    );

    assert_eq!(graph.number_of_nodes(), 6);

    let n1_out_edges: HashSet<_> = neighbors_forward(&graph, 3).collect();
    assert_eq!(n1_out_edges, [0, 5].iter().cloned().collect());


    let n2_in_edges: HashSet<_> = neighbors_backward(&graph, 2).collect();
    assert_eq!(n2_in_edges, [4, 1].iter().cloned().collect());
  }

  #[test]
  fn test_spatial() {
    let graph = graph_from_intersections(
      Vec::from(POSITIONS),
      vec![(0, 1), (1, 2), (2, 3), (3, 0), (4, 3)],
    );

    assert_eq!(graph.number_of_nodes(), 5);

    let as_set = |v: Vec<u32>| v.iter().cloned().collect::<HashSet<u32>>();

    let res0 = graph.find_nodes(&BoundingBox::new((0.5, 0.5), (1.5, 1.5)));
    assert_eq!(as_set(res0), as_set(vec![0, 3]));

    let res1 = graph.find_nodes(&BoundingBox::new((1.4, 1.4), (2.5, 2.5)));
    assert_eq!(as_set(res1), as_set(vec![4]));

    let res2 = graph.find_nodes(&BoundingBox::new((0.0, 2.0), (3.5, 3.5)));
    assert_eq!(as_set(res2), as_set(vec![0, 1, 2, 4]));
  }
}
