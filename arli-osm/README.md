This library builds a arli graph from OSM data. 

At the moment it can import [PBF](https://wiki.openstreetmap.org/wiki/PBF_Format) files. You can download any region from [geofabrik](https://download.geofabrik.de/)

To create a graph run:

```
cargo run --release --bin arli-osm -- <your_osm_data>.pbf graph.bin
```