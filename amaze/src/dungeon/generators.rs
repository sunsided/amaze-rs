use crate::dungeon::{DungeonGrid, DungeonType, DynDungeonGrid, TileType};
use crate::grid_coord_2d::{GetCoordinateBounds2D, GridCoord2D};
use rand::prelude::IndexedRandom;
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

/// Generation step for dungeon creation (for animation support).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DungeonGenerationStep {
    /// A floor tile was placed
    PlaceFloor { coord: GridCoord2D },
    /// A room was stamped
    StampRoom {
        center: GridCoord2D,
        half_width: usize,
        half_height: usize,
    },
    /// A wall was placed
    PlaceWall { coord: GridCoord2D },
    /// Exit position was set
    SetExit { coord: GridCoord2D },
    /// Generation complete
    Complete,
}

/// Visitor for dungeon generation steps (for animation).
pub trait DungeonGenerationVisitor {
    fn on_step(&mut self, step: &DungeonGenerationStep);
}

/// Default visitor that collects steps into a vector.
#[derive(Default)]
pub struct VecDungeonGenerationVisitor {
    steps: Vec<DungeonGenerationStep>,
}

impl VecDungeonGenerationVisitor {
    pub fn into_steps(self) -> Vec<DungeonGenerationStep> {
        self.steps
    }
}

impl DungeonGenerationVisitor for VecDungeonGenerationVisitor {
    fn on_step(&mut self, step: &DungeonGenerationStep) {
        self.steps.push(step.clone());
    }
}

/// No-op visitor for non-instrumented generation.
struct NoOpVisitor;

impl DungeonGenerationVisitor for NoOpVisitor {
    #[inline]
    fn on_step(&mut self, _step: &DungeonGenerationStep) {
        // No-op: don't record steps
    }
}

/// Iterator over dungeon generation steps.
pub struct DungeonGenerationSteps {
    inner: std::vec::IntoIter<DungeonGenerationStep>,
}

impl DungeonGenerationSteps {
    pub fn new(steps: Vec<DungeonGenerationStep>) -> Self {
        Self {
            inner: steps.into_iter(),
        }
    }
}

impl Iterator for DungeonGenerationSteps {
    type Item = DungeonGenerationStep;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

/// Trait for dungeon generators.
pub trait DungeonGenerator {
    /// Create a new generator with a random seed.
    fn new_random() -> Self
    where
        Self: Sized;

    /// Create a new generator with a specific seed.
    fn new_from_seed(seed: u64) -> Self
    where
        Self: Sized;

    /// Generate a dungeon.
    fn generate(&self, width: usize, height: usize, floor_count: usize) -> DungeonGrid;

    /// Generate a dungeon with step-by-step events for animation.
    fn generate_steps(
        &self,
        width: usize,
        height: usize,
        floor_count: usize,
    ) -> DungeonGenerationSteps {
        let _ = self.generate(width, height, floor_count);
        DungeonGenerationSteps::new(vec![DungeonGenerationStep::Complete])
    }

    /// Get the dungeon type this generator produces.
    fn dungeon_type(&self) -> DungeonType;

    /// Get a human-readable name for this generator.
    fn name(&self) -> &'static str {
        self.dungeon_type().name()
    }

    /// Get a description of this generator.
    fn description(&self) -> &'static str {
        self.dungeon_type().description()
    }
}

/// Procedural dungeon generator implementing the Unity random walk algorithms.
///
/// Supports three generation modes:
/// - Caverns: Unconstrained random walk
/// - Rooms: Long corridors with stamped rectangular rooms
/// - Winding: Like Rooms, but with probabilistic room suppression
pub struct DungeonWalkGenerator {
    rng_seed: u64,
    dungeon_type: DungeonType,
    /// Winding hall probability (0-99). Only used for Winding type.
    /// Probability of creating a room is (99 - winding_hall_probability) / 100.
    winding_hall_probability: u8,
    /// Minimum long walk distance (inclusive). Only used for Rooms/Winding types.
    /// Unity default: 9
    long_walk_min: usize,
    /// Maximum long walk distance (exclusive upper bound). Only used for Rooms/Winding types.
    /// Unity default: 18 (so range is 9..18, meaning 9-17 inclusive)
    long_walk_max: usize,
    /// Enable dynamic grid resizing during generation
    dynamic_resize: bool,
    /// Initial grid size when dynamic resize is enabled (default 32)
    initial_grid_size: usize,
    /// Padding around final trimmed dungeon (default 0)
    trim_padding: usize,
}

/// Helper struct to reduce parameter count in internal methods
struct WalkContext<'a, V> {
    rng: &'a mut StdRng,
    grid: &'a mut DungeonGrid,
    visitor: &'a mut V,
    width: usize,
    height: usize,
    last_floor: GridCoord2D,
}

/// Helper struct for dynamic grid generation
struct DynWalkContext<'a, V> {
    rng: &'a mut StdRng,
    dyn_grid: &'a mut DynDungeonGrid,
    visitor: &'a mut V,
    last_floor_world: (isize, isize),
    initial_size: usize,
}

impl DungeonWalkGenerator {
    /// Create a new generator of the specified type with a random seed.
    pub fn new_random(dungeon_type: DungeonType) -> Self {
        Self {
            rng_seed: rand::random(),
            dungeon_type,
            winding_hall_probability: 50, // Default Unity value
            long_walk_min: 9,             // Default Unity value
            long_walk_max: 18,            // Default Unity value (exclusive upper bound)
            dynamic_resize: false,
            initial_grid_size: 32,
            trim_padding: 0,
        }
    }

    /// Create a new generator with a specific seed.
    pub fn new_from_seed(dungeon_type: DungeonType, seed: u64) -> Self {
        let rng_seed = if seed == 0 { rand::random() } else { seed };

        Self {
            rng_seed,
            dungeon_type,
            winding_hall_probability: 50,
            long_walk_min: 9,
            long_walk_max: 18,
            dynamic_resize: false,
            initial_grid_size: 32,
            trim_padding: 0,
        }
    }

    /// Set the winding hall probability (0-99). Only affects Winding type.
    pub fn with_winding_probability(mut self, probability: u8) -> Self {
        self.winding_hall_probability = probability.min(99);
        self
    }

    /// Set the long walk range (min, max_exclusive). Only affects Rooms/Winding types.
    /// The walk length will be randomly chosen from [min, max_exclusive).
    ///
    /// # Arguments
    /// * `min` - Minimum walk length (inclusive), clamped to at least 1
    /// * `max_exclusive` - Maximum walk length (exclusive), clamped to at least min+1
    ///
    /// Values are clamped to valid ranges instead of panicking for robustness.
    pub fn with_long_walk_range(mut self, min: usize, max_exclusive: usize) -> Self {
        let min = min.max(1); // At least 1 step
        let max_exclusive = max_exclusive.max(min + 1); // At least min+1 to ensure valid range
        self.long_walk_min = min;
        self.long_walk_max = max_exclusive;
        self
    }

    /// Enable or disable dynamic grid resizing during generation.
    /// When enabled, the grid starts small and expands as the walker approaches boundaries.
    /// The final grid is trimmed to the tight bounding box of content.
    pub fn with_dynamic_resize(mut self, enabled: bool) -> Self {
        self.dynamic_resize = enabled;
        self
    }

    /// Set the initial grid size when dynamic resize is enabled.
    /// The grid starts as a square of this size and expands as needed.
    pub fn with_initial_grid_size(mut self, size: usize) -> Self {
        self.initial_grid_size = size.max(8); // Minimum 8x8
        self
    }

    /// Set the padding around the final trimmed dungeon.
    /// This adds empty tiles around the content bounding box.
    pub fn with_trim_padding(mut self, padding: usize) -> Self {
        self.trim_padding = padding;
        self
    }

    /// Generate a dungeon with the configured parameters.
    pub fn generate(&self, width: usize, height: usize, floor_count: usize) -> DungeonGrid {
        self.generate_internal(width, height, floor_count, &mut NoOpVisitor, false)
    }

    /// Generate with animation steps.
    pub fn generate_steps(
        &self,
        width: usize,
        height: usize,
        floor_count: usize,
    ) -> DungeonGenerationSteps {
        let mut visitor = VecDungeonGenerationVisitor::default();
        let _ = self.generate_internal(width, height, floor_count, &mut visitor, true);
        DungeonGenerationSteps::new(visitor.into_steps())
    }

    fn generate_internal<V: DungeonGenerationVisitor>(
        &self,
        width: usize,
        height: usize,
        floor_count: usize,
        visitor: &mut V,
        emit_wall_steps: bool,
    ) -> DungeonGrid {
        let mut rng = StdRng::seed_from_u64(self.rng_seed);

        if width == 0 || height == 0 || floor_count == 0 {
            let grid = DungeonGrid::new(width, height);
            visitor.on_step(&DungeonGenerationStep::Complete);
            return grid;
        }

        if self.dynamic_resize {
            self.generate_dynamic(floor_count, visitor, emit_wall_steps, &mut rng)
        } else {
            self.generate_fixed(
                width,
                height,
                floor_count,
                visitor,
                emit_wall_steps,
                &mut rng,
            )
        }
    }

    fn generate_fixed<V: DungeonGenerationVisitor>(
        &self,
        width: usize,
        height: usize,
        floor_count: usize,
        visitor: &mut V,
        emit_wall_steps: bool,
        rng: &mut StdRng,
    ) -> DungeonGrid {
        let mut grid = DungeonGrid::new(width, height);

        // Cap floor_count to max possible tiles (leaving some margin for walls)
        let max_possible_floor = width.saturating_mul(height).saturating_mul(9) / 10; // 90% max
        let target_floor_count = floor_count.min(max_possible_floor);

        // Start at center
        let mut walker_pos = GridCoord2D::new(width / 2, height / 2);
        let mut last_floor_pos = walker_pos;

        // Place initial floor
        grid.set(walker_pos, TileType::Floor);
        visitor.on_step(&DungeonGenerationStep::PlaceFloor { coord: walker_pos });

        // Track iterations to prevent infinite loops
        let mut iterations_since_progress = 0;
        let max_stalled_iterations = target_floor_count
            .max(width.saturating_mul(height) / 10)
            .max(1000);

        // Generate floor tiles
        while grid.floor_count() < target_floor_count {
            let floor_count_before = grid.floor_count();
            iterations_since_progress += 1;

            if iterations_since_progress > max_stalled_iterations {
                break;
            }
            match self.dungeon_type {
                DungeonType::Caverns => {
                    walker_pos = self.take_step(rng, walker_pos, width, height);
                    if !grid.is_floor(walker_pos) {
                        grid.set(walker_pos, TileType::Floor);
                        visitor.on_step(&DungeonGenerationStep::PlaceFloor { coord: walker_pos });
                        last_floor_pos = walker_pos;
                    }
                }
                DungeonType::Rooms | DungeonType::Winding => {
                    let mut ctx = WalkContext {
                        rng,
                        grid: &mut grid,
                        visitor,
                        width,
                        height,
                        last_floor: last_floor_pos,
                    };

                    walker_pos = self.take_long_walk(&mut ctx, walker_pos, target_floor_count);

                    let should_stamp_room = if self.dungeon_type == DungeonType::Rooms {
                        true
                    } else {
                        let roll = ctx.rng.random_range(0..100);
                        roll > self.winding_hall_probability
                    };

                    if should_stamp_room && ctx.grid.floor_count() < target_floor_count {
                        self.stamp_room(&mut ctx, walker_pos, target_floor_count);
                    }

                    last_floor_pos = ctx.last_floor;
                }
            }

            if grid.floor_count() > floor_count_before {
                iterations_since_progress = 0;
            }
        }

        // Set exit before trimming
        grid.set_exit(last_floor_pos);

        // Trim to content bounds (also calls place_walls and compute_edge_masks)
        grid = grid.trim(self.trim_padding);

        // Emit wall steps from trimmed grid
        if emit_wall_steps {
            for y in 0..grid.height() {
                for x in 0..grid.width() {
                    let coord = GridCoord2D::new(x, y);
                    if grid.get(coord).unwrap().is_wall() {
                        visitor.on_step(&DungeonGenerationStep::PlaceWall { coord });
                    }
                }
            }
        }

        let trimmed_exit = grid.exit().unwrap_or(last_floor_pos);
        visitor.on_step(&DungeonGenerationStep::SetExit {
            coord: trimmed_exit,
        });

        visitor.on_step(&DungeonGenerationStep::Complete);
        grid
    }

    fn generate_dynamic<V: DungeonGenerationVisitor>(
        &self,
        floor_count: usize,
        visitor: &mut V,
        emit_wall_steps: bool,
        rng: &mut StdRng,
    ) -> DungeonGrid {
        let initial_size = self.initial_grid_size;
        let mut dyn_grid = DynDungeonGrid::new(initial_size, initial_size);

        // Cap floor_count based on a reasonable maximum
        let max_possible_floor = initial_size.saturating_mul(initial_size).saturating_mul(9) / 10;
        let target_floor_count = floor_count.min(max_possible_floor.max(10000));

        // Start at world position (0, 0)
        let mut walker_world = (0isize, 0isize);
        let mut last_floor_world = (0isize, 0isize);

        // Place initial floor
        dyn_grid.ensure_bounds(0, 0, 0, 0);
        dyn_grid.set_world(0, 0, TileType::Floor);
        dyn_grid.update_floor_bounds(0, 0);
        visitor.on_step(&DungeonGenerationStep::PlaceFloor {
            coord: GridCoord2D::new(initial_size / 2, initial_size / 2),
        });

        // Stall limit - use a larger default for dynamic mode
        let mut iterations_since_progress = 0;
        let max_stalled_iterations = target_floor_count.max(5000);

        // Track floor count directly from dyn_grid
        while dyn_grid.inner().floor_count() < target_floor_count {
            let floor_count_before = dyn_grid.inner().floor_count();
            iterations_since_progress += 1;

            if iterations_since_progress > max_stalled_iterations {
                break;
            }

            match self.dungeon_type {
                DungeonType::Caverns => {
                    walker_world = self.take_step_world(rng, walker_world);
                    // Ensure bounds before setting
                    dyn_grid.ensure_bounds(
                        walker_world.0,
                        walker_world.1,
                        walker_world.0,
                        walker_world.1,
                    );
                    if !dyn_grid.is_floor_world(walker_world.0, walker_world.1) {
                        dyn_grid.set_world(walker_world.0, walker_world.1, TileType::Floor);
                        dyn_grid.update_floor_bounds(walker_world.0, walker_world.1);
                        // Emit event with a synthetic grid coord (center-based)
                        visitor.on_step(&DungeonGenerationStep::PlaceFloor {
                            coord: GridCoord2D::new(initial_size / 2, initial_size / 2),
                        });
                        last_floor_world = walker_world;
                    }
                }
                DungeonType::Rooms | DungeonType::Winding => {
                    let mut ctx = DynWalkContext {
                        rng,
                        dyn_grid: &mut dyn_grid,
                        visitor,
                        last_floor_world,
                        initial_size,
                    };

                    walker_world =
                        self.take_long_walk_world(&mut ctx, walker_world, target_floor_count);

                    let should_stamp_room = if self.dungeon_type == DungeonType::Rooms {
                        true
                    } else {
                        let roll = ctx.rng.random_range(0..100);
                        roll > self.winding_hall_probability
                    };

                    if should_stamp_room && ctx.dyn_grid.inner().floor_count() < target_floor_count
                    {
                        self.stamp_room_world(&mut ctx, walker_world, target_floor_count);
                    }

                    last_floor_world = ctx.last_floor_world;
                }
            }

            if dyn_grid.inner().floor_count() > floor_count_before {
                iterations_since_progress = 0;
            }
        }

        // Set exit world position
        dyn_grid.set_exit_world(last_floor_world.0, last_floor_world.1);
        visitor.on_step(&DungeonGenerationStep::SetExit {
            coord: GridCoord2D::new(initial_size / 2, initial_size / 2),
        });

        // Finalize: trim to content bounds with padding
        let final_grid = dyn_grid.finalize(self.trim_padding);

        // Emit wall steps if needed (from final grid)
        if emit_wall_steps {
            for y in 0..final_grid.height() {
                for x in 0..final_grid.width() {
                    let coord = GridCoord2D::new(x, y);
                    if final_grid.get(coord).unwrap().is_wall() {
                        visitor.on_step(&DungeonGenerationStep::PlaceWall { coord });
                    }
                }
            }
        }

        visitor.on_step(&DungeonGenerationStep::Complete);
        final_grid
    }

    /// Take a single random step in world coordinates (no clamping).
    fn take_step_world(&self, rng: &mut StdRng, pos: (isize, isize)) -> (isize, isize) {
        let directions: [(isize, isize); 4] = [
            (0, -1), // up
            (1, 0),  // right
            (0, 1),  // down
            (-1, 0), // left
        ];

        let &(dx, dy) = directions.choose(rng).unwrap();
        (pos.0 + dx, pos.1 + dy)
    }

    /// Take a single random step in one of 4 directions, staying in bounds.
    fn take_step(
        &self,
        rng: &mut StdRng,
        pos: GridCoord2D,
        width: usize,
        height: usize,
    ) -> GridCoord2D {
        let directions: [(isize, isize); 4] = [
            (0, -1), // up
            (1, 0),  // right
            (0, 1),  // down
            (-1, 0), // left
        ];

        let &(dx, dy) = directions.choose(rng).unwrap();
        let new_x = (pos.x as isize + dx).max(0).min(width as isize - 1) as usize;
        let new_y = (pos.y as isize + dy).max(0).min(height as isize - 1) as usize;

        GridCoord2D::new(new_x, new_y)
    }

    /// Take a long walk (Unity: Random.Range(9, 18) => 9..=17 steps).
    /// Unity picks one direction and walks straight in that direction.
    /// Stops early if target floor count is reached.
    fn take_long_walk<V: DungeonGenerationVisitor>(
        &self,
        ctx: &mut WalkContext<V>,
        start: GridCoord2D,
        target_floor_count: usize,
    ) -> GridCoord2D {
        let walk_length = ctx.rng.random_range(self.long_walk_min..self.long_walk_max);

        // Pick ONE direction for this entire long walk (Unity behavior)
        let directions: [(isize, isize); 4] = [
            (0, -1), // up
            (1, 0),  // right
            (0, 1),  // down
            (-1, 0), // left
        ];
        let &(dx, dy) = directions.choose(ctx.rng).unwrap();

        let mut pos = start;

        // Walk straight in that direction for walk_length steps
        for _ in 0..walk_length {
            // Stop early if we've reached the target floor count
            if ctx.grid.floor_count() >= target_floor_count {
                break;
            }

            let new_x = (pos.x as isize + dx).max(0).min(ctx.width as isize - 1) as usize;
            let new_y = (pos.y as isize + dy).max(0).min(ctx.height as isize - 1) as usize;
            pos = GridCoord2D::new(new_x, new_y);

            if !ctx.grid.is_floor(pos) {
                ctx.grid.set(pos, TileType::Floor);
                ctx.visitor
                    .on_step(&DungeonGenerationStep::PlaceFloor { coord: pos });
                ctx.last_floor = pos;
            }
        }

        pos
    }

    /// Stamp a rectangular room centered at pos.
    /// Unity: half-sizes are Random.Range(1, 5) => 1..=4, so room size is (2*hw+1) x (2*hh+1).
    /// Stops stamping if floor count reaches target to respect cap.
    fn stamp_room<V: DungeonGenerationVisitor>(
        &self,
        ctx: &mut WalkContext<V>,
        center: GridCoord2D,
        target_floor_count: usize,
    ) {
        let half_width = ctx.rng.random_range(1..5); // Unity: Random.Range(1,5) = 1..=4
        let half_height = ctx.rng.random_range(1..5);

        ctx.visitor.on_step(&DungeonGenerationStep::StampRoom {
            center,
            half_width,
            half_height,
        });

        let min_x = center.x.saturating_sub(half_width);
        let max_x = (center.x + half_width).min(ctx.grid.width() - 1);
        let min_y = center.y.saturating_sub(half_height);
        let max_y = (center.y + half_height).min(ctx.grid.height() - 1);

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                // Stop stamping if we've reached the target floor count
                if ctx.grid.floor_count() >= target_floor_count {
                    return;
                }

                let coord = GridCoord2D::new(x, y);
                if !ctx.grid.is_floor(coord) {
                    ctx.grid.set(coord, TileType::Floor);
                    ctx.visitor
                        .on_step(&DungeonGenerationStep::PlaceFloor { coord });
                    ctx.last_floor = coord;
                }
            }
        }
    }

    /// Take a long walk in world coordinates (no clamping, expands grid as needed).
    fn take_long_walk_world<V: DungeonGenerationVisitor>(
        &self,
        ctx: &mut DynWalkContext<V>,
        start: (isize, isize),
        target_floor_count: usize,
    ) -> (isize, isize) {
        let walk_length = ctx.rng.random_range(self.long_walk_min..self.long_walk_max);

        let directions: [(isize, isize); 4] = [
            (0, -1), // up
            (1, 0),  // right
            (0, 1),  // down
            (-1, 0), // left
        ];
        let &(dx, dy) = directions.choose(ctx.rng).unwrap();

        let mut pos = start;

        for _ in 0..walk_length {
            if ctx.dyn_grid.inner().floor_count() >= target_floor_count {
                break;
            }

            pos = (pos.0 + dx, pos.1 + dy);

            // Ensure grid has space
            ctx.dyn_grid.ensure_bounds(pos.0, pos.1, pos.0, pos.1);

            if !ctx.dyn_grid.is_floor_world(pos.0, pos.1) {
                ctx.dyn_grid.set_world(pos.0, pos.1, TileType::Floor);
                ctx.dyn_grid.update_floor_bounds(pos.0, pos.1);
                ctx.visitor.on_step(&DungeonGenerationStep::PlaceFloor {
                    coord: GridCoord2D::new(ctx.initial_size / 2, ctx.initial_size / 2),
                });
                ctx.last_floor_world = pos;
            }
        }

        pos
    }

    /// Stamp a rectangular room in world coordinates.
    fn stamp_room_world<V: DungeonGenerationVisitor>(
        &self,
        ctx: &mut DynWalkContext<V>,
        center: (isize, isize),
        target_floor_count: usize,
    ) {
        let half_width = ctx.rng.random_range(1..5);
        let half_height = ctx.rng.random_range(1..5);

        ctx.visitor.on_step(&DungeonGenerationStep::StampRoom {
            center: GridCoord2D::new(ctx.initial_size / 2, ctx.initial_size / 2),
            half_width,
            half_height,
        });

        let min_x = center.0 - half_width as isize;
        let max_x = center.0 + half_width as isize;
        let min_y = center.1 - half_height as isize;
        let max_y = center.1 + half_height as isize;

        // Ensure bounds for the entire room
        ctx.dyn_grid.ensure_bounds(min_x, min_y, max_x, max_y);

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if ctx.dyn_grid.inner().floor_count() >= target_floor_count {
                    return;
                }

                if !ctx.dyn_grid.is_floor_world(x, y) {
                    ctx.dyn_grid.set_world(x, y, TileType::Floor);
                    ctx.dyn_grid.update_floor_bounds(x, y);
                    ctx.visitor.on_step(&DungeonGenerationStep::PlaceFloor {
                        coord: GridCoord2D::new(ctx.initial_size / 2, ctx.initial_size / 2),
                    });
                    ctx.last_floor_world = (x, y);
                }
            }
        }
    }
}

impl DungeonGenerator for DungeonWalkGenerator {
    fn new_random() -> Self {
        Self::new_random(DungeonType::Caverns)
    }

    fn new_from_seed(seed: u64) -> Self {
        Self::new_from_seed(DungeonType::Caverns, seed)
    }

    fn generate(&self, width: usize, height: usize, floor_count: usize) -> DungeonGrid {
        DungeonWalkGenerator::generate(self, width, height, floor_count)
    }

    fn generate_steps(
        &self,
        width: usize,
        height: usize,
        floor_count: usize,
    ) -> DungeonGenerationSteps {
        DungeonWalkGenerator::generate_steps(self, width, height, floor_count)
    }

    fn dungeon_type(&self) -> DungeonType {
        self.dungeon_type
    }
}

#[cfg(test)]
mod generator_tests {
    use super::*;

    #[test]
    fn test_generator_dynamic_produces_tight_grid() {
        let generator = DungeonWalkGenerator::new_random(DungeonType::Caverns)
            .with_dynamic_resize(true)
            .with_initial_grid_size(32)
            .with_trim_padding(0);

        // Request more floors than the initial 32x32 grid can hold at center
        let grid = generator.generate(40, 30, 500);

        // The grid should be trimmed to fit the content tightly
        // With 500 floors, the grid should be reasonably sized
        assert!(grid.width() <= 100);
        assert!(grid.height() <= 100);
        assert!(grid.floor_count() > 0);
    }

    #[test]
    fn test_generator_dynamic_with_padding() {
        let generator = DungeonWalkGenerator::new_random(DungeonType::Caverns)
            .with_dynamic_resize(true)
            .with_initial_grid_size(32)
            .with_trim_padding(2);

        let grid = generator.generate(40, 30, 100);

        assert!(grid.width() > 0);
        assert!(grid.height() > 0);
        assert!(grid.floor_count() > 0);
    }

    #[test]
    fn test_generator_fixed_mode_trims_output() {
        let generator = DungeonWalkGenerator::new_from_seed(DungeonType::Caverns, 42)
            .with_dynamic_resize(false)
            .with_trim_padding(0);

        let grid = generator.generate(40, 30, 200);

        // Fixed mode should trim to content bounds (at least one dimension should be smaller)
        assert!(grid.width() <= 40);
        assert!(grid.height() <= 30);
        assert!(grid.floor_count() > 0);
        // Verify trimming actually occurred (at least one dimension reduced)
        assert!(grid.width() < 40 || grid.height() < 30);
    }

    #[test]
    fn test_generator_fixed_mode_with_padding() {
        let generator = DungeonWalkGenerator::new_random(DungeonType::Caverns)
            .with_dynamic_resize(false)
            .with_trim_padding(2);

        let grid = generator.generate(40, 30, 200);

        assert!(grid.width() > 0);
        assert!(grid.height() > 0);
        assert!(grid.floor_count() > 0);
    }

    #[test]
    fn test_generator_dynamic_rooms_type() {
        let generator = DungeonWalkGenerator::new_random(DungeonType::Rooms)
            .with_dynamic_resize(true)
            .with_initial_grid_size(32);

        let grid = generator.generate(40, 30, 300);

        assert!(grid.floor_count() > 0);
        assert!(grid.width() > 0);
        assert!(grid.height() > 0);
    }

    #[test]
    fn test_generator_dynamic_winding_type() {
        let generator = DungeonWalkGenerator::new_random(DungeonType::Winding)
            .with_dynamic_resize(true)
            .with_initial_grid_size(32)
            .with_winding_probability(50);

        let grid = generator.generate(40, 30, 300);

        assert!(grid.floor_count() > 0);
        assert!(grid.width() > 0);
        assert!(grid.height() > 0);
    }
}
