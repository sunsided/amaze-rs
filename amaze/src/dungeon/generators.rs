use crate::dungeon::{DungeonGrid, DungeonType, TileType};
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

impl DungeonWalkGenerator {
    /// Create a new generator of the specified type with a random seed.
    pub fn new_random(dungeon_type: DungeonType) -> Self {
        Self {
            rng_seed: rand::random(),
            dungeon_type,
            winding_hall_probability: 50, // Default Unity value
            long_walk_min: 9,             // Default Unity value
            long_walk_max: 18,            // Default Unity value (exclusive upper bound)
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
        let mut grid = DungeonGrid::new(width, height);
        let mut rng = StdRng::seed_from_u64(self.rng_seed);

        if width == 0 || height == 0 || floor_count == 0 {
            visitor.on_step(&DungeonGenerationStep::Complete);
            return grid;
        }

        // Cap floor_count to max possible tiles (leaving some margin for walls)
        // Use saturating_mul to prevent overflow on large dimensions
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
        // Dynamic stall limit: scale with problem size
        // Use target floor count as baseline, with minimum of 1000 and scaling by area
        let max_stalled_iterations = target_floor_count
            .max(width.saturating_mul(height) / 10)
            .max(1000);

        // Generate floor tiles
        while grid.floor_count() < target_floor_count {
            let floor_count_before = grid.floor_count();
            iterations_since_progress += 1;

            // Safety check: if we've been stuck for too long, exit
            if iterations_since_progress > max_stalled_iterations {
                break;
            }
            match self.dungeon_type {
                DungeonType::Caverns => {
                    // Simple random walk
                    walker_pos = self.take_step(&mut rng, walker_pos, width, height);
                    if !grid.is_floor(walker_pos) {
                        grid.set(walker_pos, TileType::Floor);
                        visitor.on_step(&DungeonGenerationStep::PlaceFloor { coord: walker_pos });
                        last_floor_pos = walker_pos;
                    }
                }
                DungeonType::Rooms | DungeonType::Winding => {
                    // Create context for helper methods
                    let mut ctx = WalkContext {
                        rng: &mut rng,
                        grid: &mut grid,
                        visitor,
                        width,
                        height,
                        last_floor: last_floor_pos,
                    };

                    // Take a long walk
                    walker_pos = self.take_long_walk(&mut ctx, walker_pos, target_floor_count);

                    // Maybe stamp a room
                    let should_stamp_room = if self.dungeon_type == DungeonType::Rooms {
                        true
                    } else {
                        // Winding: probabilistic room suppression
                        let roll = ctx.rng.random_range(0..100);
                        roll > self.winding_hall_probability
                    };

                    if should_stamp_room && ctx.grid.floor_count() < target_floor_count {
                        self.stamp_room(&mut ctx, walker_pos, target_floor_count);
                    }

                    // Extract last_floor from context
                    last_floor_pos = ctx.last_floor;
                }
            }

            // Check if we made progress
            if grid.floor_count() > floor_count_before {
                iterations_since_progress = 0;
            }
        }

        // Post-processing: place walls
        grid.place_walls();
        // Emit wall placement events for animation (optional, can be batched)
        if emit_wall_steps {
            for y in 0..height {
                for x in 0..width {
                    let coord = GridCoord2D::new(x, y);
                    if grid.get(coord).unwrap().is_wall() {
                        visitor.on_step(&DungeonGenerationStep::PlaceWall { coord });
                    }
                }
            }
        }

        // Set exit at last floor position
        grid.set_exit(last_floor_pos);
        visitor.on_step(&DungeonGenerationStep::SetExit {
            coord: last_floor_pos,
        });

        // Compute edge masks for wall rendering
        grid.compute_edge_masks();

        visitor.on_step(&DungeonGenerationStep::Complete);
        grid
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
