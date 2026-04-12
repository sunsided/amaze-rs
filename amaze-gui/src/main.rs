use amaze::dungeon::{DungeonGrid, DungeonType, DungeonWalkGenerator, TileType, solve_bfs};
use amaze::generators::{
    BinaryTree4, Eller4, GenerationStep, GrowingTree4, HuntAndKill4, Kruskal4, MazeGenerator2D,
    RecursiveBacktracker4, Sidewinder4, Wilson4,
};
use amaze::preamble::*;
use eframe::{App, Frame, NativeOptions, egui, epaint::Color32};
use std::sync::Mutex;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Mode {
    Maze,
    Dungeon,
}

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
    mode: Mode,
    seed_input: String,
    seed: u64,
    width: usize,
    height: usize,
    algorithm: AlgorithmChoice,
    maze: Mutex<Wall4Grid>,
    dungeon: Mutex<DungeonGrid>,
    dungeon_type: DungeonType,
    floor_count: usize,
    winding_probability: u8,
    long_walk_min: usize,
    long_walk_max: usize,
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
        let dungeon = DungeonWalkGenerator::new_from_seed(DungeonType::Rooms, initial_seed)
            .with_winding_probability(50)
            .with_long_walk_range(9, 18)
            .generate(initial_width, initial_height, 120);

        Self {
            mode: Mode::Maze,
            seed_input: initial_seed.to_string(),
            seed: initial_seed,
            width: initial_width,
            height: initial_height,
            algorithm,
            maze: Mutex::new(maze),
            dungeon: Mutex::new(dungeon),
            dungeon_type: DungeonType::Rooms,
            floor_count: 120,
            winding_probability: 50,
            long_walk_min: 9,
            long_walk_max: 18,
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

        egui::Panel::left("controls_panel").show_inside(ui, |ui| {
            ui.heading("Controls");

            ui.label("Mode:");
            let previous_mode = self.mode;
            egui::ComboBox::from_id_salt("mode")
                .selected_text(match self.mode {
                    Mode::Maze => "Maze",
                    Mode::Dungeon => "Dungeon",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.mode, Mode::Maze, "Maze");
                    ui.selectable_value(&mut self.mode, Mode::Dungeon, "Dungeon");
                });

            if previous_mode != self.mode {
                self.auto_fit_pending = true;
                self.start_cell = None;
                self.end_cell = None;
            }

            ui.separator();

            if self.mode == Mode::Maze {
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
                    regenerate_maze(self);
                }
            } else {
                ui.label("Dungeon Type:");
                let previous_dungeon_type = self.dungeon_type;
                egui::ComboBox::from_id_salt("dungeon_type")
                    .selected_text(match self.dungeon_type {
                        DungeonType::Caverns => "Caverns",
                        DungeonType::Rooms => "Rooms",
                        DungeonType::Winding => "Winding",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.dungeon_type,
                            DungeonType::Caverns,
                            "Caverns",
                        );
                        ui.selectable_value(&mut self.dungeon_type, DungeonType::Rooms, "Rooms");
                        ui.selectable_value(
                            &mut self.dungeon_type,
                            DungeonType::Winding,
                            "Winding",
                        );
                    });

                if previous_dungeon_type != self.dungeon_type {
                    regenerate_dungeon(self);
                }

                ui.label("Floor Tiles:");
                if ui
                    .add(
                        egui::DragValue::new(&mut self.floor_count)
                            .range(50..=5000)
                            .speed(5.0),
                    )
                    .changed()
                {
                    regenerate_dungeon(self);
                }

                if self.dungeon_type == DungeonType::Rooms
                    || self.dungeon_type == DungeonType::Winding
                {
                    ui.label("Long Walk Min:");
                    if ui
                        .add(
                            egui::DragValue::new(&mut self.long_walk_min)
                                .range(1..=50)
                                .speed(1.0),
                        )
                        .changed()
                    {
                        // Ensure max > min
                        if self.long_walk_max <= self.long_walk_min {
                            self.long_walk_max = self.long_walk_min + 1;
                        }
                        regenerate_dungeon(self);
                    }

                    ui.label("Long Walk Max:");
                    if ui
                        .add(
                            egui::DragValue::new(&mut self.long_walk_max)
                                .range(2..=100)
                                .speed(1.0),
                        )
                        .changed()
                    {
                        // Ensure max > min
                        if self.long_walk_max <= self.long_walk_min {
                            self.long_walk_min = self.long_walk_max - 1;
                        }
                        regenerate_dungeon(self);
                    }
                }

                if self.dungeon_type == DungeonType::Winding {
                    ui.label("Winding Probability (%):");
                    if ui
                        .add(egui::Slider::new(&mut self.winding_probability, 0..=100))
                        .changed()
                    {
                        regenerate_dungeon(self);
                    }
                }
            }

            ui.separator();

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
                        if self.mode == Mode::Maze {
                            regenerate_maze(self);
                        } else {
                            regenerate_dungeon(self);
                        }
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
                if self.mode == Mode::Maze {
                    regenerate_maze(self);
                } else {
                    regenerate_dungeon(self);
                }
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
                if self.mode == Mode::Maze {
                    regenerate_maze(self);
                } else {
                    regenerate_dungeon(self);
                }
            }

            ui.separator();
            if self.mode == Mode::Maze && ui.button("Animate Generation").clicked() {
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
                if self.mode == Mode::Maze {
                    let maze = self.maze.lock().unwrap();
                    let has_path = BfsSolver.solve(&maze, start, end).is_some();
                    ui.label(format!("Path ready: {has_path}"));
                } else {
                    let dungeon = self.dungeon.lock().unwrap();
                    let passability = PassabilityGrid::from(&*dungeon);
                    let has_path = solve_bfs(&passability, start, end).is_some();
                    ui.label(format!("Path ready: {has_path}"));
                }
            }

            ui.separator();
            ui.label("Middle mouse: pan");
            ui.label("Mouse wheel: zoom");
            ui.label("Click: set start/end cells");
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading(match self.mode {
                Mode::Maze => "Maze Renderer",
                Mode::Dungeon => "Dungeon Renderer",
            });
            ui.separator();

            if self.mode == Mode::Maze {
                render_maze(ui, self, &ctx);
            } else {
                render_dungeon(ui, self, &ctx);
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

fn regenerate_maze(app: &mut MyApp) {
    app.is_animating = false;
    app.animation_steps.clear();
    app.animation_index = 0;
    app.start_cell = None;
    app.end_cell = None;
    app.auto_fit_pending = true;
    let mut lock = app.maze.lock().unwrap();
    *lock = generate_maze(app.algorithm, app.seed, app.width, app.height);
}

fn regenerate_dungeon(app: &mut MyApp) {
    app.start_cell = None;
    app.end_cell = None;
    app.auto_fit_pending = true;
    let mut lock = app.dungeon.lock().unwrap();
    *lock = DungeonWalkGenerator::new_from_seed(app.dungeon_type, app.seed)
        .with_winding_probability(app.winding_probability)
        .with_long_walk_range(app.long_walk_min, app.long_walk_max)
        .generate(app.width, app.height, app.floor_count);
}

fn render_maze(ui: &mut egui::Ui, app: &mut MyApp, ctx: &egui::Context) {
    let maze = app.maze.lock().unwrap();
    let base_cell_size = 30.0;
    let cell_size = base_cell_size * app.zoom;
    let total_maze_width = maze.width() as f32 * cell_size;
    let total_maze_height = maze.height() as f32 * cell_size;

    let available_size = ui.available_size();
    let (response, painter) = ui.allocate_painter(available_size, egui::Sense::hover());
    let available_rect = response.rect;

    if app.auto_fit_pending {
        app.zoom = fit_zoom_to_available(&maze, available_size);
        app.auto_fit_pending = false;
    }

    if let Some(old_size) = app.prev_available_size
        && old_size != available_size
    {
        let delta = available_size - old_size;
        app.pan += delta / 2.0;
    }
    app.prev_available_size = Some(available_size);

    let center = available_rect.center();
    let maze_top_left = egui::pos2(
        center.x - total_maze_width / 2.0 + app.pan.x,
        center.y - total_maze_height / 2.0 + app.pan.y,
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

    if ctx.input(|i| i.pointer.any_click())
        && let Some(clicked) = hovered_coord
    {
        if app.start_cell.is_none() || app.end_cell.is_some() {
            app.start_cell = Some(clicked);
            app.end_cell = None;
        } else {
            app.end_cell = Some(clicked);
        }
    }

    let solution = if let (Some(start), Some(end)) = (app.start_cell, app.end_cell) {
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

            let fill_color = if Some(coord) == app.start_cell {
                Color32::from_rgb(255, 200, 200)
            } else if Some(coord) == app.end_cell {
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
}

fn render_dungeon(ui: &mut egui::Ui, app: &mut MyApp, ctx: &egui::Context) {
    let dungeon = app.dungeon.lock().unwrap();
    let base_cell_size = 30.0;
    let cell_size = base_cell_size * app.zoom;
    let total_dungeon_width = dungeon.width() as f32 * cell_size;
    let total_dungeon_height = dungeon.height() as f32 * cell_size;

    let available_size = ui.available_size();
    let (response, painter) = ui.allocate_painter(available_size, egui::Sense::hover());
    let available_rect = response.rect;

    if app.auto_fit_pending {
        app.zoom = fit_zoom_to_dungeon(&dungeon, available_size);
        app.auto_fit_pending = false;
    }

    if let Some(old_size) = app.prev_available_size
        && old_size != available_size
    {
        let delta = available_size - old_size;
        app.pan += delta / 2.0;
    }
    app.prev_available_size = Some(available_size);

    let center = available_rect.center();
    let dungeon_top_left = egui::pos2(
        center.x - total_dungeon_width / 2.0 + app.pan.x,
        center.y - total_dungeon_height / 2.0 + app.pan.y,
    );

    let hover_pos = ctx.input(|i| i.pointer.hover_pos());
    let hovered_coord: Option<GridCoord2D> = hover_pos.and_then(|mouse_pos| {
        if mouse_pos.x >= dungeon_top_left.x
            && mouse_pos.x < dungeon_top_left.x + total_dungeon_width
            && mouse_pos.y >= dungeon_top_left.y
            && mouse_pos.y < dungeon_top_left.y + total_dungeon_height
        {
            let hx = ((mouse_pos.x - dungeon_top_left.x) / cell_size).floor() as usize;
            let hy = ((mouse_pos.y - dungeon_top_left.y) / cell_size).floor() as usize;
            if hx < dungeon.width() && hy < dungeon.height() {
                Some(GridCoord2D::new(hx, hy))
            } else {
                None
            }
        } else {
            None
        }
    });

    if ctx.input(|i| i.pointer.any_click())
        && let Some(clicked) = hovered_coord
    {
        // Only allow clicking on floor tiles
        if dungeon.get(clicked) == Some(TileType::Floor) {
            if app.start_cell.is_none() || app.end_cell.is_some() {
                app.start_cell = Some(clicked);
                app.end_cell = None;
            } else {
                app.end_cell = Some(clicked);
            }
        }
    }

    let solution = if let (Some(start), Some(end)) = (app.start_cell, app.end_cell) {
        let passability = PassabilityGrid::from(&*dungeon);
        solve_bfs(&passability, start, end)
    } else {
        None
    };

    for y in 0..dungeon.height() {
        for x in 0..dungeon.width() {
            let coord = GridCoord2D::new(x, y);
            let tile = dungeon.get(coord).expect("grid cell exists");

            let top_left = egui::pos2(
                x as f32 * cell_size + dungeon_top_left.x,
                y as f32 * cell_size + dungeon_top_left.y,
            );
            let bottom_right = egui::pos2(
                (x as f32 + 1.0) * cell_size + dungeon_top_left.x,
                (y as f32 + 1.0) * cell_size + dungeon_top_left.y,
            );

            let is_exit = dungeon.exit() == Some(coord);
            let is_in_solution = solution
                .as_ref()
                .is_some_and(|path| path.cells().contains(&coord));

            let fill_color = match tile {
                TileType::Wall => Color32::from_rgb(40, 40, 40),
                TileType::Floor => {
                    if Some(coord) == app.start_cell {
                        Color32::from_rgb(255, 200, 200)
                    } else if Some(coord) == app.end_cell {
                        Color32::from_rgb(200, 200, 255)
                    } else if is_exit {
                        Color32::from_rgb(255, 215, 0) // Gold for exit
                    } else if is_in_solution {
                        Color32::from_rgb(180, 230, 180)
                    } else if hovered_coord.is_some_and(|c| c == coord) {
                        Color32::from_rgb(255, 255, 200)
                    } else {
                        Color32::from_rgb(220, 220, 200) // Light tan for floor
                    }
                }
                TileType::Empty => Color32::from_rgb(10, 10, 10), // Almost black
            };

            painter.rect_filled(
                egui::Rect::from_min_max(top_left, bottom_right),
                0.0,
                fill_color,
            );
        }
    }
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

fn fit_zoom_to_dungeon(dungeon: &DungeonGrid, available_size: egui::Vec2) -> f32 {
    let base_cell_size = 30.0;
    let dungeon_w = (dungeon.width() as f32) * base_cell_size;
    let dungeon_h = (dungeon.height() as f32) * base_cell_size;
    if dungeon_w <= 0.0 || dungeon_h <= 0.0 {
        return 1.0;
    }

    let padding = 24.0;
    let fit_w = (available_size.x - padding).max(32.0) / dungeon_w;
    let fit_h = (available_size.y - padding).max(32.0) / dungeon_h;
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
