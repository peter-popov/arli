mod cost_functions;
mod osrm_api;

use cost_functions::{distance_partial_cost, time_partial_cost};
use arli::waypoint::{match_waypoint};
use arli::route::*;

use arli_osm::{load_graph, OsmGraph};
use osrm_api::*;
use std::sync::Arc;
use std::time::Instant;
use warp::{reject, Filter};

async fn osrm_route_request_handler(
    waypoints: Waypoints,
    graph: Arc<OsmGraph>,
) -> Result<impl warp::Reply, warp::Rejection> {
    println!("OSRM request: {}", waypoints);

    let mut matched_origin = match_waypoint(graph.as_ref(), &waypoints.0[0]);
    if matched_origin.snapped.is_empty() {
        println!("Origin is not matched: {:?}", waypoints.0[0]);
        return Err(reject::not_found());
    }

    let route_timer = Instant::now();

    let mut matched_destination = match_waypoint(graph.as_ref(), &waypoints.0[1]);
    if matched_destination.snapped.is_empty() {
        println!("Destination is not matched: {:?}", waypoints.0[1]);
        return Err(reject::not_found());
    }

    let augmented_graph = connect_waypoints_to_graph(
        graph.as_ref(),
        &mut matched_origin,
        &mut matched_destination,
    );

    let route = route_bidir(
        (&augmented_graph, time_partial_cost),
        &matched_origin,
        &matched_destination,
    );

    if let Some(route) = route {

        let geometry = collect_route_geometry(&augmented_graph, route.ids.iter().cloned());
        let distance = calculate_weight(
            (&augmented_graph, distance_partial_cost),
            route.ids.iter().cloned(),
        );
        let duration = calculate_weight(
            (&augmented_graph, time_partial_cost),
            route.ids.iter().cloned(),
        );

        println!("Route found in {}s: cost = {:?}, distance = {:?}, duration = {:?}, nodes = {:?}",
            route_timer.elapsed().as_secs_f32(), 
            route.cost, distance, duration, route.num_resolved);

        let response = OsrmRouteResponse::new(geometry, distance, duration, route.cost, &waypoints);
        return Ok(warp::reply::json(&response));
    }

    println!("No route found");
    return Err(reject::not_found());
}

#[tokio::main]
async fn main() {
    let startup_timer = Instant::now();

    let graph = Arc::new(load_graph("graph.bin").unwrap());
    println!(
        "Loaded graph with {} nodes and {} edges in {:.1} seconds",
        graph.number_of_nodes(),
        graph.number_of_edges(),
        startup_timer.elapsed().as_secs_f32()
    );
    graph.print_stats();

    let graph = warp::any().map(move || Arc::clone(&graph));

    let cors = warp::cors().allow_any_origin();

    let route_api = warp::path("route")
        .and(warp::path("v1"))
        .and(warp::path("driving"))
        .and(warp::path::param::<Waypoints>())
        .and(warp::path::end())
        .and(graph.clone())
        .and_then(osrm_route_request_handler)
        .with(cors);

    let frontend = warp::path("frontend").and(warp::fs::dir("frontend"));

    println!("Started service with the bind address 127.0.0.1:5000");
    warp::serve(route_api.or(frontend))
        .run(([127, 0, 0, 1], 5000))
        .await;
}
