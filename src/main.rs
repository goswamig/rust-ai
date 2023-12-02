
mod maze_solver;
mod maze_app;

use crate::maze_app::MazeApp;
use iced::Application; // Import Application trait

use iced::{Settings, window};

fn main() -> iced::Result {
    MazeApp::run(Settings {
        window: window::Settings {
            size: (800, 600), // Increased window size
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}
