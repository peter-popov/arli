use super::categorize::*;
use super::models::*;
use osmpbfreader::objects::{NodeId, WayId};
use std::collections::HashMap;
use std::io::Read;
use std::time::Instant;

// Way as represented in OpenStreetMap
struct Way {
    id: WayId,
    nodes: Vec<NodeId>,
    properties: EdgeProperties,
}

struct Reader {
    nodes: HashMap<NodeId, Node>,
    ways: Vec<Way>,
}

impl Reader {
    fn new() -> Reader {
        Reader {
            nodes: HashMap::new(),
            ways: Vec::new(),
        }
    }

    fn split_way(&self, way: &Way) -> Vec<Edge> {
        let mut result = Vec::new();

        let mut source = NodeId(0);
        let mut points = Vec::new();
        for (i, &node_id) in way.nodes.iter().enumerate() {
            let node = self.nodes[&node_id];
            if i == 0 {
                source = node_id;
                points.push(node.coord);
            } else {
                points.push(node.coord);

                if node.uses > 1 || i == way.nodes.len() - 1 {
                    result.push(Edge {
                        id: way.id,
                        source,
                        target: node_id,
                        geometry: points.into(),
                        properties: way.properties,
                    });

                    source = node_id;
                    points = vec![node.coord];
                }
            }
        }
        result
    }

    fn read_ways<R: Read>(&mut self, pbf: &mut osmpbfreader::OsmPbfReader<R>) {
        for obj in pbf.par_iter() {
            if let Ok(osmpbfreader::OsmObj::Way(way)) = obj {
                let mut properties = EdgeProperties::default();
                for (key, val) in way.tags.iter() {
                    properties.update(key.as_str(), val.as_str());
                }
                properties.normalize();
                if properties.accessible() {
                    for node in &way.nodes {
                        self.nodes.entry(*node).or_insert(Node::default()).uses += 1;
                    }
                    self.ways.push(Way {
                        id: way.id,
                        nodes: way.nodes,
                        properties,
                    });
                }
            }
        }
    }

    fn read_nodes<R: Read>(&mut self, pbf: &mut osmpbfreader::OsmPbfReader<R>) {
        for obj in pbf.par_iter() {
            if let Ok(osmpbfreader::OsmObj::Node(node)) = obj {
                self.nodes.entry(node.id).and_modify(|mut_node| {
                    mut_node.set_coord(node.lon() as f32, node.lat() as f32)
                });
            }
        }
    }

    fn edges(&self) -> Vec<Edge> {
        self.ways
            .iter()
            .flat_map(|way| self.split_way(way))
            .collect()
    }
}

pub fn read_edges(filename: &str) -> Result<Vec<Edge>, String> {
    let mut r = Reader::new();
    let file = std::fs::File::open(filename).map_err(|e| e.to_string())?;
    let mut pbf = osmpbfreader::OsmPbfReader::new(file);

    {
        let t = Instant::now();
        r.read_ways(&mut pbf);
        println!("Decoded ways {:.2}s", t.elapsed().as_secs_f32());
    }
    {
        let t = Instant::now();
        pbf.rewind().map_err(|e| e.to_string())?;
        r.read_nodes(&mut pbf);
        println!("Decoded nodes {:.2}s", t.elapsed().as_secs_f32());
    }
    let t = Instant::now();
    let edges = r.edges();
    println!("Split ways {:.2}s", t.elapsed().as_secs_f32());

    Ok(edges)
}
