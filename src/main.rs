use warp::Filter;
use tokio::sync::{broadcast, Mutex};
use std::sync::Arc;
use web_app::{routes, AppState};
mod maze_solver;
mod web_app;


#[tokio::main]
async fn main() {
    println!("main.rs: Calling main");

    let (update_tx, _) = broadcast::channel(10000); // Change to broadcast channel based on number of updates count
    let solver = maze_solver::MazeSolver::new(update_tx.clone()); 
    let state = AppState {
        solver: Arc::new(Mutex::new(solver)),
    };
    let shared_state = Arc::new(state);

    let update_tx_clone = update_tx.clone();

    // Serve static files from the "web" directory
    let web_dir = warp::path("web").and(warp::fs::dir("./web"));

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["*"])
        .allow_methods(vec!["GET", "POST", "DELETE", "PUT", "OPTIONS"]);

    let routes = routes(shared_state.clone(), update_tx).with(cors);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

