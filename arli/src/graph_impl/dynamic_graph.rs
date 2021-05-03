use crate::graph::*;
use super::common::*;

#[derive(Default, Clone)]
struct Node {
  out_edges: Vec<Idx>,
  in_edges: Vec<Idx>,
}

/// Simple graph implementation which stores edge references and geometry in as an vector in each node. Not memory efficient. But allows adding nodes dynamically - useful for testing.
pub struct DynamicGraph<NodeData> {
  nodes: Vec<Node>,
  data: Vec<NodeData>,
}

impl<NodeData> DynamicGraph<NodeData> {
  pub fn new() -> Self {
    Self {
      nodes: Vec::new(),
      data: Vec::new(),
    }
  }

  pub fn new_with_data(data: Vec<NodeData>) -> Self {
    Self {
      nodes: vec![Default::default(); data.len()],
      data: data,
    }
  }

  pub fn add_node(&mut self, data: NodeData) -> Idx {
    let size = self.data.len();
    self.nodes.push(Default::default());
    self.data.push(data);
    size as Idx
  }

  pub fn add_edge(&mut self, from: Idx, to: Idx) -> &mut Self {
    self.nodes[from as usize].out_edges.push(to);
    self.nodes[to as usize].in_edges.push(from);
    self
  }

  pub fn number_of_nodes(&self) -> usize {
    self.nodes.len()
  }

  pub fn number_of_edges(&self) -> usize {
    self
      .nodes
      .iter()
      .map(|node| node.out_edges.len() + node.in_edges.len())
      .sum::<usize>()
      / 2
  }
}

impl<NodeData> GraphBase for DynamicGraph<NodeData> {
  type NodeId = Idx;
}

impl<'a, NodeData> IntoNeighbors<Forward> for &'a DynamicGraph<NodeData> {
  type Neighbors = std::iter::Cloned<std::slice::Iter<'a, Idx>>;

  fn neighbors(self, node_id: Idx) -> Self::Neighbors {
    self.nodes[node_id as usize].out_edges.iter().cloned()
  }
}

impl<'a, NodeData> IntoNeighbors<Backward> for &'a DynamicGraph<NodeData> {
  type Neighbors = std::iter::Cloned<std::slice::Iter<'a, Idx>>;

  fn neighbors(self, node_id: Idx) -> Self::Neighbors {
    self.nodes[node_id as usize].in_edges.iter().cloned()
  }
}

impl<NodeData> GraphData for DynamicGraph<NodeData> {
  type Data = NodeData;

  fn data(&self, node_id: Idx) -> &Self::Data {
    &self.data[node_id as usize]
  }
}

impl<NodeData> Extensible for DynamicGraph<NodeData> {
  type Extension = MoreNodes;

  fn new_extension(&self) -> Self::Extension {
    MoreNodes::new(self.number_of_nodes() as Idx)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use super::super::super::test_utils::graph_from_data_and_edges;
  use std::collections::HashSet;

  #[test]
  fn test_dynamic_graph() {

    let graph = graph_from_data_and_edges(
      vec!["1", "2", "3", "4", "5"],
      vec![(0, 1), (2, 0), (2, 1), (2, 4), (1, 2), (0, 3), (4, 3)],
    );

    assert_eq!(graph.number_of_nodes(), 5);
    assert_eq!(graph.number_of_edges(), 7);

    let n1_out_edges: HashSet<_> = neighbors_forward(&graph, 0).collect();
    assert_eq!(n1_out_edges, [1, 3].iter().cloned().collect());

    let n2_in_edges: HashSet<_> = neighbors_backward(&graph, 1).collect();
    assert_eq!(n2_in_edges, [0, 2].iter().cloned().collect());
  }
}
