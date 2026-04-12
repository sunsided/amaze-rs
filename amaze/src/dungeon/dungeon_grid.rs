use crate::dungeon::TileType;
use crate::grid_coord_2d::{GetCoordinateBounds2D, GridCoord2D, LinearizeCoords2D};
use std::collections::HashSet;
use std::ops::{Index, IndexMut};

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
        let size = width * height;
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

impl IndexMut<GridCoord2D> for DungeonGrid {
    fn index_mut(&mut self, coord: GridCoord2D) -> &mut Self::Output {
        let idx = self.linearize_coords(coord);
        &mut self.tiles[idx]
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
}
