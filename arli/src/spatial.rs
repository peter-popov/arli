//! Geographic types.

use geo::{
  haversine_destination::HaversineDestination, haversine_distance::*,
  line_locate_point::LineLocatePoint,
};

use geo::{LineString, Point, Rect};
use s2::{cellid::CellID, latlng::LatLng, s1::angle::*};

#[doc(hidden)]
pub use geo::Coordinate;

pub type Position = Coordinate<f32>;

pub type Polyline = LineString<f32>;

pub type BoundingBox = Rect<f32>;


pub fn envelope(center: &Position, distance_m: f32) -> BoundingBox {
  let center_point = Point::from(*center);
  let right = center_point.haversine_destination(0., distance_m).0;
  let top = center_point.haversine_destination(90., distance_m).0;

  BoundingBox::new(
    Coordinate {
      x: 2.0 * center.x - right.x,
      y: 2.0 * center.y - top.y,
    },
    Coordinate {
      x: right.x,
      y: top.y,
    },
  )
}

pub fn haversine_distance(from: &Position, to: &Position) -> f32 {
  //We copy coords here :(. Need to figure out something better
  Point::from(*from).haversine_distance(&Point::from(*to))
}

pub fn bounding_box<P: Iterator<Item = Position>>(points: P) -> Option<BoundingBox> {
  let mut extremes: Option<(Position, Position)> = None;
  for p in points {
    let (bl, tr) = extremes.get_or_insert((p, p));
    bl.x = bl.x.min(p.x);
    bl.y = bl.y.min(p.y);
    tr.x = tr.x.max(p.x);
    tr.y = tr.y.max(p.y);
  }

  extremes.map(|e| BoundingBox::new(e.0, e.1))
}

fn to_s2_latlng(p: &Position) -> s2::latlng::LatLng {
  LatLng::new(Angle::from(Deg(p.y as f64)), Angle::from(Deg(p.x as f64)))
}

pub fn to_s2(p: &Position) -> CellID {
  CellID::from(to_s2_latlng(p))
}

pub fn s2_cover(rect: &BoundingBox, level: u8) -> s2::cellunion::CellUnion {
  let center = to_s2_latlng(&rect.center());
  let size = to_s2_latlng(&Position {
    x: rect.width(),
    y: rect.height(),
  });
  let coverer = s2::region::RegionCoverer {
    min_level: level,
    max_level: level,
    level_mod: 1,
    max_cells: 100,
  };

  coverer.covering(&s2::rect::Rect::from_center_size(center, size))
}

/**
 * This function returns partial geometry cut at a specific point.
 * @TODO: this implementation is incorrect :(
 */
pub fn cut_geometry_before<T: Into<Position>, Geometry: Iterator<Item = T>>(
  geometry: Geometry,
  point: Position,
) -> Vec<Position> {
  let line_string: Polyline = geometry.collect();

  let remaining_segments = line_string.lines().skip_while(|line| {
    if let Some(factor) = line.line_locate_point(&geo::Point::from(point)) {
      return factor >= 1.0;
    }
    false
  });

  let mut result = vec![point];
  result.extend(remaining_segments.map(|line| line.end_point().0));
  result
}

pub fn cut_geometry_after<T: Into<Position>, Geometry: Iterator<Item = T>>(
  geometry: Geometry,
  point: Position,
) -> Vec<Position> {
  let line_string: Polyline = geometry.collect();

  let mut result: Vec<Position> = line_string
    .lines()
    .take_while(|line| {
      line
        .line_locate_point(&geo::Point::from(point))
        .map_or(false, |factor| factor >= 1.0)
    })
    .map(|line| line.start_point().0)
    .collect();

  result.push(point);
  result
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_cut_geometry_before() {
    let coordinates: Vec<Position> = vec![
      [-122.4005270, 37.7890733],
      [-122.4003553, 37.7891921],
      [-122.4001461, 37.7893489],
      [-122.3996579, 37.7897474],
      [-122.3993843, 37.7899763],
      [-122.3991322, 37.7897898],
    ]
    .iter()
    .map(|v| Position::from(*v))
    .collect();

    let cut_at = Position {
      x: -122.3998698,
      y: 37.78952064,
    };
    let result = cut_geometry_before(coordinates.iter().cloned(), cut_at);

    assert_eq!(result.len(), 4);

    assert_eq!(result[0], cut_at);
    assert_eq!(result[1], coordinates[3]);
    assert_eq!(result[2], coordinates[4]);
  }

  #[test]
  fn test_cut_geometry_after() {
    let coordinates: Vec<Position> = vec![
      [-122.4005270, 37.7890733],
      [-122.4003553, 37.7891921],
      [-122.4001461, 37.7893489],
      [-122.3996579, 37.7897474],
      [-122.3993843, 37.7899763],
      [-122.3991322, 37.7897898],
    ]
    .iter()
    .map(|v| Position::from(*v))
    .rev() //Reversed
    .collect();

    let cut_at = Position {
      x: -122.3998698,
      y: 37.78952064,
    };
    let result = cut_geometry_after(coordinates.iter().cloned(), cut_at);

    assert_eq!(result.len(), 3);

    assert_eq!(result[0], coordinates[0]);
    assert_eq!(result[1], coordinates[1]);
    assert_eq!(result[2], cut_at);
  }
}
