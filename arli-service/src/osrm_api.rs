use arli::spatial::{Position, Coordinate};
use arli::graph::Weight;
use polyline::encode_coordinates;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Deserialize, Serialize)]
struct OsrmWaypoint {
  distance: f32,
  location: Vec<f32>,
}

impl OsrmWaypoint {
  pub fn from(p: &Position) -> Self {
    OsrmWaypoint {
      distance: 0.0,
      location: vec![p.x, p.y],
    }
  }
}

#[derive(Deserialize, Serialize)]
struct OsrmLeg {
  weight: f64,
  distance: f64,
  summary: String,
  duration: f64,
  steps: Vec<String>,
}

#[derive(Deserialize, Serialize)]
struct OsrmRoute {
  distance: f64,
  duration: f64,
  geometry: String,
  legs: Vec<OsrmLeg>,
}

#[derive(Deserialize, Serialize)]
pub struct OsrmRouteResponse {
  code: String,
  routes: Vec<OsrmRoute>,
  waypoints: Vec<OsrmWaypoint>,
}

#[derive(Debug)]
pub struct RequestError(String);

impl RequestError {
  pub fn with(s: &str) -> Self {
    RequestError(String::from(s))
  }
}

pub struct Waypoints(pub Vec<Position>);

impl FromStr for Waypoints {
  type Err = RequestError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut result = Vec::new();
    for coord_str in s.split(';') {
      let coords: Vec<_> = coord_str.split(',').map(|s| s.parse::<f32>()).collect();
      if coords.len() != 2 {
        return Err(RequestError::with(
          "Each waypoint must have two coordinates",
        ));
      };
      let coords: Vec<_> = coords.iter().filter_map(|r| r.as_ref().ok()).collect();
      if coords.len() != 2 {
        return Err(RequestError(format!(
          "Error one the coordinates for {}",
          coord_str
        )));
      };
      result.push(Position::from((*coords[0], *coords[1])));
    }
    if result.len() != 2 {
      return Err(RequestError(format!(
        "Expect exactly 2 waypoints, {} found",
        result.len()
      )));
    }
    Ok(Waypoints(result))
  }
}

impl fmt::Display for Waypoints {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    for p in &self.0 {
      write!(f, "{}, {}", p.x, p.y)?
    }
    Ok(())
  }
}

fn map_coordinates<P: Into<Position>>(p: P) -> Coordinate<f64> {
  let pp: Position = p.into();
  Coordinate::<f64>::from((pp.x as f64, pp.y as f64))
}

impl OsrmRouteResponse {
  pub fn new<P:Into<Position>, Geometry: IntoIterator<Item = P>, W: Weight + Into<f64>>(
    geometry: Geometry,
    route_distance: W,
    route_duration: W,
    cost: W,
    waypoints: &Waypoints,
  ) -> OsrmRouteResponse {
    let route = OsrmRoute {
      duration: route_duration.into(),
      distance: route_distance.into(),
      geometry: encode_coordinates(geometry.into_iter().map(map_coordinates), 5).unwrap(),
      legs: vec![OsrmLeg {
        weight: cost.into(),
        distance: route_distance.into(),
        summary: String::from("test"),
        duration: route_duration.into(),
        steps: vec![],
      }],
    };

    OsrmRouteResponse {
      code: String::from("Ok"),
      routes: vec![route],
      waypoints: vec![
        OsrmWaypoint::from(&waypoints.0[0]),
        OsrmWaypoint::from(&waypoints.0[1]),
      ],
    }
  }
}
