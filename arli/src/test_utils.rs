use crate::spatial::*;
use crate::graph_impl::*;

use std::collections::HashMap;

pub struct Segment {
  geometry: Vec<Position>,
}

pub fn simple_segment_length_cost(from: &Segment, _to: &Segment) -> i32 {
  haversine_distance(&from.geometry[0], &from.geometry[1]) as i32
}

impl Segment {
  pub fn new(from: Position, to: Position) -> Self {
    Segment {
      geometry: vec![from, to],
    }
  }
}

impl HasGeometry for Segment {
  type Points = std::vec::IntoIter<Position>;
  fn geometry(&self) -> Self::Points {
    self.geometry.clone().into_iter()
  }
}

pub fn graph_from_data_and_edges<T>(data: Vec<T>, edges: Vec<(usize, usize)>) -> DynamicGraph<T> {
  let mut graph = DynamicGraph::new_with_data(data);
  for (from, to) in edges {
    graph.add_edge(from as u32, to as u32);
  }
  graph
}

pub fn graph_from_intersections(positions: Vec<Position>, adjacency: Vec<(usize, usize)>) -> DynamicSpatialGraph<Segment> {
  
  let mut segments = Vec::new();
  let mut segments_adj = HashMap::new();

  for (from, to) in &adjacency {
    let id = segments.len();
    segments.push(Segment::new(positions[*from], positions[*to]));
    segments_adj.entry(from).or_insert_with(||Vec::new()).push(id);
  }
  
  let mut graph = DynamicSpatialGraph::new_with_data(segments);

  for (id, (_, to)) in adjacency.iter().enumerate() {
    if let Some(outgoing) = segments_adj.get(to) {
      for outgoing_id in outgoing {
        graph.add_edge(id as u32, *outgoing_id as u32);
      }
    }
  }

  graph
}