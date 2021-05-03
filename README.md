# arli - Abstract Routing LIbrary

**arli** is a Rust libray which provides generic building blocks for route planning in the road netwroks. It provides abstract primites for representing road graphs and implemention of major algithms like road snapping, path matching, one-to-one and matrix routings.

## Preparing OSM data

The service loads `graph.bin` from root folder on startup. First you need to [download]((https://download.geofabrik.de/)) the OSM file for your region. 
Then you can create a routing graph by using the following command:

```
cargo run --bin arli-osm -- <your_osm_data>.pbf graph.bin
```

## Running the service  

```
> cargo run --bin arli-service 
...
Imported graph with 398441 nodes and 988200 edges
Started service with the bind address 127.0.0.1:5000
```

## Running the frontend

arli service also comes with a simple web frontend. Once the service is running  simply open [http://localhost:5000/frontend/index.html](http://localhost:5000/frontend/index.html) in your browser.
You should see the map and route controls. Fronted send requests to `127.0.0.1:5000`
