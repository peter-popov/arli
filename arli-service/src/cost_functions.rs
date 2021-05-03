use arli::waypoint::SnappedPosition;
use arli_osm::Segment;

pub fn distance_cost(from: &Segment, _to: &Segment) -> i32 {
  from.length as i32
}

pub fn time_cost(from: &Segment, _to: &Segment) -> i32 {
  if from.speed_limit > 0 {
    (from.length * 3.6 / from.speed_limit as f32) as i32
  } else {
    3600
  }
}

pub fn distance_partial_cost(
  from: &Segment,
  _to: &Segment,
  snapped: Option<SnappedPosition>,
) -> i32 {
  let (factor, distance) = snapped
    .map(|s| (s.factor, s.distance))
    .unwrap_or((1.0, 0.0));
  (from.length * factor as f32 + distance * 1.4) as i32
}

pub fn time_partial_cost(from: &Segment, _to: &Segment, snapped: Option<SnappedPosition>) -> i32 {
  let (factor, distance) = snapped
    .map(|s| (s.factor, s.distance))
    .unwrap_or((1.0, 0.0));

  if from.speed_limit > 0 {
    // Assume pedestrian speed of 4 km/h for the distance to matched waypoint
    (from.length * 3.6 * factor as f32 / from.speed_limit as f32 + distance * 3.6 / 4.0) as i32
  } else {
    3600
  }
}
