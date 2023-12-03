// web_app.rs
use warp::cors;

use warp::{Filter, http::Response, path};
use std::sync::{Arc, Mutex};
use crate::maze_solver::MazeSolver;
use serde::Serialize; // Add this import for serialization


pub struct AppState {
    pub solver: Mutex<MazeSolver>,
}

// Add a struct for Q-value data serialization
#[derive(Serialize)]
pub struct QTableData {
    state: (usize, usize),
    q_values: Vec<f64>,
}

pub fn routes(shared_state: Arc<AppState>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let web_dir = warp::path("web").and(warp::fs::dir("./web"));

    let get_maze_state = path!("state")
    .and(warp::get())
    .and(with_state(shared_state.clone()))
    .map(|state: Arc<AppState>| {
        let solver = state.solver.lock().unwrap();

        // Use the getter methods to access states and actions
        let q_values = solver.get_q_values();
        let q_table_data: Vec<QTableData> = solver.get_states().iter().map(|&s| {
            let mut q_values_for_state = Vec::new();
            for &action in solver.get_actions() {
                let q_value = q_values.get(&(s.0, s.1, action)).cloned().unwrap_or(0.0);
                q_values_for_state.push(q_value);
            }
            QTableData {
                state: s,
                q_values: q_values_for_state,
            }
        }).collect();

        let maze_data = solver.get_current_state();
        warp::reply::json(&(maze_data, q_table_data))
    });



    let make_move = path!("maze" / "step")
    .and(warp::post()) // Ensure this is POST
    .and(with_state(shared_state.clone()))
    .map(|state: Arc<AppState>| {
        let mut solver = state.solver.lock().unwrap();
        let game_over = solver.make_move();

        // Generate the Q-table data for serialization
        let q_values = solver.get_q_values();
        let q_table_data: Vec<QTableData> = solver.get_states().iter().map(|&s| {
            let mut q_values_for_state = Vec::new();
            for &action in solver.get_actions() {
                let q_value = q_values.get(&(s.0, s.1, action)).cloned().unwrap_or(0.0);
                q_values_for_state.push(q_value);
            }
            QTableData {
                state: s,
                q_values: q_values_for_state,
            }
        }).collect();

        let maze_data = solver.get_current_state();
        warp::reply::json(&(
            maze_data,
            if game_over { "Game over" } else { "" },
            q_table_data // Include the Q-table data in the response
        ))
    });

    
    let reset_maze = path!("maze" / "reset")
        .and(warp::post())
        .and(with_state(shared_state.clone()))
        .map(|state: Arc<AppState>| {
            let mut solver = state.solver.lock().unwrap();
            solver.reset();
            warp::reply::json(&solver.get_current_state())
        });

    let index = warp::path::end().and(warp::fs::file("./web/index.html"));
    web_dir.or(index).or(get_maze_state).or(make_move).or(reset_maze)        
}

fn with_state(state: Arc<AppState>) -> impl Filter<Extract = (Arc<AppState>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || state.clone())
}
