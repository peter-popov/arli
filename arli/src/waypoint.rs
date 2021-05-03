//! Waypoint matching.

use crate::graph::{Identifier, IntoGeometry, Spatial};
use crate::spatial::*;
use geo::{Closest, closest_point::*, haversine_distance::*, line_locate_point::*};
use std::fmt;

#[derive(Copy, Clone)]
pub struct SnappedPosition {
  pub snapped: Position,
  pub distance: f32,
  pub factor: f32,
}

pub struct SnappedOnEdge<N: Identifier>(pub SnappedPosition, pub N);

pub struct MatchedWaypoint<N: Identifier> {
  pub waypoint: Position,
  pub snapped: Vec<SnappedOnEdge<N>>,
}

fn snap_to_geometry(
  geometry: &Polyline,
  position: &Position,
  max_distance: f32,
) -> Option<SnappedPosition> {
  let position = geo::Point::from(*position);
  match geometry.closest_point(&position) {
    Closest::SinglePoint(closest_point) => {
      let distance = position.haversine_distance(&closest_point);
      if distance < max_distance {
        let factor = geometry.line_locate_point(&closest_point).unwrap();
        return Some(SnappedPosition {
          snapped: closest_point.0,
          distance: distance,
          factor: factor,
        });
      }
      return None;
    }
    Closest::Intersection(point_on_line) => {
      return Some(SnappedPosition {
        snapped: point_on_line.0,
        distance: 0.0,
        factor: geometry.line_locate_point(&point_on_line).unwrap(),
      })
    }
    Closest::Indeterminate => return None,
  }
}

pub fn match_waypoint<G: Copy + IntoGeometry + Spatial>(
  graph: G,
  waypoint: &Position,
) -> MatchedWaypoint<G::NodeId> {
  let elements_nearby = graph.find_nodes(&envelope(waypoint, 100.));

  let mut snapped_positions: Vec<_> = elements_nearby
    .into_iter()
    // TODO: Rtree does not seem to work, returns too many elements
    //.inspect(|x| println!(" > found nearby: {}", x))
    .filter_map(|id| {
      snap_to_geometry(
        &Polyline::from(graph.geometry(id).collect::<Vec<_>>()),
        waypoint,
        100.0,
      )
      .map(|snapped| SnappedOnEdge(snapped, id))
    })
    .collect();

  snapped_positions.sort_by(|a, b| a.0.distance.partial_cmp(&b.0.distance).unwrap());
  snapped_positions.truncate(4);

  MatchedWaypoint {
    waypoint: *waypoint,
    snapped: snapped_positions,
  }
}

impl fmt::Debug for SnappedPosition {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{{({}, {}), d = {}, f = {}}}",
      self.snapped.x, self.snapped.y, self.distance, self.factor
    )
  }
}

impl<N: Identifier> fmt::Debug for SnappedOnEdge<N> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{{id = {:?}, snapped = {:?}}}", self.1, self.0)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use geo::{map_coords::MapCoords};
  use geo::point;


  #[test]
  fn test_snap_to_geometry() {
    let ref_pos = point!(x: 13.34, y: 52.46).0;
    let offsets: Polyline = vec![[0.002, 0.0], [0.004, 0.005], [0.0, 0.009]].into();
    let geometry: Polyline = offsets.map_coords(|&(x, y)| (ref_pos.x + x, ref_pos.y + y));

    let result0 = snap_to_geometry(&geometry, &ref_pos, 200.0);
    println!("result0 = {:?}", result0.unwrap());

    let result1 = snap_to_geometry(
      &geometry,
      &point!(x: ref_pos.x + 0.005, y: ref_pos.y + 0.002).0,
      200.0,
    );
    println!("result1 = {:?}", result1.unwrap());

    let result2 = snap_to_geometry(
      &geometry,
      &point!(x: ref_pos.x + 0.002, y: ref_pos.y + 0.007).0,
      200.0,
    );
    println!("result2 = {:?}", result2.unwrap());
  }
}
