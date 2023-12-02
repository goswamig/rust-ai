use warp::Filter;
use warp::cors;

use std::sync::{Arc, Mutex};
use warp::http::Response;
use serde::Serialize; // Add this import for serialization

mod maze_solver;
mod web_app;
use web_app::{routes, AppState};

#[tokio::main]
async fn main() {
    let solver = maze_solver::MazeSolver::new();
    let state = AppState {
        solver: Mutex::new(solver),
    };
    let shared_state = Arc::new(state);

    // Serve static files from the "web" directory
    let web_dir = warp::path("web").and(warp::fs::dir("./web"));

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["*"])
        .allow_methods(vec!["GET", "POST", "DELETE", "PUT", "OPTIONS"]);

    let routes = routes(shared_state.clone()).with(cors);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

