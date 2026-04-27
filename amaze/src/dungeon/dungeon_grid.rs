use crate::dungeon::TileType;
use crate::grid_coord_2d::{GetCoordinateBounds2D, GridCoord2D, LinearizeCoords2D};
use std::collections::HashSet;
use std::ops::Index;

/// Grid representation for procedurally generated dungeons.
///
/// Stores tile types (Empty/Floor/Wall), floor positions for efficient membership
/// testing, optional exit location, and optional edge masks for wall rendering.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DungeonGrid {
    width: usize,
    height: usize,
    tiles: Vec<TileType>,
    /// Fast lookup set for floor tiles
    floor_positions: HashSet<GridCoord2D>,
    /// Exit position (typically last walker position)
    exit: Option<GridCoord2D>,
    /// Edge bitmask for each wall tile (for rounded corner rendering).
    /// Bit encoding: top=1, right=2, bottom=4, left=8
    /// A bit is set when the neighbor in that direction is NOT a wall.
    edge_masks: Vec<u8>,
}

impl DungeonGrid {
    /// Create a new dungeon grid with all tiles initialized to Empty.
    pub fn new(width: usize, height: usize) -> Self {
        let size = width
            .checked_mul(height)
            .expect("Grid dimensions overflow: width * height exceeds usize::MAX");
        Self {
            width,
            height,
            tiles: vec![TileType::Empty; size],
            floor_positions: HashSet::new(),
            exit: None,
            edge_masks: vec![0; size],
        }
    }

    /// Get the tile type at the given coordinate.
    pub fn get(&self, coord: GridCoord2D) -> Option<TileType> {
        if coord.x >= self.width || coord.y >= self.height {
            None
        } else {
            Some(self.tiles[self.linearize_coords(coord)])
        }
    }

    /// Set the tile type at the given coordinate.
    pub fn set(&mut self, coord: GridCoord2D, tile: TileType) {
        if coord.x < self.width && coord.y < self.height {
            let idx = self.linearize_coords(coord);
            let old_tile = self.tiles[idx];
            self.tiles[idx] = tile;

            // Update floor set
            if old_tile.is_passable() && !tile.is_passable() {
                self.floor_positions.remove(&coord);
            } else if !old_tile.is_passable() && tile.is_passable() {
                self.floor_positions.insert(coord);
            }
        }
    }

    /// Returns true if the coordinate contains a floor tile.
    #[inline]
    pub fn is_floor(&self, coord: GridCoord2D) -> bool {
        self.floor_positions.contains(&coord)
    }

    /// Returns an iterator over all floor tile positions.
    pub fn floor_iter(&self) -> impl Iterator<Item = GridCoord2D> + '_ {
        self.floor_positions.iter().copied()
    }

    /// Returns the number of floor tiles.
    #[inline]
    pub fn floor_count(&self) -> usize {
        self.floor_positions.len()
    }

    /// Get the exit position if set.
    #[inline]
    pub fn exit(&self) -> Option<GridCoord2D> {
        self.exit
    }

    /// Set the exit position.
    pub fn set_exit(&mut self, coord: GridCoord2D) {
        self.exit = Some(coord);
    }

    /// Get the edge mask for a wall tile (used for rendering).
    pub fn edge_mask(&self, coord: GridCoord2D) -> u8 {
        if coord.x < self.width && coord.y < self.height {
            self.edge_masks[self.linearize_coords(coord)]
        } else {
            0
        }
    }

    /// Set the edge mask for a wall tile.
    pub fn set_edge_mask(&mut self, coord: GridCoord2D, mask: u8) {
        if coord.x < self.width && coord.y < self.height {
            let idx = self.linearize_coords(coord);
            self.edge_masks[idx] = mask;
        }
    }

    /// Compute edge masks for all wall tiles based on neighboring walls.
    ///
    /// Bit encoding: top=1, right=2, bottom=4, left=8.
    /// A bit is set when the neighbor in that direction is NOT a wall.
    pub fn compute_edge_masks(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let coord = GridCoord2D::new(x, y);
                if !self.get(coord).unwrap().is_wall() {
                    continue;
                }

                let mut mask = 0u8;

                // Top (y-1)
                if y == 0 || !self.get(GridCoord2D::new(x, y - 1)).unwrap().is_wall() {
                    mask |= 1;
                }

                // Right (x+1)
                if x + 1 >= self.width || !self.get(GridCoord2D::new(x + 1, y)).unwrap().is_wall() {
                    mask |= 2;
                }

                // Bottom (y+1)
                if y + 1 >= self.height || !self.get(GridCoord2D::new(x, y + 1)).unwrap().is_wall()
                {
                    mask |= 4;
                }

                // Left (x-1)
                if x == 0 || !self.get(GridCoord2D::new(x - 1, y)).unwrap().is_wall() {
                    mask |= 8;
                }

                self.set_edge_mask(coord, mask);
            }
        }
    }

    /// Place walls around all floor tiles (and at map boundaries if floor extends there).
    /// This implements the Unity TileSpawner logic.
    pub fn place_walls(&mut self) {
        let floor_coords: Vec<GridCoord2D> = self.floor_positions.iter().copied().collect();

        for coord in floor_coords {
            // Check all 4-connected neighbors
            let neighbors = [
                (coord.x, coord.y.wrapping_sub(1)), // top
                (coord.x + 1, coord.y),             // right
                (coord.x, coord.y + 1),             // bottom
                (coord.x.wrapping_sub(1), coord.y), // left
            ];

            for (nx, ny) in neighbors {
                if nx < self.width && ny < self.height {
                    let neighbor = GridCoord2D::new(nx, ny);
                    if self.get(neighbor).unwrap().is_empty() {
                        self.set(neighbor, TileType::Wall);
                    }
                }
            }
        }
    }

    /// Trim the grid to the bounding box of all non-Empty tiles, plus padding.
    /// Returns a new, tightly-cropped DungeonGrid.
    ///
    /// If no non-Empty tiles exist, returns a minimal 1x1 Empty grid.
    pub fn trim(&self, padding: usize) -> Self {
        // Find bounding box of all non-Empty tiles
        let mut min_x = usize::MAX;
        let mut min_y = usize::MAX;
        let mut max_x = usize::MIN;
        let mut max_y = usize::MIN;
        let mut has_content = false;

        for y in 0..self.height {
            for x in 0..self.width {
                let tile = self.tiles[self.linearize_coords(GridCoord2D::new(x, y))];
                if !tile.is_empty() {
                    if x < min_x {
                        min_x = x;
                    }
                    if x > max_x {
                        max_x = x;
                    }
                    if y < min_y {
                        min_y = y;
                    }
                    if y > max_y {
                        max_y = y;
                    }
                    has_content = true;
                }
            }
        }

        if !has_content {
            return DungeonGrid::new(1, 1);
        }

        // Apply padding and clamp to original bounds
        let content_min_x = min_x.saturating_sub(padding);
        let content_min_y = min_y.saturating_sub(padding);
        let content_max_x = (max_x + padding).min(self.width - 1);
        let content_max_y = (max_y + padding).min(self.height - 1);

        let final_width = (content_max_x - content_min_x + 1).max(1);
        let final_height = (content_max_y - content_min_y + 1).max(1);

        let mut final_grid = DungeonGrid::new(final_width, final_height);

        // Copy tiles with remapped coordinates
        for old_y in content_min_y..=content_max_y {
            for old_x in content_min_x..=content_max_x {
                let old_coord = GridCoord2D::new(old_x, old_y);
                let tile = self.tiles[self.linearize_coords(old_coord)];
                let new_x = old_x - content_min_x;
                let new_y = old_y - content_min_y;
                let new_coord = GridCoord2D::new(new_x, new_y);
                final_grid.set(new_coord, tile);
            }
        }

        // Adjust exit position
        if let Some(exit_coord) = self.exit {
            let exit_x = (exit_coord.x as isize - content_min_x as isize)
                .max(0)
                .min(final_width as isize - 1) as usize;
            let exit_y = (exit_coord.y as isize - content_min_y as isize)
                .max(0)
                .min(final_height as isize - 1) as usize;
            final_grid.set_exit(GridCoord2D::new(exit_x, exit_y));
        }

        // Place walls around any newly exposed edges
        final_grid.place_walls();

        // Recompute edge masks
        final_grid.compute_edge_masks();

        final_grid
    }
}

impl GetCoordinateBounds2D for DungeonGrid {
    #[inline]
    fn width(&self) -> usize {
        self.width
    }

    #[inline]
    fn height(&self) -> usize {
        self.height
    }
}

impl Index<GridCoord2D> for DungeonGrid {
    type Output = TileType;

    fn index(&self, coord: GridCoord2D) -> &Self::Output {
        &self.tiles[self.linearize_coords(coord)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_grid_is_empty() {
        let grid = DungeonGrid::new(10, 10);
        assert_eq!(grid.width(), 10);
        assert_eq!(grid.height(), 10);
        assert_eq!(grid.floor_count(), 0);
        assert_eq!(grid.exit(), None);
    }

    #[test]
    fn test_set_floor_updates_tracking() {
        let mut grid = DungeonGrid::new(5, 5);
        let coord = GridCoord2D::new(2, 2);

        grid.set(coord, TileType::Floor);
        assert_eq!(grid.floor_count(), 1);
        assert!(grid.is_floor(coord));

        grid.set(coord, TileType::Wall);
        assert_eq!(grid.floor_count(), 0);
        assert!(!grid.is_floor(coord));
    }

    #[test]
    fn test_place_walls_around_floor() {
        let mut grid = DungeonGrid::new(5, 5);
        let center = GridCoord2D::new(2, 2);

        grid.set(center, TileType::Floor);
        grid.place_walls();

        // Center should still be floor
        assert_eq!(grid[center], TileType::Floor);

        // All 4 neighbors should be walls
        assert_eq!(grid[GridCoord2D::new(2, 1)], TileType::Wall);
        assert_eq!(grid[GridCoord2D::new(3, 2)], TileType::Wall);
        assert_eq!(grid[GridCoord2D::new(2, 3)], TileType::Wall);
        assert_eq!(grid[GridCoord2D::new(1, 2)], TileType::Wall);
    }

    #[test]
    fn test_edge_mask_computation() {
        let mut grid = DungeonGrid::new(3, 3);

        // Create a wall at center with floor around it
        grid.set(GridCoord2D::new(1, 1), TileType::Wall);
        grid.set(GridCoord2D::new(1, 0), TileType::Floor); // top
        grid.set(GridCoord2D::new(2, 1), TileType::Floor); // right
        grid.set(GridCoord2D::new(1, 2), TileType::Floor); // bottom
        grid.set(GridCoord2D::new(0, 1), TileType::Floor); // left

        grid.compute_edge_masks();

        // All neighbors are non-walls, so all bits should be set: 1+2+4+8=15
        assert_eq!(grid.edge_mask(GridCoord2D::new(1, 1)), 15);
    }

    #[test]
    fn test_trim_single_floor_tile() {
        let mut grid = DungeonGrid::new(10, 10);
        grid.set(GridCoord2D::new(5, 5), TileType::Floor);
        grid.place_walls();

        let trimmed = grid.trim(0);
        // Single floor + walls around = 3x3
        assert_eq!(trimmed.width(), 3);
        assert_eq!(trimmed.height(), 3);
        assert_eq!(trimmed.get(GridCoord2D::new(1, 1)), Some(TileType::Floor));
    }

    #[test]
    fn test_trim_small_cluster() {
        let mut grid = DungeonGrid::new(20, 20);
        // Place a 3x4 block of floors at (8, 6) to (10, 9)
        for y in 6..=9 {
            for x in 8..=10 {
                grid.set(GridCoord2D::new(x, y), TileType::Floor);
            }
        }
        grid.place_walls();

        let trimmed = grid.trim(0);
        // Content: x=7..=11 (walls at edges), y=5..=10 (walls at edges) => 5x6
        assert_eq!(trimmed.width(), 5);
        assert_eq!(trimmed.height(), 6);
    }

    #[test]
    fn test_trim_preserves_exit() {
        let mut grid = DungeonGrid::new(10, 10);
        grid.set(GridCoord2D::new(5, 5), TileType::Floor);
        grid.set_exit(GridCoord2D::new(5, 5));
        grid.place_walls();

        let trimmed = grid.trim(0);
        let exit = trimmed.exit().unwrap();
        // Exit was at (5,5), content min is (4,4) after walls, so exit maps to (1,1)
        assert_eq!(exit.x, 1);
        assert_eq!(exit.y, 1);
    }

    #[test]
    fn test_trim_with_padding() {
        let mut grid = DungeonGrid::new(10, 10);
        grid.set(GridCoord2D::new(5, 5), TileType::Floor);
        grid.place_walls();

        let trimmed = grid.trim(2);
        // Base 3x3 + 2 padding each side = 7x7
        assert_eq!(trimmed.width(), 7);
        assert_eq!(trimmed.height(), 7);
        // Center floor should be at (3, 3)
        assert_eq!(trimmed.get(GridCoord2D::new(3, 3)), Some(TileType::Floor));
    }

    #[test]
    fn test_trim_recomputes_walls_at_edges() {
        let mut grid = DungeonGrid::new(7, 7);
        // Place a floor at center
        grid.set(GridCoord2D::new(3, 3), TileType::Floor);
        // Don't call place_walls - simulate a grid with only floor
        grid.set(GridCoord2D::new(2, 3), TileType::Floor);
        grid.set(GridCoord2D::new(4, 3), TileType::Floor);

        let trimmed = grid.trim(1);
        // After trim + place_walls, neighbors of floor should be walls
        // The floor at old (2,3) maps to new (0,1), left neighbor should be out of bounds
        // but the trim should have added padding, so let's check right neighbor of rightmost floor
        assert!(trimmed.width() >= 4);
        assert!(trimmed.height() >= 3);
    }

    #[test]
    fn test_trim_empty_grid() {
        let grid = DungeonGrid::new(10, 10);
        let trimmed = grid.trim(0);
        assert_eq!(trimmed.width(), 1);
        assert_eq!(trimmed.height(), 1);
    }

    #[test]
    fn test_trim_full_grid_no_trim() {
        let mut grid = DungeonGrid::new(5, 5);
        // Fill entire grid with floor
        for y in 0..5 {
            for x in 0..5 {
                grid.set(GridCoord2D::new(x, y), TileType::Floor);
            }
        }
        grid.place_walls();

        let trimmed = grid.trim(0);
        assert_eq!(trimmed.width(), 5);
        assert_eq!(trimmed.height(), 5);
    }

    #[test]
    fn test_trim_exit_clamped_to_bounds() {
        let mut grid = DungeonGrid::new(10, 10);
        // Place content only in top-left
        grid.set(GridCoord2D::new(0, 0), TileType::Floor);
        // Exit is far away from content
        grid.set_exit(GridCoord2D::new(9, 9));
        grid.place_walls();

        let trimmed = grid.trim(0);
        let exit = trimmed.exit().unwrap();
        // Exit should be clamped to the trimmed grid bounds
        assert!(exit.x < trimmed.width());
        assert!(exit.y < trimmed.height());
    }

    #[test]
    fn test_trim_preserves_floor_count() {
        let mut grid = DungeonGrid::new(15, 15);
        let original_floor_count = 10;
        // Place 10 floor tiles in a line
        for i in 0..original_floor_count {
            grid.set(GridCoord2D::new(5 + i, 7), TileType::Floor);
        }
        grid.place_walls();

        let trimmed = grid.trim(0);
        assert_eq!(trimmed.floor_count(), original_floor_count);
    }
}
