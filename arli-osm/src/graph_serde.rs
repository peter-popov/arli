use crate::graph_builder::OsmGraph;
use std::fs::File;
use std::io::{ErrorKind, Error, BufWriter, BufReader};

pub fn save_graph(graph: &OsmGraph, path: &str){
  let file = BufWriter::new(File::create(path).unwrap());
  bincode::serialize_into(file, &graph).unwrap();
}


pub fn load_graph(path: &str) -> std::io::Result<OsmGraph> {
  let file = BufReader::new(File::open(path)?);
  let mut graph: OsmGraph = bincode::deserialize_from(file).map_err(|_| Error::from(ErrorKind::InvalidData))?;
  graph.shrink();
  Ok(graph)
}