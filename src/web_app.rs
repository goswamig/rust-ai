// web_app.rs
use warp::cors;
use warp::ws::{WebSocket, Message};
use std::sync::Arc;
use crate::maze_solver::{MazeSolver, MazeUpdate};
use serde::Serialize;
use tokio::sync::{broadcast, Mutex};

use warp::{Filter, path};
use futures_util::{StreamExt, SinkExt};

pub struct AppState {
    pub solver: Arc<Mutex<MazeSolver>>,
}

// Add a struct for Q-value data serialization
#[derive(Serialize)]
pub struct QTableData {
    state: (usize, usize),
    q_values: Vec<f64>,
}

pub async fn handle_simulation(socket: WebSocket, state: Arc<AppState>, mut update_rx: broadcast::Receiver<MazeUpdate>) {
    println!("web_app.rs: Calling handle_simulation");

    let (mut tx, mut rx) = socket.split();
    tokio::spawn(async move {
        while let Ok(update) = update_rx.recv().await {
            //println!("web_app.rs: Sending update over WebSocket: {:?}", update);
            match serde_json::to_string(&update) {
                Ok(message) => {
                    let message = Message::text(message);
                    if let Err(e) = tx.send(message).await {
                        println!("web_app.rs: Error sending message over WebSocket: {:?}", e);
                        break;
                    }
                },
                Err(e) => println!("web_app.rs: Serialization error: {:?}", e),
            }
        }
    });
    

    while let Some(result) = rx.next().await {
        if let Err(e) = result {
            println!("Error receiving message: {:?}", e);
        }
    }

}


pub fn routes(shared_state: Arc<AppState>, update_tx: broadcast::Sender<MazeUpdate>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    println!("web_app.rs: Calling routes");

        // Clone update_tx for each usage
    let update_tx_for_ws_route = update_tx.clone();
    let update_tx_for_simulate_maze = update_tx.clone();
    let web_dir = warp::path("web").and(warp::fs::dir("./web"));

    let get_maze_state = path!("state")
        .and(warp::get())
        .and(with_state(shared_state.clone()))
        .and_then(|state: Arc<AppState>| async move {
            let solver = state.solver.lock().await;
            let q_values = solver.get_q_values();
            let q_table_data: Vec<QTableData> = solver.get_states().iter().map(|&s| {
                let mut q_values_for_state = Vec::new();
                for &action in solver.get_actions() {
                    let action_num = action as i32;
                    let key = format!("{},{},{}", s.0, s.1, action_num);
                    let q_value = q_values.get(&key).cloned().unwrap_or(0.0);
                    q_values_for_state.push(q_value);
                }
                QTableData {
                    state: s,
                    q_values: q_values_for_state,
                }
            }).collect();
            let maze_data = solver.get_current_state();
            Ok(warp::reply::json(&(maze_data, q_table_data))) as Result<_, warp::Rejection>
    });


    let make_move = path!("maze" / "step")
        .and(warp::post())
        .and(with_state(shared_state.clone()))
        .and_then(|state: Arc<AppState>| async move {
            let mut solver = state.solver.lock().await;
            let game_over = solver.make_move();
            let q_values = solver.get_q_values();
            let q_table_data: Vec<QTableData> = solver.get_states().iter().map(|&s| {
                let mut q_values_for_state = Vec::new();
                for &action in solver.get_actions() {
                    let action_num = action as i32;
                    let key = format!("{},{},{}", s.0, s.1, action_num);
                    let q_value = q_values.get(&key).cloned().unwrap_or(0.0);
                    q_values_for_state.push(q_value);
                }
                QTableData {
                    state: s,
                    q_values: q_values_for_state,
                }
            }).collect();

            let maze_data = solver.get_current_state();

            Ok(warp::reply::json(&(
                maze_data,
                if game_over { "Game over" } else { "" },
                q_table_data
            ))) as Result<_, warp::Rejection>
        });

    
    let reset_maze = path!("maze" / "reset")
        .and(warp::post())
        .and(with_state(shared_state.clone()))
        .and_then(|state: Arc<AppState>| async move {
            let mut solver = state.solver.lock().await;
            solver.reset();
            Ok(warp::reply::json(&solver.get_current_state())) as Result<_, warp::Rejection>
        });

    // ws_route
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(with_state(shared_state.clone()))
        .map(move |ws: warp::ws::Ws, state: Arc<AppState>| {
            let update_tx_clone = update_tx_for_ws_route.clone();
            ws.on_upgrade(move |socket| {
                let update_rx = update_tx_clone.subscribe();
                tokio::spawn(async move {
                    handle_simulation(socket, state, update_rx).await;
                });
                futures_util::future::ready(())
            })
    });

    // simulate_maze route
    let simulate_maze = path!("maze" / "simulate")
        .and(warp::ws())
        .and(with_state(shared_state.clone()))
        .map(move |ws: warp::ws::Ws, state: Arc<AppState>| {
            let update_tx_clone = update_tx_for_simulate_maze.clone();
            ws.on_upgrade(move |socket| {
                println!("simulate_maze: WebSocket connection upgrading");
                let num_subscribers = update_tx_clone.receiver_count();
                println!("simulate_maze: Number of subscribers before new connection: {}", num_subscribers);
                let update_rx = update_tx_clone.subscribe();
                println!("simulate_maze: New subscription made, total subscribers now: {}", update_tx_clone.receiver_count());
                tokio::spawn(async move {
                    let mut solver = state.solver.lock().await;
                    println!("simulate_maze: Starting maze simulation."); // Log the start of the simulation
                    solver.run().await; // Start the maze solving process
                    println!("simulate_maze: Maze simulation completed."); // Log completion of the simulation
                    drop(solver); // Release the lock
                    handle_simulation(socket, state, update_rx).await;
                });
                futures_util::future::ready(())
            })
        });

    let index = warp::path::end().and(warp::fs::file("./web/index.html"));
        // web_dir.or(index).or(get_maze_state).or(make_move).or(reset_maze).or(simulate_maze).or(ws_route)       
        // Combining all routes
        let routes = web_dir
        .or(index)
        .or(get_maze_state)
        .or(make_move)
        .or(reset_maze)
        .or(simulate_maze)
        .or(ws_route);

        routes           
    }

    fn with_state(state: Arc<AppState>) -> impl Filter<Extract = (Arc<AppState>,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || state.clone())
    }
