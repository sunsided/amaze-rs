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
}

impl DungeonWalkGenerator {
    /// Create a new generator of the specified type with a random seed.
    pub fn new_random(dungeon_type: DungeonType) -> Self {
        Self {
            rng: StdRng::from_os_rng(),
            dungeon_type,
            winding_hall_probability: 50, // Default Unity value
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
        }
    }

    /// Set the winding hall probability (0-99). Only affects Winding type.
    pub fn with_winding_probability(mut self, probability: u8) -> Self {
        self.winding_hall_probability = probability.min(99);
        self
    }

    /// Generate a dungeon with the configured parameters.
    pub fn generate(&self, width: usize, height: usize, floor_count: usize) -> DungeonGrid {
        self.generate_with_steps(width, height, floor_count).0
    }

    /// Generate with animation steps.
    pub fn generate_steps(
        &self,
        width: usize,
        height: usize,
        floor_count: usize,
    ) -> DungeonGenerationSteps {
        DungeonGenerationSteps::new(self.generate_with_steps(width, height, floor_count).1)
    }

    fn generate_with_steps(
        &self,
        width: usize,
        height: usize,
        floor_count: usize,
    ) -> (DungeonGrid, Vec<DungeonGenerationStep>) {
        let mut grid = DungeonGrid::new(width, height);
        let mut visitor = VecDungeonGenerationVisitor::default();
        let mut rng = self.rng.clone();

        if width == 0 || height == 0 || floor_count == 0 {
            visitor.on_step(&DungeonGenerationStep::Complete);
            return (grid, visitor.into_steps());
        }

        // Cap floor_count to max possible tiles (leaving some margin for walls)
        let max_possible_floor = (width * height).saturating_mul(9) / 10; // 90% max
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
                    // Take a long walk
                    walker_pos = self.take_long_walk(
                        &mut rng,
                        &mut grid,
                        &mut visitor,
                        walker_pos,
                        width,
                        height,
                        &mut last_floor_pos,
                    );

                    // Maybe stamp a room
                    let should_stamp_room = if self.dungeon_type == DungeonType::Rooms {
                        true
                    } else {
                        // Winding: probabilistic room suppression
                        let roll = rng.random_range(0..100);
                        roll > self.winding_hall_probability
                    };

                    if should_stamp_room && grid.floor_count() < target_floor_count {
                        self.stamp_room(
                            &mut rng,
                            &mut grid,
                            &mut visitor,
                            walker_pos,
                            &mut last_floor_pos,
                        );
                    }
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
        (grid, visitor.into_steps())
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
    fn take_long_walk<V: DungeonGenerationVisitor>(
        &self,
        rng: &mut StdRng,
        grid: &mut DungeonGrid,
        visitor: &mut V,
        start: GridCoord2D,
        width: usize,
        height: usize,
        last_floor: &mut GridCoord2D,
    ) -> GridCoord2D {
        let walk_length = rng.random_range(9..18); // Unity: Random.Range(9,18) = 9..=17
        let mut pos = start;

        for _ in 0..walk_length {
            pos = self.take_step(rng, pos, width, height);
            if !grid.is_floor(pos) {
                grid.set(pos, TileType::Floor);
                visitor.on_step(&DungeonGenerationStep::PlaceFloor { coord: pos });
                *last_floor = pos;
            }
        }

        pos
    }

    /// Stamp a rectangular room centered at pos.
    /// Unity: half-sizes are Random.Range(1, 5) => 1..=4, so room size is (2*hw+1) x (2*hh+1).
    fn stamp_room<V: DungeonGenerationVisitor>(
        &self,
        rng: &mut StdRng,
        grid: &mut DungeonGrid,
        visitor: &mut V,
        center: GridCoord2D,
        last_floor: &mut GridCoord2D,
    ) {
        let half_width = rng.random_range(1..5); // Unity: Random.Range(1,5) = 1..=4
        let half_height = rng.random_range(1..5);

        visitor.on_step(&DungeonGenerationStep::StampRoom {
            center,
            half_width,
            half_height,
        });

        let min_x = center.x.saturating_sub(half_width);
        let max_x = (center.x + half_width).min(grid.width() - 1);
        let min_y = center.y.saturating_sub(half_height);
        let max_y = (center.y + half_height).min(grid.height() - 1);

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let coord = GridCoord2D::new(x, y);
                if !grid.is_floor(coord) {
                    grid.set(coord, TileType::Floor);
                    visitor.on_step(&DungeonGenerationStep::PlaceFloor { coord });
                    *last_floor = coord;
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
}
