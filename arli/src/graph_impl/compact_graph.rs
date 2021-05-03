use crate::graph::*;
use super::common::*;
use serde::{Deserialize, Serialize};
use std::mem::size_of;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
struct Node {
  // Offset in the `edge_references` array to iterate over outgoing edges.
  pub out_edges_offset: Idx,
  // Offset in the `edge_references` array to iterate over outgoing edges.
  pub in_edges_offset: Idx,
}

/// Graph with geometry and spatial index which uses a compact memory layout for it's data. The graph is immutable.
#[derive(Serialize, Deserialize)]
pub struct CompactGraph<NodeData> {
  nodes: Vec<Node>,
  data: Vec<NodeData>,
  // All ingoing and outgoing target node ids are stored in this big array. Node::out_edges and Node::in_edges will refer into it.
  edge_references: Vec<Idx>,
}

impl<NodeData> GraphBase for CompactGraph<NodeData> {
  type NodeId = Idx;
}

impl<'a, NodeData> IntoNeighbors<Forward> for &'a CompactGraph<NodeData> {
  type Neighbors = RefIterator<'a, Self::NodeId>;

  fn neighbors(self, node_id: Idx) -> Self::Neighbors {
    let start = self.nodes[node_id as usize].out_edges_offset;
    let end = self.nodes[node_id as usize + 1].out_edges_offset; // Safe to do +1 since we added a sentinel node
    RefIterator::new(&self.edge_references, start, end)
  }
}

impl<'a, NodeData> IntoNeighbors<Backward> for &'a CompactGraph<NodeData> {
  type Neighbors = RefIterator<'a, Idx>;

  fn neighbors(self, node_id: Idx) -> Self::Neighbors {
    let start = self.nodes[node_id as usize].in_edges_offset;
    let end = self.nodes[node_id as usize + 1].in_edges_offset; // Safe to do +1 since we added a sentinel node
    RefIterator::new(&self.edge_references, start, end)
  }
}

impl<NodeData> GraphData for CompactGraph<NodeData> {
  type Data = NodeData;

  fn data(&self, node_id: Idx) -> &Self::Data {
    &self.data[node_id as usize]
  }
}

impl<NodeData> CompactGraph<NodeData> {
  pub fn from_row_data(data: Vec<NodeData>, offsets: Vec<usize>, out_references: Vec<Idx>) -> Self {
    let num_nodes = data.len();
    let num_edges = out_references.len();

    let mut nodes: Vec<Node> = Vec::with_capacity(num_nodes);
    let mut edge_references = out_references;

    // Collect outgoing edges and geometry
    for out_offset in offsets {
      nodes.push(Node {
        out_edges_offset: out_offset as Idx,
        in_edges_offset: 0,
      });
    }

    // A sentinel node, so that we can easily iterate from node[i].offset to node[i+1].offset
    nodes.push(Node {
      out_edges_offset: num_edges as Idx,
      in_edges_offset: 0,
    });

    // Constructing ingoing references:
    // 1. Collect all pairs of `(to, from)` and sort them by `to`
    let mut in_references_tmp = Vec::with_capacity(num_edges);
    for from in 0..num_nodes {
      let range_start = nodes[from].out_edges_offset;
      let range_end = nodes[from + 1].out_edges_offset;
      for to in RefIterator::new(&edge_references, range_start, range_end) {
        in_references_tmp.push((to as usize, from as Idx))
      }
    }
    in_references_tmp.sort();

    // 2. Populate edge_references for ingoing edges and track number of ingoing in `nodes[n+1].in_edges_offset`
    edge_references.reserve(num_edges);
    for (to, from) in in_references_tmp {
      nodes[to + 1].in_edges_offset += 1;
      edge_references.push(from);
    }

    // 3. Convert number of ingoing edges into he global offsets.
    nodes[0].in_edges_offset = num_edges as Idx;
    for n in 0..num_nodes {
      nodes[n + 1].in_edges_offset += nodes[n].in_edges_offset;
    }

    CompactGraph {
      data: data,
      nodes: nodes,
      edge_references: edge_references,
    }
  }

  pub fn number_of_nodes(&self) -> usize {
    self.nodes.len()
  }

  pub fn number_of_edges(&self) -> usize {
    self.edge_references.len() / 2
  }

  pub fn print_stats(&self) {
    print_vector_size("self.nodes", &self.nodes);
    print_vector_size("self.data", &self.data);
    print_vector_size("self.edge_references", &self.edge_references);
  }

  pub fn shrink(&mut self) {
    self.data.shrink_to_fit();
    self.nodes.shrink_to_fit();
    self.edge_references.shrink_to_fit();
  }
}

pub fn print_vector_size<T>(name: &str, v: &Vec<T>) {
  println!(
    "\t{}: count = {}/{}, element = {}, total = {} Mb",
    name,
    v.len(),
    v.capacity(),
    size_of::<T>(),
    v.capacity() * size_of::<T>() / 1024 / 1024
  );
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::HashSet;

  #[test]
  fn test_compact_graph() {
    let data = vec!["node0", "node1", "node1-", "node2"];

    let graph = CompactGraph::from_row_data(data, vec![0, 2, 3, 4], vec![1, 3, 2, 3]);

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
  }
}
