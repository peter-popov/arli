use crate::osm4routing::{read_edges, Edge as OsmEdge};
use arli::graph_impl::{CompactGraph, CompactSpatialGraph};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Segment {
  pub length: f32,
  pub speed_limit: u8,
}

pub type OsmGraph = CompactSpatialGraph<Segment>;

pub fn import_osm_pbf(pbf_path: &str) -> Result<OsmGraph, String> {
  let edges = read_edges(pbf_path)?;

  Ok(build_compact_graph(&edges))
}

pub fn build_compact_graph(osm_edges: &Vec<OsmEdge>) -> OsmGraph {
  let mut segments: Vec<Segment> = Vec::new();
  let mut target_nodes: Vec<usize> = Vec::new();
  let mut out_segments: HashMap<usize, Vec<u32>> = HashMap::new();

  let mut points = Vec::new();
  let mut geom_offsets = Vec::new();

  points.push(geo::Coordinate::from((0.0f32, 0.0f32))); // Sentinel for backward range
  for record in osm_edges {
    if record.properties.car_forward != 0 {
      geom_offsets.push((points.len(), points.len() + record.geometry.num_coords()));
      points.extend(record.geometry.0.iter());

      let forward = Segment {
        length: record.length(),
        speed_limit: record.properties.speed_limit_km_h,
      };
      out_segments
        .entry(record.source.0 as usize)
        .or_insert_with(|| Vec::new())
        .push(segments.len() as u32);
      target_nodes.push(record.target.0 as usize);
      segments.push(forward);
    }

    if record.properties.car_backward != 0 {
      // We reuse coordinates for the edge in the opposite direction. Create a range (before, last]
      geom_offsets.push((
        points.len() - 1,
        points.len() - record.geometry.num_coords() - 1,
      ));

      let backward = Segment {
        length: record.length(),
        speed_limit: record.properties.speed_limit_km_h,
      };
      out_segments
        .entry(record.target.0 as usize)
        .or_insert_with(|| Vec::new())
        .push(segments.len() as u32);
      target_nodes.push(record.source.0 as usize);
      segments.push(backward);
    }
  }
  let mut edge_refs: Vec<u32> = Vec::new();
  let mut edge_offsets = Vec::new();

  for target_id in &target_nodes {
    edge_offsets.push(edge_refs.len());
    if let Some(targets) = out_segments.get(target_id) {
      edge_refs.extend(targets);
    }
  }

  let mut graph = OsmGraph::from_row_data(
    CompactGraph::from_row_data(segments, edge_offsets, edge_refs),
    geom_offsets,
    points,
  );
  graph.shrink();
  graph
}
