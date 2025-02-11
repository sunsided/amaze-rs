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
    prev_available_size: Option<egui::Vec2>,
    start_cell: Option<GridCoord2D>,
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
            prev_available_size: None,
            start_cell: None, // Initialize as None (no start cell selected)
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
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
                    .range(5..=100)
                    .clamp_existing_to_range(false)
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
                    .range(5..=100)
                    .clamp_existing_to_range(false)
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
                self.start_cell = None; // Optional: Reset start cell on view reset
            }

            // Instructions
            ui.label("Use the middle mouse button to pan the maze.");
            ui.label("Use the mouse wheel to zoom in or out.");
            ui.label("Click to select a cell.");
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
            let total_maze_width = (maze.width() as f32) * cell_size;
            let total_maze_height = (maze.height() as f32) * cell_size;

            // Allocate the painter with the available size
            let available_rect = ctx.available_rect();
            let available_size = available_rect.size();
            let (_response, painter) = ui.allocate_painter(available_size, egui::Sense::hover());

            // Adjust pan if the available size has changed
            if let Some(old_size) = self.prev_available_size {
                if old_size != available_size {
                    let delta = available_size - old_size;
                    self.pan += delta / 2.0;
                }
            }

            // Update the previous available size
            self.prev_available_size = Some(available_size);

            // Determine the center of the CentralPanel
            let center = available_rect.center();

            // Calculate the top-left position of the maze to center it
            let maze_top_left = egui::pos2(
                center.x - (total_maze_width / 2.0) + self.pan.x,
                center.y - (total_maze_height / 2.0) + self.pan.y,
            );

            // Get the hover position
            let hover_pos = ctx.input(|i| i.pointer.hover_pos());

            // Determine which cell is hovered
            let hovered_coord: Option<GridCoord2D> = if let Some(mouse_pos) = hover_pos {
                if mouse_pos.x >= maze_top_left.x
                    && mouse_pos.x < (maze_top_left.x + total_maze_width)
                    && mouse_pos.y >= maze_top_left.y
                    && mouse_pos.y < (maze_top_left.y + total_maze_height)
                {
                    let hovered_x = ((mouse_pos.x - maze_top_left.x) / cell_size).floor() as usize;
                    let hovered_y = ((mouse_pos.y - maze_top_left.y) / cell_size).floor() as usize;

                    if hovered_x < maze.width() && hovered_y < maze.height() {
                        Some(GridCoord2D::new(hovered_x, hovered_y))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            // Handle mouse clicks to select/unselect the start cell
            if ctx.input(|i| i.pointer.any_click()) {
                if let Some(mouse_pos) = ctx.input(|i| i.pointer.interact_pos()) {
                    if mouse_pos.x >= maze_top_left.x
                        && mouse_pos.x < (maze_top_left.x + total_maze_width)
                        && mouse_pos.y >= maze_top_left.y
                        && mouse_pos.y < (maze_top_left.y + total_maze_height)
                    {
                        let clicked_x =
                            ((mouse_pos.x - maze_top_left.x) / cell_size).floor() as usize;
                        let clicked_y =
                            ((mouse_pos.y - maze_top_left.y) / cell_size).floor() as usize;

                        if clicked_x < maze.width() && clicked_y < maze.height() {
                            let clicked_coord = GridCoord2D::new(clicked_x, clicked_y);

                            if self.start_cell == Some(clicked_coord) {
                                // Unselect if the same cell is clicked again
                                self.start_cell = None;
                            } else {
                                // Select the new cell as start position
                                self.start_cell = Some(clicked_coord);
                            }
                        }
                    }
                }
            }

            // Iterate over each cell and draw walls with appropriate highlights
            for y in 0..maze.height() {
                for x in 0..maze.width() {
                    let coord = GridCoord2D::new(x, y);
                    let wall = maze.get(coord).unwrap();

                    // Calculate the position with pan and zoom
                    let top_left = egui::pos2(
                        x as f32 * cell_size + maze_top_left.x,
                        y as f32 * cell_size + maze_top_left.y,
                    );
                    let top_right = egui::pos2(
                        (x as f32 + 1.0) * cell_size + maze_top_left.x,
                        y as f32 * cell_size + maze_top_left.y,
                    );
                    let bottom_left = egui::pos2(
                        x as f32 * cell_size + maze_top_left.x,
                        (y as f32 + 1.0) * cell_size + maze_top_left.y,
                    );
                    let bottom_right = egui::pos2(
                        (x as f32 + 1.0) * cell_size + maze_top_left.x,
                        (y as f32 + 1.0) * cell_size + maze_top_left.y,
                    );

                    // Determine fill color based on hover and selection
                    let fill_color = if Some(coord) == self.start_cell {
                        Color32::from_rgb(255, 200, 200) // Light red for start cell
                    } else if hovered_coord.map_or(false, |c| c.x == x && c.y == y) {
                        Color32::from_rgb(255, 255, 200) // Light yellow for hovered cell
                    } else if (x + y) % 2 == 0 {
                        Color32::from_rgb(240, 240, 240)
                    } else {
                        Color32::from_rgb(220, 220, 220)
                    };

                    // Fill the cell with the determined color
                    painter.rect_filled(
                        egui::Rect::from_min_max(top_left, bottom_right),
                        0.0,
                        fill_color,
                    );

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
