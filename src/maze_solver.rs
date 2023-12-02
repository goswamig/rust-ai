use std::collections::HashMap;
use rand::{Rng, seq::SliceRandom};
// In maze_solver.rs
use rand::rngs::StdRng;
use rand::{SeedableRng};

// Environment settings   
pub const GRID_SIZE: usize = 5;

// Hyperparameters
const ALPHA: f64 = 0.1;
const GAMMA: f64 = 0.9;
const EPSILON: f64 = 0.1;
const EPISODES: usize = 1000;

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
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

}

impl MazeSolver {
    pub fn new() -> Self {
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

        }
    }

    // Add a getter method for states
    pub fn get_states(&self) -> &Vec<(usize, usize)> {
        &self.states
    }

    // Add a getter method for actions
    pub fn get_actions(&self) -> &Vec<Action> {
        &self.actions
    }

    pub fn run(&mut self) {
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
            }
        }
    }

    pub fn reset(&mut self) {
        self.current_state = (0, 0); // Reset to start
        self.path.clear(); // Clear the path
    }

    pub fn get_current_state(&self) -> HashMap<String, Vec<(usize, usize)>> {
        let mut state = HashMap::new();
        state.insert("agent".to_string(), vec![self.current_state]);
        state.insert("obstacles".to_string(), self.obstacles.clone());
        state.insert("goal".to_string(), vec![self.goal]);
        state.insert("path".to_string(), self.path.clone());
        //println!("Current State: {:?}", state);  // Logging the current state

        state
    }

    pub fn make_move(&mut self) -> bool  {
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
        return false; 
    }

    pub fn get_q_values(&self) -> HashMap<(usize, usize, Action), f64> {
        self.states.iter()
            .flat_map(|&state| {
                self.actions.iter().map(move |&action| {
                    ((state.0, state.1, action), *self.q_table.get(&((state, action))).unwrap_or(&0.0))
                })
            })
            .collect()
    }
    
    


    fn get_next_state(&self, state: (usize, usize), action: Action) -> (usize, usize) {
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
