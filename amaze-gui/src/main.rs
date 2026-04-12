use amaze::generators::{
    BinaryTree4, Eller4, GenerationStep, GrowingTree4, HuntAndKill4, Kruskal4, MazeGenerator2D,
    RecursiveBacktracker4, Sidewinder4, Wilson4,
};
use amaze::preamble::*;
use eframe::{egui, epaint::Color32, App, Frame, NativeOptions};
use std::sync::Mutex;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum AlgorithmChoice {
    RecursiveBacktracker,
    GrowingTree,
    Kruskal,
    Eller,
    Wilson,
    HuntAndKill,
    Sidewinder,
    BinaryTree,
}

impl AlgorithmChoice {
    fn as_str(self) -> &'static str {
        match self {
            Self::RecursiveBacktracker => "Recursive Backtracker",
            Self::GrowingTree => "Growing Tree",
            Self::Kruskal => "Kruskal",
            Self::Eller => "Eller",
            Self::Wilson => "Wilson",
            Self::HuntAndKill => "Hunt and Kill",
            Self::Sidewinder => "Sidewinder",
            Self::BinaryTree => "Binary Tree",
        }
    }
}

struct MyApp {
    seed_input: String,
    seed: u64,
    width: usize,
    height: usize,
    algorithm: AlgorithmChoice,
    maze: Mutex<Wall4Grid>,
    zoom: f32,
    pan: egui::Vec2,
    dragging: bool,
    last_cursor_pos: egui::Pos2,
    prev_available_size: Option<egui::Vec2>,
    start_cell: Option<GridCoord2D>,
    end_cell: Option<GridCoord2D>,
    animation_steps: Vec<GenerationStep>,
    animation_index: usize,
    is_animating: bool,
    auto_fit_pending: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        let initial_seed = 1337;
        let initial_width = 18;
        let initial_height = 24;
        let algorithm = AlgorithmChoice::RecursiveBacktracker;
        let maze = generate_maze(algorithm, initial_seed, initial_width, initial_height);

        Self {
            seed_input: initial_seed.to_string(),
            seed: initial_seed,
            width: initial_width,
            height: initial_height,
            algorithm,
            maze: Mutex::new(maze),
            zoom: 1.0,
            pan: egui::Vec2::new(0.0, 0.0),
            dragging: false,
            last_cursor_pos: egui::Pos2::new(0.0, 0.0),
            prev_available_size: None,
            start_cell: None,
            end_cell: None,
            animation_steps: Vec::new(),
            animation_index: 0,
            is_animating: false,
            auto_fit_pending: true,
        }
    }
}

impl App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut Frame) {
        let ctx = ui.ctx().clone();

        handle_panning_zooming(&ctx, self);
        tick_animation(self);

        egui::SidePanel::left("controls_panel").show(&ctx, |ui| {
            ui.heading("Maze Controls");

            ui.label("Algorithm:");
            let previous_algorithm = self.algorithm;
            egui::ComboBox::from_id_salt("algorithm")
                .selected_text(self.algorithm.as_str())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.algorithm,
                        AlgorithmChoice::RecursiveBacktracker,
                        AlgorithmChoice::RecursiveBacktracker.as_str(),
                    );
                    ui.selectable_value(
                        &mut self.algorithm,
                        AlgorithmChoice::GrowingTree,
                        AlgorithmChoice::GrowingTree.as_str(),
                    );
                    ui.selectable_value(
                        &mut self.algorithm,
                        AlgorithmChoice::Kruskal,
                        AlgorithmChoice::Kruskal.as_str(),
                    );
                    ui.selectable_value(
                        &mut self.algorithm,
                        AlgorithmChoice::Eller,
                        AlgorithmChoice::Eller.as_str(),
                    );
                    ui.selectable_value(
                        &mut self.algorithm,
                        AlgorithmChoice::Wilson,
                        AlgorithmChoice::Wilson.as_str(),
                    );
                    ui.selectable_value(
                        &mut self.algorithm,
                        AlgorithmChoice::HuntAndKill,
                        AlgorithmChoice::HuntAndKill.as_str(),
                    );
                    ui.selectable_value(
                        &mut self.algorithm,
                        AlgorithmChoice::Sidewinder,
                        AlgorithmChoice::Sidewinder.as_str(),
                    );
                    ui.selectable_value(
                        &mut self.algorithm,
                        AlgorithmChoice::BinaryTree,
                        AlgorithmChoice::BinaryTree.as_str(),
                    );
                });

            if previous_algorithm != self.algorithm {
                regenerate(self);
            }

            ui.label("Seed (u64):");
            let seed_response = ui.add(
                egui::TextEdit::singleline(&mut self.seed_input)
                    .hint_text("Enter seed (u64)")
                    .desired_width(150.0),
            );
            if seed_response.changed() {
                if let Ok(seed) = self.seed_input.parse::<u64>() {
                    if seed != self.seed {
                        self.seed = seed;
                        regenerate(self);
                    }
                } else {
                    ui.label(
                        egui::RichText::new("Invalid seed! Please enter a valid u64 number.")
                            .color(Color32::RED),
                    );
                }
            }

            ui.label("Width:");
            if ui
                .add(
                    egui::DragValue::new(&mut self.width)
                        .range(5..=100)
                        .clamp_existing_to_range(false)
                        .speed(1.0),
                )
                .changed()
            {
                regenerate(self);
            }

            ui.label("Height:");
            if ui
                .add(
                    egui::DragValue::new(&mut self.height)
                        .range(5..=100)
                        .clamp_existing_to_range(false)
                        .speed(1.0),
                )
                .changed()
            {
                regenerate(self);
            }

            ui.separator();
            if ui.button("Animate Generation").clicked() {
                self.animation_steps =
                    generate_steps(self.algorithm, self.seed, self.width, self.height);
                self.animation_index = 0;
                self.is_animating = true;
                let mut lock = self.maze.lock().unwrap();
                *lock = Wall4Grid::new(self.width, self.height);
            }

            if ui.button("Reset View").clicked() {
                self.auto_fit_pending = true;
                self.pan = egui::Vec2::new(0.0, 0.0);
                self.start_cell = None;
                self.end_cell = None;
            }

            if let (Some(start), Some(end)) = (self.start_cell, self.end_cell) {
                let maze = self.maze.lock().unwrap();
                let has_path = BfsSolver.solve(&maze, start, end).is_some();
                ui.label(format!("Path ready: {has_path}"));
            }

            ui.separator();
            ui.label("Middle mouse: pan");
            ui.label("Mouse wheel: zoom");
            ui.label("Click: set start/end cells");
        });

        egui::CentralPanel::default().show(&ctx, |ui| {
            ui.heading("Maze Renderer");
            ui.separator();

            let maze = self.maze.lock().unwrap();
            let base_cell_size = 30.0;
            let cell_size = base_cell_size * self.zoom;
            let total_maze_width = maze.width() as f32 * cell_size;
            let total_maze_height = maze.height() as f32 * cell_size;

            let available_size = ui.available_size();
            let (response, painter) = ui.allocate_painter(available_size, egui::Sense::hover());
            let available_rect = response.rect;

            if self.auto_fit_pending {
                self.zoom = fit_zoom_to_available(&maze, available_size);
                self.auto_fit_pending = false;
            }

            if let Some(old_size) = self.prev_available_size {
                if old_size != available_size {
                    let delta = available_size - old_size;
                    self.pan += delta / 2.0;
                }
            }
            self.prev_available_size = Some(available_size);

            let center = available_rect.center();
            let maze_top_left = egui::pos2(
                center.x - total_maze_width / 2.0 + self.pan.x,
                center.y - total_maze_height / 2.0 + self.pan.y,
            );

            let hover_pos = ctx.input(|i| i.pointer.hover_pos());
            let hovered_coord: Option<GridCoord2D> = hover_pos.and_then(|mouse_pos| {
                if mouse_pos.x >= maze_top_left.x
                    && mouse_pos.x < maze_top_left.x + total_maze_width
                    && mouse_pos.y >= maze_top_left.y
                    && mouse_pos.y < maze_top_left.y + total_maze_height
                {
                    let hx = ((mouse_pos.x - maze_top_left.x) / cell_size).floor() as usize;
                    let hy = ((mouse_pos.y - maze_top_left.y) / cell_size).floor() as usize;
                    if hx < maze.width() && hy < maze.height() {
                        Some(GridCoord2D::new(hx, hy))
                    } else {
                        None
                    }
                } else {
                    None
                }
            });

            if ctx.input(|i| i.pointer.any_click()) {
                if let Some(clicked) = hovered_coord {
                    if self.start_cell.is_none() || self.end_cell.is_some() {
                        self.start_cell = Some(clicked);
                        self.end_cell = None;
                    } else {
                        self.end_cell = Some(clicked);
                    }
                }
            }

            let solution = if let (Some(start), Some(end)) = (self.start_cell, self.end_cell) {
                BfsSolver.solve(&maze, start, end)
            } else {
                None
            };

            for y in 0..maze.height() {
                for x in 0..maze.width() {
                    let coord = GridCoord2D::new(x, y);
                    let wall = maze.get(coord).expect("grid cell exists");

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

                    let is_in_solution = solution
                        .as_ref()
                        .is_some_and(|path| path.cells().contains(&coord));

                    let fill_color = if Some(coord) == self.start_cell {
                        Color32::from_rgb(255, 200, 200)
                    } else if Some(coord) == self.end_cell {
                        Color32::from_rgb(200, 200, 255)
                    } else if is_in_solution {
                        Color32::from_rgb(180, 230, 180)
                    } else if hovered_coord.is_some_and(|c| c == coord) {
                        Color32::from_rgb(255, 255, 200)
                    } else if (x + y) % 2 == 0 {
                        Color32::from_rgb(240, 240, 240)
                    } else {
                        Color32::from_rgb(220, 220, 220)
                    };

                    painter.rect_filled(
                        egui::Rect::from_min_max(top_left, bottom_right),
                        0.0,
                        fill_color,
                    );

                    if wall.contains(Direction4::NORTH) {
                        painter.line_segment(
                            [top_left, top_right],
                            egui::Stroke::new(2.0, Color32::BLACK),
                        );
                    }
                    if wall.contains(Direction4::SOUTH) {
                        painter.line_segment(
                            [bottom_left, bottom_right],
                            egui::Stroke::new(2.0, Color32::BLACK),
                        );
                    }
                    if wall.contains(Direction4::EAST) {
                        painter.line_segment(
                            [top_right, bottom_right],
                            egui::Stroke::new(2.0, Color32::BLACK),
                        );
                    }
                    if wall.contains(Direction4::WEST) {
                        painter.line_segment(
                            [top_left, bottom_left],
                            egui::Stroke::new(2.0, Color32::BLACK),
                        );
                    }
                }
            }
        });

        ctx.request_repaint();
    }
}

fn generate_maze(algorithm: AlgorithmChoice, seed: u64, width: usize, height: usize) -> Wall4Grid {
    match algorithm {
        AlgorithmChoice::RecursiveBacktracker => {
            RecursiveBacktracker4::new_from_seed(seed).generate(width, height)
        }
        AlgorithmChoice::GrowingTree => {
            <GrowingTree4 as MazeGenerator2D>::new_from_seed(seed).generate(width, height)
        }
        AlgorithmChoice::Kruskal => {
            <Kruskal4 as MazeGenerator2D>::new_from_seed(seed).generate(width, height)
        }
        AlgorithmChoice::Eller => {
            <Eller4 as MazeGenerator2D>::new_from_seed(seed).generate(width, height)
        }
        AlgorithmChoice::Wilson => {
            <Wilson4 as MazeGenerator2D>::new_from_seed(seed).generate(width, height)
        }
        AlgorithmChoice::HuntAndKill => {
            <HuntAndKill4 as MazeGenerator2D>::new_from_seed(seed).generate(width, height)
        }
        AlgorithmChoice::Sidewinder => {
            <Sidewinder4 as MazeGenerator2D>::new_from_seed(seed).generate(width, height)
        }
        AlgorithmChoice::BinaryTree => {
            <BinaryTree4 as MazeGenerator2D>::new_from_seed(seed).generate(width, height)
        }
    }
}

fn generate_steps(
    algorithm: AlgorithmChoice,
    seed: u64,
    width: usize,
    height: usize,
) -> Vec<GenerationStep> {
    match algorithm {
        AlgorithmChoice::RecursiveBacktracker => RecursiveBacktracker4::new_from_seed(seed)
            .generate_steps(width, height)
            .collect(),
        AlgorithmChoice::GrowingTree => <GrowingTree4 as MazeGenerator2D>::new_from_seed(seed)
            .generate_steps(width, height)
            .collect(),
        AlgorithmChoice::Kruskal => <Kruskal4 as MazeGenerator2D>::new_from_seed(seed)
            .generate_steps(width, height)
            .collect(),
        AlgorithmChoice::Eller => <Eller4 as MazeGenerator2D>::new_from_seed(seed)
            .generate_steps(width, height)
            .collect(),
        AlgorithmChoice::Wilson => <Wilson4 as MazeGenerator2D>::new_from_seed(seed)
            .generate_steps(width, height)
            .collect(),
        AlgorithmChoice::HuntAndKill => <HuntAndKill4 as MazeGenerator2D>::new_from_seed(seed)
            .generate_steps(width, height)
            .collect(),
        AlgorithmChoice::Sidewinder => <Sidewinder4 as MazeGenerator2D>::new_from_seed(seed)
            .generate_steps(width, height)
            .collect(),
        AlgorithmChoice::BinaryTree => <BinaryTree4 as MazeGenerator2D>::new_from_seed(seed)
            .generate_steps(width, height)
            .collect(),
    }
}

fn regenerate(app: &mut MyApp) {
    app.is_animating = false;
    app.animation_steps.clear();
    app.animation_index = 0;
    app.start_cell = None;
    app.end_cell = None;
    app.auto_fit_pending = true;
    let mut lock = app.maze.lock().unwrap();
    *lock = generate_maze(app.algorithm, app.seed, app.width, app.height);
}

fn fit_zoom_to_available(maze: &Wall4Grid, available_size: egui::Vec2) -> f32 {
    let base_cell_size = 30.0;
    let maze_w = (maze.width() as f32) * base_cell_size;
    let maze_h = (maze.height() as f32) * base_cell_size;
    if maze_w <= 0.0 || maze_h <= 0.0 {
        return 1.0;
    }

    let padding = 24.0;
    let fit_w = (available_size.x - padding).max(32.0) / maze_w;
    let fit_h = (available_size.y - padding).max(32.0) / maze_h;
    fit_w.min(fit_h).clamp(0.1, 5.0)
}

fn tick_animation(app: &mut MyApp) {
    if !app.is_animating {
        return;
    }

    let mut maze = app.maze.lock().unwrap();
    let mut processed = 0usize;
    while app.animation_index < app.animation_steps.len() && processed < 8 {
        match app.animation_steps[app.animation_index] {
            GenerationStep::Carve { from, to } => maze.remove_wall_between(from, to),
            GenerationStep::Complete => {
                app.is_animating = false;
                break;
            }
            _ => {}
        }
        app.animation_index += 1;
        processed += 1;
    }

    if app.animation_index >= app.animation_steps.len() {
        app.is_animating = false;
    }
}

fn handle_panning_zooming(ctx: &egui::Context, app: &mut MyApp) {
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
