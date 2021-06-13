//! Route finding algorithms.

use crate::graph::*;
use crate::overlay::OverlayGraph;
use crate::search_space::*;
use crate::spatial::*;
use crate::waypoint::*;
use std::cmp;

use std::collections::HashSet;

pub trait RoutableGraph: GraphData + IntoNeighbors<Forward> + IntoNeighbors<Backward> + IntoGeometry + Spatial {}
impl<T> RoutableGraph for T where T: GraphData + IntoNeighbors<Forward> + IntoNeighbors<Backward> + Spatial + IntoGeometry {}

pub struct Route<W: Weight, N: Identifier> {
  pub cost: W,
  pub ids: Vec<N>,
  pub num_resolved: u32,
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

  //route_bidir((graph, cost), &from_matched, &to_matched)
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
            num_resolved: forward_search.num_resolved(),
          });
        }
      }
      None => return None,
    }
  }
}

struct BidirectionalSearch<W:Weight, N:Identifier> {
  min_cost: Option<W>,
  metting_node: Option<N>,
}

impl<W:Weight, N:Identifier> BidirectionalSearch<W, N> {
  pub fn new() -> Self {
    Self {
      min_cost: None,
      metting_node: None,
    }
  }

  fn when_forward_relaxed(&mut self, backward: Option<W>, node: N, cost:W) {
    if let Some(min_bacward) = backward {
      if self.min_cost.filter(|v| min_bacward + cost < *v).is_none() {
        self.min_cost.replace(min_bacward + cost);
        self.metting_node.replace(node);
      }
    }
  }

  fn when_bacward_relaxed(&mut self, forward: Option<W>, node: N, cost:W) {
    if let Some(min_forward) = forward {
      if self.min_cost.filter(|v| min_forward + cost < *v).is_none() {
        self.min_cost.replace(min_forward + cost);
        self.metting_node.replace(node);
      }
    }
  }

  pub fn route_found(&self, forward: &SearchSpace<W, N>, backward: &SearchSpace<W, N>) -> Option<(N, W)> {
    // TODO: search seems to stop too late! 9mi vertices vs 6mi for signle direction!
    if let Some((_, min_f)) = forward.min() { 
      if let Some((_, min_b)) = backward.min() {
        let has_min_value = self.min_cost.filter(|min_value| min_f + min_b >= *min_value);
        return self.metting_node.zip(has_min_value);
      }
    }
    None
  }
}


pub fn route_bidir<G: Copy + IntoNeighbors<Forward> + IntoNeighbors<Backward> + Weighted>(
  graph: G,
  from: &MatchedWaypoint<G::NodeId>,
  to: &MatchedWaypoint<G::NodeId>,
) -> Option<Route<G::Weight, G::NodeId>> {
  let mut forward_search: SearchSpace<G::Weight, G::NodeId> = SearchSpace::new();
  let mut backward_search: SearchSpace<G::Weight, G::NodeId> = SearchSpace::new();


  for SnappedOnEdge(_, id) in &from.snapped {
    //todo: partial cost and augmented graph are needed to properly initialize the start and end edges
    forward_search.init(*id);
  }

  for SnappedOnEdge(_, id) in &to.snapped {
    //todo: partial cost and augmented graph are needed to properly initialize the start and end edges
    backward_search.init(*id);
  }

  let mut search = BidirectionalSearch::new();

  loop {

    if let Some((node, cost)) = search.route_found(&forward_search, &backward_search) {
      return Some(Route {
                cost: cost,
                // Need to reverse the list to get elements in the routing order
                ids: forward_search.unwind(node).iter()
                  .skip(1) // this id will be in both search spaces
                  .rev()
                  .cloned()
                  .chain(backward_search.unwind(node)).collect(),
                num_resolved: forward_search.num_resolved() + backward_search.num_resolved()
              });
    }

    forward_search.update_and_track::<Forward, _, _>(graph, |node, cost| {
      search.when_forward_relaxed(backward_search.is_settled(node), node, cost);
    });
    
    backward_search.update_and_track::<Backward, _, _>(graph, |node, cost| {
      search.when_bacward_relaxed(forward_search.is_settled(node), node, cost);
    });

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
