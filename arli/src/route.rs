//! Route finding algorithms.

use crate::graph::*;
use crate::overlay::OverlayGraph;
use crate::search_space::*;
use crate::spatial::*;
use crate::waypoint::*;

use std::collections::HashSet;

pub trait RoutableGraph: GraphData + IntoNeighbors<Forward> + IntoNeighbors<Backward> + IntoGeometry + Spatial {}
impl<T> RoutableGraph for T where T: GraphData + IntoNeighbors<Forward> + IntoNeighbors<Backward> + Spatial + IntoGeometry {}

pub struct Route<W: Weight, N: Identifier> {
  pub cost: W,
  pub ids: Vec<N>,
}

pub fn connect_waypoints_to_graph<G: Copy + IntoNeighbors<Forward> + IntoGeometry + Extensible>(
  graph: G,
  origin: &mut MatchedWaypoint<G::NodeId>,
  _: &mut MatchedWaypoint<G::NodeId>,
) -> OverlayGraph<G> {
  let mut overlay = OverlayGraph::new(graph);

  for snapped in &mut origin.snapped {
    snapped.1 = overlay.add_origin(snapped.1, snapped.0).unwrap();
  }

  overlay
}

pub fn snap_and_route<G: Copy + RoutableGraph<P = Position> + Weighted>(
  graph: G,
  from: &Position,
  to: &Position,
) -> Option<Route<G::Weight, G::NodeId>> {
  let from_matched = match_waypoint(graph, from);
  if from_matched.snapped.is_empty() {
    println!(
      "From ({}, {}) isn't snapped",
      from_matched.waypoint.x, from_matched.waypoint.y
    );
    return None;
  }

  let to_matched = match_waypoint(graph, to);
  if to_matched.snapped.is_empty() {
    println!(
      "To ({}, {}) isn't snapped",
      to_matched.waypoint.x, to_matched.waypoint.y
    );
    return None;
  }

  route(graph, &from_matched, &to_matched)
}

pub fn snap_and_route_with_cost<
  W: Weight,
  G: Copy + RoutableGraph,
  C: Copy + Fn(&G::Data, &G::Data) -> W,
>(
  graph: G,
  cost: C,
  from: &Position,
  to: &Position,
) -> Option<Route<W, G::NodeId>> {
  let from_matched = match_waypoint(graph, from);
  if from_matched.snapped.is_empty() {
    println!(
      "From ({}, {}) isn't snapped",
      from_matched.waypoint.x, from_matched.waypoint.y
    );
    return None;
  }

  let to_matched = match_waypoint(graph, to);
  if to_matched.snapped.is_empty() {
    println!(
      "To ({}, {}) isn't snapped",
      to_matched.waypoint.x, to_matched.waypoint.y
    );
    return None;
  }

  route((graph, cost), &from_matched, &to_matched)
}

pub fn route<G: Copy + IntoNeighbors<Forward> + Weighted>(
  graph: G,
  from: &MatchedWaypoint<G::NodeId>,
  to: &MatchedWaypoint<G::NodeId>,
) -> Option<Route<G::Weight, G::NodeId>> {
  let mut forward_search: SearchSpace<G::Weight, G::NodeId> = SearchSpace::new();

  for SnappedOnEdge(_, id) in &from.snapped {
    //todo: partial cost and augmented graph are needed to properly initialize the start and end edges
    forward_search.init(*id);
  }

  let target_ids: HashSet<G::NodeId> = to.snapped.iter().map(|s| s.1).collect();

  loop {
    forward_search.update(graph);
    match forward_search.min() {
      Some((id, value)) => {
        if target_ids.contains(&id) {
          return Some(Route {
            cost: value,
            // Need to reverse the list to get elements in the routing order
            ids: forward_search.unwind(id).iter().rev().cloned().collect(),
          });
        }
      }
      None => return None,
    }
  }
}

pub fn collect_route_geometry<G: Copy + IntoGeometry, Ids: Iterator<Item = G::NodeId>>(
  graph: G,
  ids: Ids,
) -> Vec<Position> {
  ids
    .flat_map(|id| graph.geometry(id))
    .map(|p| p.into())
    .collect()
}

pub fn calculate_weight<G: Copy + Weighted, Ids: Iterator<Item = G::NodeId>>(
  graph: G,
  ids: Ids,
) -> G::Weight
where
  G::Weight: std::iter::Sum<G::Weight>,
{
  ids.map(|id| graph.transition_weight(id, id)).sum()
}
