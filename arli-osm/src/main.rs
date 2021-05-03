extern crate arli;

mod graph_builder;
mod graph_serde;
mod osm4routing;

use clap::{value_t_or_exit, App, Arg};
use graph_builder::import_osm_pbf;
use graph_serde::save_graph;
use std::time::Instant;

fn main() {
    let matches = App::new("arli-osm")
        .arg(Arg::with_name("pbf").required(true))
        .arg(Arg::with_name("out").required(true))
        .get_matches();

    let pbf_path = value_t_or_exit!(matches, "pbf", String);

    let out_graph = value_t_or_exit!(matches, "out", String);

    let load_timer = Instant::now();

    let graph = import_osm_pbf(&pbf_path).unwrap();

    println!(
        "Loaded graph with {} nodes and {} edges in {:.2} seconds",
        graph.number_of_nodes(),
        graph.number_of_edges(),
        load_timer.elapsed().as_secs_f32()
    );

    graph.print_stats();

    save_graph(&graph, &out_graph);
}
