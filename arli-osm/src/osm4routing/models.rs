use osmpbfreader::objects::{NodeId, WayId};
use super::categorize::EdgeProperties;
use geo::{Coordinate, LineString, haversine_length::*};


// Coord are coordinates in decimal degress WGS84
pub type Coord = Coordinate<f32>;
pub type Geometry = LineString<f32>;

// Node is the OpenStreetMap node
#[derive(Copy, Clone)]
pub struct Node {
    pub id: NodeId,
    pub coord: Coord,
    pub uses: i16,
}

impl Default for Node {
    fn default() -> Node {
        Node {
            id: NodeId(0),
            coord: Coord{x:0.0, y:0.0},
            uses: Default::default(),
        }
    }
}

impl Node {
    pub fn set_coord(&mut self, lon: f32, lat: f32) {
        self.coord.x = lon;
        self.coord.y = lat;
    }
}

// Edge is a topological representation with only two extremities and no geometry
pub struct Edge {
    pub id: WayId,
    pub source: NodeId,
    pub target: NodeId,
    pub geometry: Geometry,
    pub properties: EdgeProperties,
}

impl Edge {
    // Length in meters of the edge
    pub fn length(&self) -> f32 { self.geometry.haversine_length()}
}
