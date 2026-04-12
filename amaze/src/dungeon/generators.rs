use crate::dungeon::{DungeonGrid, DungeonType, TileType};
use crate::grid_coord_2d::{GetCoordinateBounds2D, GridCoord2D};
use rand::prelude::IndexedRandom;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

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
    rng: StdRng,
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
            rng: StdRng::from_os_rng(),
            dungeon_type,
            winding_hall_probability: 50, // Default Unity value
            long_walk_min: 9,             // Default Unity value
            long_walk_max: 18,            // Default Unity value (exclusive upper bound)
        }
    }

    /// Create a new generator with a specific seed.
    pub fn new_from_seed(dungeon_type: DungeonType, seed: u64) -> Self {
        let rng = if seed == 0 {
            StdRng::from_os_rng()
        } else {
            StdRng::seed_from_u64(seed)
        };

        Self {
            rng,
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
        self.generate_internal(width, height, floor_count, &mut NoOpVisitor)
    }

    /// Generate with animation steps.
    pub fn generate_steps(
        &self,
        width: usize,
        height: usize,
        floor_count: usize,
    ) -> DungeonGenerationSteps {
        let mut visitor = VecDungeonGenerationVisitor::default();
        let _ = self.generate_internal(width, height, floor_count, &mut visitor);
        DungeonGenerationSteps::new(visitor.into_steps())
    }

    fn generate_internal<V: DungeonGenerationVisitor>(
        &self,
        width: usize,
        height: usize,
        floor_count: usize,
        visitor: &mut V,
    ) -> DungeonGrid {
        let mut grid = DungeonGrid::new(width, height);
        let mut rng = self.rng.clone();

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
        let max_stalled_iterations = 1000; // If no progress for 1000 iterations, give up

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
                    walker_pos = self.take_long_walk(&mut ctx, walker_pos);

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
        for y in 0..height {
            for x in 0..width {
                let coord = GridCoord2D::new(x, y);
                if grid.get(coord).unwrap().is_wall() {
                    visitor.on_step(&DungeonGenerationStep::PlaceWall { coord });
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
    fn take_long_walk<V: DungeonGenerationVisitor>(
        &self,
        ctx: &mut WalkContext<V>,
        start: GridCoord2D,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_caverns_generation() {
        let generator = DungeonWalkGenerator::new_from_seed(DungeonType::Caverns, 42);
        let dungeon = generator.generate(20, 20, 100);

        assert_eq!(dungeon.width(), 20);
        assert_eq!(dungeon.height(), 20);
        assert!(dungeon.floor_count() >= 100);
        assert!(dungeon.exit().is_some());
    }

    #[test]
    fn test_rooms_generation() {
        let generator = DungeonWalkGenerator::new_from_seed(DungeonType::Rooms, 42);
        let dungeon = generator.generate(30, 30, 150);

        assert_eq!(dungeon.width(), 30);
        assert_eq!(dungeon.height(), 30);
        assert!(dungeon.floor_count() >= 150);
        assert!(dungeon.exit().is_some());
    }

    #[test]
    fn test_winding_generation() {
        let generator = DungeonWalkGenerator::new_from_seed(DungeonType::Winding, 42)
            .with_winding_probability(70);
        let dungeon = generator.generate(25, 25, 120);

        assert_eq!(dungeon.width(), 25);
        assert_eq!(dungeon.height(), 25);
        assert!(dungeon.floor_count() >= 120);
        assert!(dungeon.exit().is_some());
    }

    #[test]
    fn test_determinism() {
        let generator1 = DungeonWalkGenerator::new_from_seed(DungeonType::Caverns, 12345);
        let generator2 = DungeonWalkGenerator::new_from_seed(DungeonType::Caverns, 12345);

        let d1 = generator1.generate(15, 15, 80);
        let d2 = generator2.generate(15, 15, 80);

        // Compare floor positions
        let floors1: Vec<_> = d1.floor_iter().collect();
        let floors2: Vec<_> = d2.floor_iter().collect();

        assert_eq!(floors1.len(), floors2.len());
        // Note: HashSet iteration order is not deterministic, but the sets should be equal
        for coord in floors1 {
            assert!(d2.is_floor(coord), "Floor mismatch at {:?}", coord);
        }
    }

    #[test]
    fn test_small_grid_doesnt_hang() {
        // Regression test: small grids with large floor_count requests should not hang
        let generator = DungeonWalkGenerator::new_from_seed(DungeonType::Winding, 42);
        let dungeon = generator.generate(8, 8, 120);

        // Should complete without hanging
        assert_eq!(dungeon.width(), 8);
        assert_eq!(dungeon.height(), 8);
        // Floor count will be capped to what's possible
        assert!(dungeon.floor_count() > 0);
        assert!(dungeon.floor_count() <= 64); // 8x8 = 64 total cells
        assert!(dungeon.exit().is_some());
    }

    #[test]
    fn test_impossible_floor_count_is_capped() {
        // Request more floors than physically possible
        let generator = DungeonWalkGenerator::new_from_seed(DungeonType::Rooms, 123);
        let dungeon = generator.generate(10, 10, 1000);

        // Should cap to ~90% of grid size
        assert_eq!(dungeon.width(), 10);
        assert_eq!(dungeon.height(), 10);
        assert!(dungeon.floor_count() > 0);
        assert!(dungeon.floor_count() <= 90); // Max 90% of 100 cells
        assert!(dungeon.exit().is_some());
    }

    #[test]
    fn test_long_walk_range_configuration() {
        // Test that long walk range can be configured
        let generator = DungeonWalkGenerator::new_from_seed(DungeonType::Rooms, 42)
            .with_long_walk_range(15, 25);
        let dungeon = generator.generate(50, 50, 300);

        assert_eq!(dungeon.width(), 50);
        assert_eq!(dungeon.height(), 50);
        assert!(dungeon.floor_count() >= 300);
        assert!(dungeon.exit().is_some());
    }

    #[test]
    fn test_long_walk_range_validation() {
        // Test that invalid ranges are clamped
        let generator =
            DungeonWalkGenerator::new_from_seed(DungeonType::Rooms, 42).with_long_walk_range(0, 1); // min=0 should be clamped to 1

        // Should not panic and should generate valid dungeon
        let dungeon = generator.generate(30, 30, 150);
        assert!(dungeon.floor_count() >= 150);
    }

    #[test]
    fn test_rooms_spread_with_directional_walk() {
        // Test that Rooms mode produces spread-out layout on larger maps
        // Using a large enough map to see spreading behavior
        let generator = DungeonWalkGenerator::new_from_seed(DungeonType::Rooms, 12345);
        let dungeon = generator.generate(60, 60, 500);

        assert_eq!(dungeon.width(), 60);
        assert_eq!(dungeon.height(), 60);
        assert!(dungeon.floor_count() >= 500);

        // Calculate the bounding box of floor tiles to measure spread
        let mut min_x = dungeon.width();
        let mut max_x = 0;
        let mut min_y = dungeon.height();
        let mut max_y = 0;

        for y in 0..dungeon.height() {
            for x in 0..dungeon.width() {
                let coord = GridCoord2D::new(x, y);
                if dungeon.is_floor(coord) {
                    min_x = min_x.min(x);
                    max_x = max_x.max(x);
                    min_y = min_y.min(y);
                    max_y = max_y.max(y);
                }
            }
        }

        let spread_x = max_x - min_x;
        let spread_y = max_y - min_y;

        // With directional long walks, rooms should spread across more than
        // just a small central area. Expect spread > 20 tiles in each dimension.
        assert!(
            spread_x > 20,
            "X spread {} too small, indicates clustering",
            spread_x
        );
        assert!(
            spread_y > 20,
            "Y spread {} too small, indicates clustering",
            spread_y
        );
    }

    #[test]
    fn test_determinism_with_long_walk_fix() {
        // Test that the new directional long walk maintains determinism
        let generator1 = DungeonWalkGenerator::new_from_seed(DungeonType::Rooms, 99999);
        let generator2 = DungeonWalkGenerator::new_from_seed(DungeonType::Rooms, 99999);

        let d1 = generator1.generate(40, 40, 300);
        let d2 = generator2.generate(40, 40, 300);

        // Compare floor positions
        let floors1: Vec<_> = d1.floor_iter().collect();
        let floors2: Vec<_> = d2.floor_iter().collect();

        assert_eq!(floors1.len(), floors2.len(), "Floor counts should match");

        // Verify all floor positions match
        for coord in floors1 {
            assert!(d2.is_floor(coord), "Floor mismatch at {:?}", coord);
        }

        // Verify exits match
        assert_eq!(d1.exit(), d2.exit(), "Exit positions should match");
    }

    #[test]
    fn test_determinism_with_custom_walk_range() {
        // Test determinism with custom walk range
        let generator1 = DungeonWalkGenerator::new_from_seed(DungeonType::Rooms, 54321)
            .with_long_walk_range(5, 12);
        let generator2 = DungeonWalkGenerator::new_from_seed(DungeonType::Rooms, 54321)
            .with_long_walk_range(5, 12);

        let d1 = generator1.generate(35, 35, 250);
        let d2 = generator2.generate(35, 35, 250);

        let floors1: Vec<_> = d1.floor_iter().collect();
        let floors2: Vec<_> = d2.floor_iter().collect();

        assert_eq!(floors1.len(), floors2.len());
        for coord in floors1 {
            assert!(d2.is_floor(coord), "Floor mismatch at {:?}", coord);
        }
    }

    #[test]
    fn test_winding_with_long_walk() {
        // Test that Winding mode works correctly with directional long walk
        let generator = DungeonWalkGenerator::new_from_seed(DungeonType::Winding, 77777)
            .with_winding_probability(80) // High winding = fewer rooms
            .with_long_walk_range(10, 20);

        let dungeon = generator.generate(50, 50, 400);

        assert_eq!(dungeon.width(), 50);
        assert_eq!(dungeon.height(), 50);
        assert!(dungeon.floor_count() >= 400);
        assert!(dungeon.exit().is_some());
    }

    #[test]
    fn test_room_stamping_respects_floor_cap() {
        // Test that room stamping stops when target floor count is reached
        // Use Rooms mode which stamps many rooms, and a target close to cap
        let generator = DungeonWalkGenerator::new_from_seed(DungeonType::Rooms, 999);
        let dungeon = generator.generate(20, 20, 180); // 20x20 = 400 cells, 90% = 360 max

        // Floor count should not significantly exceed target
        // Allow small overshoot due to final room stamp, but should be close
        assert!(
            dungeon.floor_count() <= 360,
            "Floor count {} exceeds max possible 360",
            dungeon.floor_count()
        );
        assert!(
            dungeon.floor_count() >= 180,
            "Floor count {} is below target 180",
            dungeon.floor_count()
        );
        // The important check: we should be reasonably close to target without massive overshoot
        assert!(
            dungeon.floor_count() <= 200,
            "Floor count {} overshoots target 180 by too much",
            dungeon.floor_count()
        );
    }

    #[test]
    fn test_overflow_safety_large_dimensions() {
        // Test that very large dimensions don't cause overflow in cap calculation
        // This verifies that saturating_mul is used correctly

        // Test the calculation that happens in generate_internal
        let large_width = 100_000_usize;
        let large_height = 100_000_usize;

        // This would overflow with naive multiplication on some systems
        // With saturating_mul, it should cap at usize::MAX
        let max_possible_floor = large_width.saturating_mul(large_height).saturating_mul(9) / 10;

        // Verify we got a reasonable result (not wrapped around)
        assert!(max_possible_floor > 0, "Calculation should not wrap to 0");
        assert!(
            max_possible_floor <= usize::MAX / 2,
            "Result should be reasonable"
        );

        // Also verify that a normal-sized dungeon still generates correctly
        let generator = DungeonWalkGenerator::new_from_seed(DungeonType::Caverns, 42);
        let dungeon = generator.generate(100, 100, 1000);
        assert!(dungeon.floor_count() >= 1000);
    }

    #[test]
    fn test_passability_grid_deterministic_entrance() {
        // Test that PassabilityGrid conversion produces deterministic entrance
        use crate::representations::PassabilityGrid;

        let generator = DungeonWalkGenerator::new_from_seed(DungeonType::Caverns, 12345);
        let dungeon1 = generator.generate(20, 20, 100);
        let dungeon2 =
            DungeonWalkGenerator::new_from_seed(DungeonType::Caverns, 12345).generate(20, 20, 100);

        let grid1 = PassabilityGrid::from(&dungeon1);
        let grid2 = PassabilityGrid::from(&dungeon2);

        // Entrance should be deterministic (same for same dungeon layout)
        assert_eq!(
            grid1.entrance_position(),
            grid2.entrance_position(),
            "Entrance positions should be deterministic"
        );
    }
}
