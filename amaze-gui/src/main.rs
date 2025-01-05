use amaze::generators::RecursiveBacktracker4;
use amaze::preamble::*;
use eframe::{egui, epaint::Color32, App, Frame, NativeOptions};
use std::sync::Mutex;

struct MyApp {
    // Application state
    seed_input: String,
    seed: u64,
    width: usize,
    height: usize,
    maze: Mutex<Wall4Grid>,
    // Transformation state
    zoom: f32,
    pan: egui::Vec2,
    dragging: bool,
    last_cursor_pos: egui::Pos2,
    // Track the previous available size of the central panel
    prev_available_size: Option<egui::Vec2>,
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
            zoom: 1.0,
            pan: egui::Vec2::new(0.0, 0.0),
            dragging: false,
            last_cursor_pos: egui::Pos2::new(0.0, 0.0),
            prev_available_size: None, // Initialize as None
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Handle user interactions for panning and zooming
        handle_panning_zooming(ctx, self);

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

            // Attempt to parse the seed input
            if seed_response.changed() {
                match self.seed_input.parse::<u64>() {
                    Ok(seed) => {
                        if seed != self.seed {
                            self.seed = seed;
                            let maze_generator = RecursiveBacktracker4::new_from_seed(self.seed);
                            let new_maze = maze_generator.generate(self.width, self.height);
                            let mut maze_lock = self.maze.lock().unwrap();
                            *maze_lock = new_maze;
                        }
                    }
                    Err(_) => {
                        // Display error message for invalid input
                        ui.label(
                            egui::RichText::new("Invalid seed! Please enter a valid u64 number.")
                                .color(Color32::RED),
                        );
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

            ui.separator();

            // Reset View Button
            if ui.button("Reset View").clicked() {
                self.zoom = 1.0;
                self.pan = egui::Vec2::new(0.0, 0.0);
            }

            // Instructions
            ui.label("Use the middle mouse button to pan the maze.");
            ui.label("Use the mouse wheel to zoom in or out.");
        });

        // Central Panel for Maze Rendering
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Maze Renderer");
            ui.separator();

            // Obtain the maze
            let maze = self.maze.lock().unwrap();

            // Define the base size of each cell
            let base_cell_size = 30.0;

            // Calculate the scaled cell size based on the zoom level
            let cell_size = base_cell_size * self.zoom;

            // Determine the total size of the maze
            let total_width = (maze.width() as f32) * cell_size;
            let total_height = (maze.height() as f32) * cell_size;

            // Allocate the painter with the available size
            let available_size = ui.available_size();
            let (response, painter) = ui.allocate_painter(available_size, egui::Sense::hover());

            // Adjust pan if the available size has changed
            if let Some(old_size) = self.prev_available_size {
                if old_size != available_size {
                    let delta = available_size - old_size;
                    // Compensate for the change by adjusting the pan
                    self.pan += delta / 2.0;
                }
            }

            // Update the previous available size
            self.prev_available_size = Some(available_size);

            // Calculate pan offsets to center the maze
            let center_x = (available_size.x - total_width) / 2.0 + self.pan.x;
            let center_y = (available_size.y - total_height) / 2.0 + self.pan.y;

            // Iterate over each cell and draw walls
            for y in 0..maze.height() {
                for x in 0..maze.width() {
                    let coord = GridCoord2D::new(x, y);
                    let wall = maze.get(coord).unwrap();

                    // Calculate the position with pan and zoom
                    let top_left = egui::pos2(
                        x as f32 * cell_size + center_x,
                        y as f32 * cell_size + center_y,
                    );
                    let top_right = egui::pos2(
                        (x as f32 + 1.0) * cell_size + center_x,
                        y as f32 * cell_size + center_y,
                    );
                    let bottom_left = egui::pos2(
                        x as f32 * cell_size + center_x,
                        (y as f32 + 1.0) * cell_size + center_y,
                    );
                    let bottom_right = egui::pos2(
                        (x as f32 + 1.0) * cell_size + center_x,
                        (y as f32 + 1.0) * cell_size + center_y,
                    );

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

        // Request a repaint to update the UI continuously
        ctx.request_repaint();
    }
}

/// Handles panning and zooming interactions
fn handle_panning_zooming(ctx: &egui::Context, app: &mut MyApp) {
    // Handle panning
    if ctx.input(|i| i.pointer.middle_down()) {
        if let Some(cursor_pos) = ctx.input(|i| i.pointer.latest_pos()) {
            if app.dragging {
                let delta = cursor_pos - app.last_cursor_pos;
                app.pan += delta;
            }
            app.dragging = true;
            app.last_cursor_pos = cursor_pos;
        }
    } else {
        app.dragging = false;
    }

    // Handle zooming with the mouse wheel
    let scroll_delta = ctx.input(|i| i.smooth_scroll_delta.y);
    if scroll_delta != 0.0 {
        let zoom_factor = 1.1_f32.powf(scroll_delta / 10.0);
        app.zoom = (app.zoom * zoom_factor).clamp(0.1, 5.0);
    }
}

fn main() {
    let native_options = NativeOptions::default();
    eframe::run_native(
        "Maze Renderer",
        native_options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
    .unwrap();
}
