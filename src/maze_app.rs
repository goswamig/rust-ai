use iced::{
    Application, Command, Element, Length, Subscription,
    widget::{Button, Column, Container, Row, Text},
    Color,
    executor::Default as DefaultExecutor,
    Clipboard,
};
use iced::button; // Use `button` directly
use std::time::Duration;
use tokio::time::sleep;

use crate::maze_solver::{MazeSolver, GRID_SIZE};

pub struct MazeApp {
    start_button: button::State, // Use `button::State` directly
    maze_solver: MazeSolver,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    StartPressed,
}

impl Application for MazeApp {
    type Executor = DefaultExecutor;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
            Self {
                start_button: button::State::new(),
                maze_solver: MazeSolver::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Maze Solver")
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        // Adjusted for simplicity; ensure this matches your requirements
        Subscription::none()
    }

    fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Message> {
        match message {
            Message::StartPressed => {
                self.maze_solver.make_move();
            }
        }
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        
        let mut column = Column::new()
            .push(
                Button::new(&mut self.start_button, Text::new("Step"))
                    .on_press(Message::StartPressed),
            );


            for x in 0..GRID_SIZE {
                let mut row = Row::new();
                for y in 0..GRID_SIZE {
                    let cell_text = if self.maze_solver.obstacles.contains(&(x, y)) {
                        Text::new("O").color(Color::from_rgb(1.0, 0.0, 0.0)) // Red for obstacles
                    } else if (x, y) == self.maze_solver.current_state {
                        Text::new("A").color(Color::from_rgb(0.0, 1.0, 0.0)) // Green for the agent
                    } else {
                        let symbol = if self.maze_solver.path.contains(&(x, y)) { "*" } else { "." };
                        Text::new(symbol) // Blue for the path
                    };
    
                    let cell = Container::new(cell_text)
                        .width(Length::Units(40))
                        .height(Length::Units(40))
                        .center_x()
                        .center_y();
    
                    row = row.push(cell);
                }
                column = column.push(row);
            }

        column.into()
    }
}
