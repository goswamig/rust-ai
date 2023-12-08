use std::collections::HashMap;
use rand::{Rng, seq::SliceRandom};
use rand::rngs::StdRng;
use rand::SeedableRng;
use serde::{Serialize, Deserialize};
use tokio::sync::{mpsc, Mutex};
use tokio::sync::broadcast;
use rand::seq::IteratorRandom;



// Environment settings   
pub const GRID_SIZE: usize = 5;

// Hyperparameters
const ALPHA: f64 = 0.1;
const GAMMA: f64 = 0.9;
const EPSILON: f64 = 0.1;
const EPISODES: usize = 1000;

#[derive(Serialize, Deserialize, Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum Action {
    Up, Down, Left, Right,
}

// // Define a struct for updates
// #[derive(Serialize, Clone, Debug)]
// pub struct MazeUpdate {
//     pub current_state: HashMap<String, Vec<(usize, usize)>>,
//     pub q_table: HashMap<(usize, usize, Action), f64>,
// }

#[derive(Serialize, Clone, Debug)]
pub struct MazeUpdate {
    pub current_state: HashMap<String, Vec<(usize, usize)>>,
    pub q_table: HashMap<String, f64>,
}

pub struct MazeSolver {
    q_table: HashMap<((usize, usize), Action), f64>,
    pub states: Vec<(usize, usize)>,
    pub actions: Vec<Action>,
    pub obstacles: Vec<(usize, usize)>,
    pub goal: (usize, usize),
    rng: StdRng,
    pub current_state: (usize, usize),
    pub path: Vec<(usize, usize)>, // Track the path taken by the agent
    pub update_tx: broadcast::Sender<MazeUpdate>,  // change here
}

impl MazeSolver {
    pub fn new(update_tx: broadcast::Sender<MazeUpdate>) -> Self {
        println!("maze_solver.rs: Calling new");
        let states: Vec<(usize, usize)> = (0..GRID_SIZE)
            .flat_map(|x| (0..GRID_SIZE).map(move |y| (x, y)))
            .collect();
        let actions = vec![Action::Up, Action::Down, Action::Left, Action::Right];
        let obstacles = vec![(1, 1), (2, 2), (3, 3)];
        let goal = (4, 4);
        let rng = StdRng::from_entropy(); // Initialize a thread-safe RNG

        let mut q_table: HashMap<((usize, usize), Action), f64> = HashMap::new();
        for &state in &states {
            for &action in &actions {
                q_table.insert((state, action), 0.0);
            }
        }

        MazeSolver {
            q_table,
            states,
            actions,
            obstacles,
            goal,
            rng: StdRng::from_entropy(),
            current_state: (0, 0), // Initialize current_state
            path: Vec::new(), // Initialize path
            update_tx,
        }
    }

    // Add a getter method for states
    pub fn get_states(&self) -> &Vec<(usize, usize)> {
        println!("maze_solver.rs: Calling get_states");

        &self.states
    }

        // Add a public method to access update_tx if needed
        pub fn get_update_tx(&self) -> &broadcast::Sender<MazeUpdate> {
            println!("maze_solver.rs: Calling get_update_tx");

            &self.update_tx
        }

    // Add a getter method for actions
    pub fn get_actions(&self) -> &Vec<Action> {
        //println!("maze_solver.rs: Calling get_actions");

        &self.actions
    }

    pub async fn run(&mut self) {
        println!("maze_solver.rs: Calling run");
        //tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        // Send a static update before starting the algorithm
        // let static_update = MazeUpdate {
        //     current_state: self.generate_static_current_state(), // Call a function to generate static current state
        //     q_table: self.generate_static_q_table(), // Call a function to generate static q_table
        // };
        // if let Err(e) = self.update_tx.send(static_update) {
        //     println!("Failed to send static update: {:?}", e);
        // }

        for _ in 0..EPISODES {
            self.current_state = (0, 0); // Start state
            while self.current_state != self.goal {    
                let action = if self.rng.gen::<f64>() < EPSILON {
                    *self.actions.choose(&mut self.rng).unwrap()
                } else {
                    *self.actions.iter().max_by(|&&a1, &&a2| self.q_table[&(self.current_state, a1)].partial_cmp(&self.q_table[&(self.current_state, a2)]).unwrap()).unwrap()
                };

                let next_state = self.get_next_state(self.current_state, action);

                let reward = if next_state == self.goal {
                    100.0
                } else if self.obstacles.contains(&next_state) {
                    -50.0
                } else {
                    -1.0
                };

                let next_max = self.actions.iter().map(|&a| self.q_table[&(next_state, a)]).fold(f64::MIN, f64::max);

                let q_value = self.q_table.get_mut(&(self.current_state, action)).unwrap();
                *q_value += ALPHA * (reward + GAMMA * next_max - *q_value);

                self.current_state = next_state;

                // After making a move, send an update
                let update = MazeUpdate {
                    current_state: self.get_current_state(),
                    q_table: self.get_q_values(),
                };

                //Store the return value of send and print it
                // let send_result = self.update_tx.send(update);
                // match send_result {
                //     Ok(num_receivers) => println!("Update sent to {} receivers", num_receivers),
                //     Err(e) => println!("Failed to send update: {:?}", e),
                // }
                //println!("maze_solver.rs: Sending update for state {:?}", self.current_state);
                let _ = self.update_tx.send(update);  // change here, broadcast channels don't need await

                 //delay after sending each update        
                 //tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

             }
         }
    }


    // Function to generate a random current state for testing
    fn generate_static_current_state(&self) -> HashMap<String, Vec<(usize, usize)>> {
        let mut rng = rand::thread_rng();
        let mut state = HashMap::new();

        let agent_position = self.states
            .iter()
            .filter(|&&pos| !self.obstacles.contains(&pos) && pos != self.goal)
            .choose(&mut rng)
            .unwrap_or(&(0, 0));

        state.insert("agent".to_string(), vec![*agent_position]);
        state.insert("obstacles".to_string(), self.obstacles.clone());
        state.insert("goal".to_string(), vec![self.goal]);
        state.insert("path".to_string(), vec![]); // Empty path for simplicity
        state
    }

    // Function to generate random q_table values for testing
    fn generate_static_q_table(&self) -> HashMap<String, f64> {
        let mut rng = rand::thread_rng();
        let mut q_table = HashMap::new();

        for &state in &self.states {
            for &action in &self.actions {
                let value = rng.gen_range(0.0..100.0);
                let key = format!("{},{},{}", state.0, state.1, action as i32); // Convert to string key
                q_table.insert(key, value);
            }
        }
        q_table
    }


    pub fn reset(&mut self) {
        println!("maze_solver.rs: Calling reset");

        self.current_state = (0, 0); // Reset to start
        self.path.clear(); // Clear the path
    }

    pub fn get_current_state(&self) -> HashMap<String, Vec<(usize, usize)>> {
        //println!("maze_solver.rs: Calling get_current_state");

        let mut state = HashMap::new();
        state.insert("agent".to_string(), vec![self.current_state]);
        state.insert("obstacles".to_string(), self.obstacles.clone());
        state.insert("goal".to_string(), vec![self.goal]);
        state.insert("path".to_string(), self.path.clone());
        //println!("Current State: {:?}", state);  // Logging the current state

        state
    }

    pub fn make_move(&mut self) -> bool  {
        println!("maze_solver.rs: Calling make_move");

        // Make a single move
        if self.current_state == self.goal {
            return true; // Reached the goal
        }

        let action = if self.rng.gen::<f64>() < EPSILON {
            *self.actions.choose(&mut self.rng).unwrap()
        } else {
            *self.actions.iter().max_by(|&&a1, &&a2| self.q_table[&(self.current_state, a1)].partial_cmp(&self.q_table[&(self.current_state, a2)]).unwrap()).unwrap()
        };

        let next_state = self.get_next_state(self.current_state, action);

        // ... (rest of the Q-learning update logic)
        let reward = if next_state == self.goal {
            100.0
        } else if self.obstacles.contains(&next_state) {
            -50.0
        } else {
            -1.0
        };

        let next_max = self.actions.iter().map(|&a| self.q_table[&(next_state, a)]).fold(f64::MIN, f64::max);

        let q_value = self.q_table.get_mut(&(self.current_state, action)).unwrap();
        *q_value += ALPHA * (reward + GAMMA * next_max - *q_value);

        self.current_state = next_state; // Update current_state
        self.path.push(next_state); // Add to path
        println!("Updated Q-value for state {:?} and action {:?}: {}", self.current_state, action, *q_value);

        return false; 
    }

    // pub fn get_q_values(&self) -> HashMap<(usize, usize, Action), f64> {
    //     //println!("maze_solver.rs: Calling get_q_values");

    //     self.states.iter()
    //         .flat_map(|&state| {
    //             self.actions.iter().map(move |&action| {
    //                 ((state.0, state.1, action), *self.q_table.get(&((state, action))).unwrap_or(&0.0))
    //             })
    //         })
    //         .collect()
    // }
    
    pub fn get_q_values(&self) -> HashMap<String, f64> {
        let mut q_values = HashMap::new();

        for &state in &self.states {
            for &action in &self.actions {
                // Convert action to i32 for string representation
                let action_num = match action {
                    Action::Up => 0,
                    Action::Down => 1,
                    Action::Left => 2,
                    Action::Right => 3,
                };

                let key = format!("{},{},{}", state.0, state.1, action_num);
                let value = *self.q_table.get(&((state, action))).unwrap_or(&0.0);
                q_values.insert(key, value);
            }
        }

        q_values
    }



    fn get_next_state(&self, state: (usize, usize), action: Action) -> (usize, usize) {
        //println!("maze_solver.rs: Calling get_next_state");

        let (mut x, mut y) = state;
        match action {
            Action::Up => x = x.saturating_sub(1),
            Action::Down => x = usize::min(x + 1, GRID_SIZE - 1),
            Action::Left => y = y.saturating_sub(1),
            Action::Right => y = usize::min(y + 1, GRID_SIZE - 1),
        }
        let next_state = (x, y);
        return next_state
    //comment previous line and uncommemt below if you let agent gotio obstacle
        // if self.obstacles.contains(&next_state) {
        //     state // Return current state if next state is an obstacle
        // } else {
        //     next_state
        // }
    }

    // Additional methods or helper functions can be added here if needed
}
