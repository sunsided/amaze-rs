use amaze::generators::RecursiveBacktracker4;
use amaze::preamble::*;
use eframe::{egui, epaint::Color32, App, Frame, NativeOptions};
use std::sync::Mutex;

fn main() {
    let native_options = NativeOptions::default();
    eframe::run_native(
        "Maze Renderer",
        native_options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
    .unwrap();
}

struct MyApp {
    // Application state
    seed_input: String,
    seed: u64,
    width: usize,
    height: usize,
    maze: Mutex<Wall4Grid>,
}

impl Default for MyApp {
    fn default() -> Self {
        let initial_seed = 1337;
        let initial_width = 18;
        let initial_height = 24;
        let maze_generator = RecursiveBacktracker4::new_from_seed(initial_seed);
        let maze = maze_generator.generate(initial_width, initial_height);

        Self {
            seed_input: initial_seed.to_string(),
            seed: initial_seed,
            width: initial_width,
            height: initial_height,
            maze: Mutex::new(maze),
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Side Panel for Controls
        egui::SidePanel::left("controls_panel").show(ctx, |ui| {
            ui.heading("Maze Controls");

            // Input for Seed
            ui.label("Seed (u64):");
            let seed_response = ui.add(
                egui::TextEdit::singleline(&mut self.seed_input)
                    .hint_text("Enter seed (u64)")
                    .desired_width(150.0),
            );

            // Update seed and regenerate maze if seed input changes and is valid
            if seed_response.changed() {
                if let Ok(seed) = self.seed_input.parse::<u64>() {
                    if seed != self.seed {
                        self.seed = seed;
                        let maze_generator = RecursiveBacktracker4::new_from_seed(self.seed);
                        let new_maze = maze_generator.generate(self.width, self.height);
                        let mut maze_lock = self.maze.lock().unwrap();
                        *maze_lock = new_maze;
                    }
                }
            }

            // Input for Width
            ui.label("Width:");
            let width_response = ui.add(
                egui::DragValue::new(&mut self.width)
                    .clamp_range(5..=100)
                    .speed(1.0)
                    .prefix(""),
            );

            if width_response.changed() {
                let maze_generator = RecursiveBacktracker4::new_from_seed(self.seed);
                let new_maze = maze_generator.generate(self.width, self.height);
                let mut maze_lock = self.maze.lock().unwrap();
                *maze_lock = new_maze;
            }

            // Input for Height
            ui.label("Height:");
            let height_response = ui.add(
                egui::DragValue::new(&mut self.height)
                    .clamp_range(5..=100)
                    .speed(1.0)
                    .prefix(""),
            );

            if height_response.changed() {
                let maze_generator = RecursiveBacktracker4::new_from_seed(self.seed);
                let new_maze = maze_generator.generate(self.width, self.height);
                let mut maze_lock = self.maze.lock().unwrap();
                *maze_lock = new_maze;
            }

            // Button to Regenerate Maze
            if ui.button("Regenerate Maze").clicked() {
                if let Ok(seed) = self.seed_input.parse::<u64>() {
                    if seed != self.seed {
                        self.seed = seed;
                    }
                }
                let maze_generator = RecursiveBacktracker4::new_from_seed(self.seed);
                let new_maze = maze_generator.generate(self.width, self.height);
                let mut maze_lock = self.maze.lock().unwrap();
                *maze_lock = new_maze;
            }

            ui.separator();

            // Instructions
            ui.label("Adjust the seed, width, or height and click 'Regenerate Maze' to create a new maze.");
        });

        // Central Panel for Maze Rendering
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Maze Renderer");

            // Obtain the maze
            let maze = self.maze.lock().unwrap();

            // Define the size of each cell based on the maze dimensions
            let max_cell_size: f32 = 40.0;
            let cell_size = max_cell_size
                .min(ui.available_width() / maze.width() as f32)
                .min(ui.available_height() / maze.height() as f32);

            // Determine the total size of the maze
            let total_width = (maze.width() as f32) * cell_size;
            let total_height = (maze.height() as f32) * cell_size;

            // Create a scrollable area if the maze is large
            egui::ScrollArea::both().show(ui, |ui| {
                // Allocate space for the maze
                let (response, painter) = ui.allocate_painter(
                    egui::Vec2::new(total_width, total_height),
                    egui::Sense::hover(),
                );

                // Iterate over each cell and draw walls
                for y in 0..maze.height() {
                    for x in 0..maze.width() {
                        let coord = GridCoord2D::new(x, y);
                        let wall = maze.get(coord).unwrap();

                        let top_left = egui::pos2(x as f32 * cell_size, y as f32 * cell_size);
                        let top_right =
                            egui::pos2((x as f32 + 1.0) * cell_size, y as f32 * cell_size);
                        let bottom_left =
                            egui::pos2(x as f32 * cell_size, (y as f32 + 1.0) * cell_size);
                        let bottom_right =
                            egui::pos2((x as f32 + 1.0) * cell_size, (y as f32 + 1.0) * cell_size);

                        // Optional: Fill cell background for better visuals
                        if (x + y) % 2 == 0 {
                            painter.rect_filled(
                                egui::Rect::from_min_max(top_left, bottom_right),
                                0.0,
                                Color32::from_rgb(240, 240, 240),
                            );
                        } else {
                            painter.rect_filled(
                                egui::Rect::from_min_max(top_left, bottom_right),
                                0.0,
                                Color32::from_rgb(220, 220, 220),
                            );
                        }

                        // Draw North Wall
                        if wall.contains(Direction4::NORTH) {
                            painter.line_segment(
                                [top_left, top_right],
                                egui::Stroke::new(2.0, Color32::BLACK),
                            );
                        }

                        // Draw South Wall
                        if wall.contains(Direction4::SOUTH) {
                            painter.line_segment(
                                [bottom_left, bottom_right],
                                egui::Stroke::new(2.0, Color32::BLACK),
                            );
                        }

                        // Draw East Wall
                        if wall.contains(Direction4::EAST) {
                            painter.line_segment(
                                [top_right, bottom_right],
                                egui::Stroke::new(2.0, Color32::BLACK),
                            );
                        }

                        // Draw West Wall
                        if wall.contains(Direction4::WEST) {
                            painter.line_segment(
                                [top_left, bottom_left],
                                egui::Stroke::new(2.0, Color32::BLACK),
                            );
                        }
                    }
                }
            });
        });

        // Request a repaint to update the UI continuously
        ctx.request_repaint();
    }
}
