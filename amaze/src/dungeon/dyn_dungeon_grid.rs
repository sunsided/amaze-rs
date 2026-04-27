use crate::dungeon::{DungeonGrid, TileType};
use crate::grid_coord_2d::{GetCoordinateBounds2D, GridCoord2D};

const EXPAND_CHUNK: isize = 16;

pub struct DynDungeonGrid {
    inner: DungeonGrid,
    origin_x: isize,
    origin_y: isize,
    min_floor_x: isize,
    min_floor_y: isize,
    max_floor_x: isize,
    max_floor_y: isize,
    exit_world: Option<(isize, isize)>,
}

impl DynDungeonGrid {
    pub fn new(initial_width: usize, initial_height: usize) -> Self {
        let half_w = (initial_width as isize) / 2;
        let half_h = (initial_height as isize) / 2;

        Self {
            inner: DungeonGrid::new(initial_width, initial_height),
            origin_x: -half_w,
            origin_y: -half_h,
            min_floor_x: isize::MAX,
            min_floor_y: isize::MAX,
            max_floor_x: isize::MIN,
            max_floor_y: isize::MIN,
            exit_world: None,
        }
    }

    pub fn world_to_grid(&self, wx: isize, wy: isize) -> Option<GridCoord2D> {
        let gx = wx.checked_sub(self.origin_x)?;
        let gy = wy.checked_sub(self.origin_y)?;
        if gx >= 0
            && gx < self.inner.width() as isize
            && gy >= 0
            && gy < self.inner.height() as isize
        {
            Some(GridCoord2D::new(gx as usize, gy as usize))
        } else {
            None
        }
    }

    pub fn grid_to_world(&self, coord: GridCoord2D) -> (isize, isize) {
        (
            coord.x as isize + self.origin_x,
            coord.y as isize + self.origin_y,
        )
    }

    pub fn update_floor_bounds(&mut self, wx: isize, wy: isize) {
        if wx < self.min_floor_x {
            self.min_floor_x = wx;
        }
        if wx > self.max_floor_x {
            self.max_floor_x = wx;
        }
        if wy < self.min_floor_y {
            self.min_floor_y = wy;
        }
        if wy > self.max_floor_y {
            self.max_floor_y = wy;
        }
    }

    pub fn ensure_bounds(&mut self, min_x: isize, min_y: isize, max_x: isize, max_y: isize) {
        let needs_left = min_x < self.origin_x;
        let needs_right = max_x >= self.origin_x + self.inner.width() as isize;
        let needs_top = min_y < self.origin_y;
        let needs_bottom = max_y >= self.origin_y + self.inner.height() as isize;

        if !needs_left && !needs_right && !needs_top && !needs_bottom {
            return;
        }

        let current_width = self.inner.width() as isize;
        let current_height = self.inner.height() as isize;

        let expand_left = if needs_left {
            ((self.origin_x - min_x + EXPAND_CHUNK - 1) / EXPAND_CHUNK) * EXPAND_CHUNK
        } else {
            0
        };

        let expand_right = if needs_right {
            ((max_x - (self.origin_x + current_width) + EXPAND_CHUNK) / EXPAND_CHUNK) * EXPAND_CHUNK
        } else {
            0
        };

        let expand_top = if needs_top {
            ((self.origin_y - min_y + EXPAND_CHUNK - 1) / EXPAND_CHUNK) * EXPAND_CHUNK
        } else {
            0
        };

        let expand_bottom = if needs_bottom {
            ((max_y - (self.origin_y + current_height) + EXPAND_CHUNK) / EXPAND_CHUNK)
                * EXPAND_CHUNK
        } else {
            0
        };

        let new_width = (current_width + expand_left + expand_right) as usize;
        let new_height = (current_height + expand_top + expand_bottom) as usize;

        let mut new_grid = DungeonGrid::new(new_width, new_height);

        // Copy existing tiles
        for old_y in 0..current_height {
            for old_x in 0..current_width {
                let old_coord = GridCoord2D::new(old_x as usize, old_y as usize);
                if let Some(tile) = self.inner.get(old_coord) {
                    let new_x = (old_x + expand_left) as usize;
                    let new_y = (old_y + expand_top) as usize;
                    let new_coord = GridCoord2D::new(new_x, new_y);
                    new_grid.set(new_coord, tile);
                }
            }
        }

        // Update exit position in grid coordinates
        if let Some(exit_coord) = self.inner.exit() {
            let new_exit_x = (exit_coord.x as isize + expand_left) as usize;
            let new_exit_y = (exit_coord.y as isize + expand_top) as usize;
            new_grid.set_exit(GridCoord2D::new(new_exit_x, new_exit_y));
        }

        self.origin_x -= expand_left;
        self.origin_y -= expand_top;
        self.inner = new_grid;
    }

    pub fn set_world(&mut self, wx: isize, wy: isize, tile: TileType) {
        if let Some(coord) = self.world_to_grid(wx, wy) {
            let was_floor = self.inner.is_floor(coord);
            self.inner.set(coord, tile);

            if tile.is_passable() && !was_floor {
                self.update_floor_bounds(wx, wy);
            }
        }
    }

    pub fn get_world(&self, wx: isize, wy: isize) -> Option<TileType> {
        self.world_to_grid(wx, wy)
            .and_then(|coord| self.inner.get(coord))
    }

    pub fn is_floor_world(&self, wx: isize, wy: isize) -> bool {
        self.world_to_grid(wx, wy)
            .is_some_and(|coord| self.inner.is_floor(coord))
    }

    pub fn set_exit_world(&mut self, wx: isize, wy: isize) {
        self.exit_world = Some((wx, wy));
        if let Some(coord) = self.world_to_grid(wx, wy) {
            self.inner.set_exit(coord);
        }
    }

    pub fn finalize(self, padding: usize) -> DungeonGrid {
        let padding = padding as isize;

        let content_min_x = if self.min_floor_x == isize::MAX {
            0
        } else {
            self.min_floor_x - padding
        };
        let content_min_y = if self.min_floor_y == isize::MAX {
            0
        } else {
            self.min_floor_y - padding
        };
        let content_max_x = if self.max_floor_x == isize::MIN {
            0
        } else {
            self.max_floor_x + padding
        };
        let content_max_y = if self.max_floor_y == isize::MIN {
            0
        } else {
            self.max_floor_y + padding
        };

        let clamped_min_x = content_min_x.max(self.origin_x);
        let clamped_min_y = content_min_y.max(self.origin_y);
        let clamped_max_x = content_max_x.min(self.origin_x + self.inner.width() as isize - 1);
        let clamped_max_y = content_max_y.min(self.origin_y + self.inner.height() as isize - 1);

        let final_width = (clamped_max_x - clamped_min_x + 1).max(1) as usize;
        let final_height = (clamped_max_y - clamped_min_y + 1).max(1) as usize;

        let mut final_grid = DungeonGrid::new(final_width, final_height);

        for old_y in 0..self.inner.height() as isize {
            for old_x in 0..self.inner.width() as isize {
                let world_x = old_x + self.origin_x;
                let world_y = old_y + self.origin_y;

                if world_x < clamped_min_x
                    || world_x > clamped_max_x
                    || world_y < clamped_min_y
                    || world_y > clamped_max_y
                {
                    continue;
                }

                let old_coord = GridCoord2D::new(old_x as usize, old_y as usize);
                let new_x = (world_x - clamped_min_x) as usize;
                let new_y = (world_y - clamped_min_y) as usize;
                let new_coord = GridCoord2D::new(new_x, new_y);

                if let Some(tile) = self.inner.get(old_coord) {
                    final_grid.set(new_coord, tile);
                }
            }
        }

        // Adjust exit position
        if let Some((exit_wx, exit_wy)) = self.exit_world {
            let exit_x = (exit_wx - clamped_min_x)
                .max(0)
                .min(final_width as isize - 1) as usize;
            let exit_y = (exit_wy - clamped_min_y)
                .max(0)
                .min(final_height as isize - 1) as usize;
            final_grid.set_exit(GridCoord2D::new(exit_x, exit_y));
        } else if let Some(exit_coord) = self.inner.exit() {
            let exit_wx = exit_coord.x as isize + self.origin_x;
            let exit_wy = exit_coord.y as isize + self.origin_y;
            let exit_x = (exit_wx - clamped_min_x)
                .max(0)
                .min(final_width as isize - 1) as usize;
            let exit_y = (exit_wy - clamped_min_y)
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

    pub fn inner(&self) -> &DungeonGrid {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut DungeonGrid {
        &mut self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_grid_origin_at_center() {
        let grid = DynDungeonGrid::new(32, 32);
        assert_eq!(grid.origin_x, -16);
        assert_eq!(grid.origin_y, -16);
        assert_eq!(grid.inner.width(), 32);
        assert_eq!(grid.inner.height(), 32);
    }

    #[test]
    fn test_world_to_grid_conversion() {
        let grid = DynDungeonGrid::new(32, 32);

        // World (0, 0) should map to grid (16, 16)
        let coord = grid.world_to_grid(0, 0).unwrap();
        assert_eq!(coord.x, 16);
        assert_eq!(coord.y, 16);

        // World (-16, -16) should map to grid (0, 0)
        let coord = grid.world_to_grid(-16, -16).unwrap();
        assert_eq!(coord.x, 0);
        assert_eq!(coord.y, 0);

        // World (15, 15) should map to grid (31, 31)
        let coord = grid.world_to_grid(15, 15).unwrap();
        assert_eq!(coord.x, 31);
        assert_eq!(coord.y, 31);
    }

    #[test]
    fn test_grid_to_world_conversion() {
        let grid = DynDungeonGrid::new(32, 32);

        let (wx, wy) = grid.grid_to_world(GridCoord2D::new(16, 16));
        assert_eq!(wx, 0);
        assert_eq!(wy, 0);

        let (wx, wy) = grid.grid_to_world(GridCoord2D::new(0, 0));
        assert_eq!(wx, -16);
        assert_eq!(wy, -16);
    }

    #[test]
    fn test_set_world_and_get_world() {
        let mut grid = DynDungeonGrid::new(32, 32);
        grid.set_world(0, 0, TileType::Floor);
        grid.update_floor_bounds(0, 0);

        assert_eq!(grid.get_world(0, 0), Some(TileType::Floor));
        assert!(grid.is_floor_world(0, 0));
        assert!(!grid.is_floor_world(1, 0));
    }

    #[test]
    fn test_expand_right() {
        let mut grid = DynDungeonGrid::new(32, 32);
        // Initial right boundary is at world x = 15
        assert_eq!(grid.origin_x + grid.inner.width() as isize, 16);

        // Request a tile beyond the right boundary
        grid.ensure_bounds(0, 0, 20, 0);
        assert!(grid.inner.width() >= 37); // 32 + at least 5 to reach 20
        assert_eq!(grid.get_world(20, 0), Some(TileType::Empty));
    }

    #[test]
    fn test_expand_left() {
        let mut grid = DynDungeonGrid::new(32, 32);
        // Initial left boundary is at world x = -16

        // Request a tile beyond the left boundary
        grid.ensure_bounds(-20, 0, 0, 0);
        assert!(grid.inner.width() >= 36); // 32 + at least 4 to reach -20
        assert_eq!(grid.get_world(-20, 0), Some(TileType::Empty));
    }

    #[test]
    fn test_expand_top() {
        let mut grid = DynDungeonGrid::new(32, 32);

        grid.ensure_bounds(0, -20, 0, 0);
        assert!(grid.inner.height() >= 36);
        assert_eq!(grid.get_world(0, -20), Some(TileType::Empty));
    }

    #[test]
    fn test_expand_bottom() {
        let mut grid = DynDungeonGrid::new(32, 32);

        grid.ensure_bounds(0, 0, 0, 20);
        assert!(grid.inner.height() >= 37);
        assert_eq!(grid.get_world(0, 20), Some(TileType::Empty));
    }

    #[test]
    fn test_expand_all_directions() {
        let mut grid = DynDungeonGrid::new(32, 32);

        grid.ensure_bounds(-20, -20, 20, 20);
        assert!(grid.inner.width() >= 41);
        assert!(grid.inner.height() >= 41);
        assert_eq!(grid.get_world(-20, -20), Some(TileType::Empty));
        assert_eq!(grid.get_world(20, 20), Some(TileType::Empty));
    }

    #[test]
    fn test_finalize_trims_correctly() {
        let mut grid = DynDungeonGrid::new(32, 32);

        // Place floor tiles in a small region
        for x in -5..=5 {
            for y in -3..=3 {
                grid.ensure_bounds(x, y, x, y);
                grid.set_world(x, y, TileType::Floor);
            }
        }

        let final_grid = grid.finalize(0);
        assert_eq!(final_grid.width(), 11);
        assert_eq!(final_grid.height(), 7);
    }

    #[test]
    fn test_finalize_preserves_exit() {
        let mut grid = DynDungeonGrid::new(32, 32);

        for x in -5..=5 {
            grid.ensure_bounds(x, 0, x, 0);
            grid.set_world(x, 0, TileType::Floor);
        }
        grid.set_exit_world(5, 0);

        let final_grid = grid.finalize(0);
        let exit = final_grid.exit().unwrap();

        // Exit should be at (10, 3) in the trimmed grid
        // The content goes from x=-5 to x=5, so width=11, and exit at world x=5 maps to grid x=10
        assert_eq!(exit.x, 10);
    }

    #[test]
    fn test_finalize_with_padding() {
        let mut grid = DynDungeonGrid::new(32, 32);

        for x in -3..=3 {
            for y in -2..=2 {
                grid.ensure_bounds(x, y, x, y);
                grid.set_world(x, y, TileType::Floor);
            }
        }

        let final_grid = grid.finalize(2);
        assert_eq!(final_grid.width(), 11); // 7 + 2*2 = 11
        assert_eq!(final_grid.height(), 9); // 5 + 2*2 = 9
    }

    #[test]
    fn test_finalize_recomputes_walls() {
        let mut grid = DynDungeonGrid::new(32, 32);

        // Place a single floor tile at origin
        grid.ensure_bounds(0, 0, 0, 0);
        grid.set_world(0, 0, TileType::Floor);

        // Finalize with padding to have room for walls
        let final_grid = grid.finalize(1);

        // Grid should be 3x3 (from -1 to 1 with padding)
        assert_eq!(final_grid.width(), 3);
        assert_eq!(final_grid.height(), 3);

        // Center should be floor
        let center = GridCoord2D::new(1, 1);
        assert_eq!(final_grid.get(center), Some(TileType::Floor));

        // All 4 neighbors should be walls
        assert_eq!(final_grid.get(GridCoord2D::new(0, 1)), Some(TileType::Wall));
        assert_eq!(final_grid.get(GridCoord2D::new(2, 1)), Some(TileType::Wall));
        assert_eq!(final_grid.get(GridCoord2D::new(1, 0)), Some(TileType::Wall));
        assert_eq!(final_grid.get(GridCoord2D::new(1, 2)), Some(TileType::Wall));
    }

    #[test]
    fn test_expand_preserves_existing_tiles() {
        let mut grid = DynDungeonGrid::new(32, 32);

        // Place floor at center
        grid.set_world(0, 0, TileType::Floor);
        grid.update_floor_bounds(0, 0);

        // Expand in all directions
        grid.ensure_bounds(-20, -20, 20, 20);

        // Original floor should still be there
        assert_eq!(grid.get_world(0, 0), Some(TileType::Floor));
        assert!(grid.is_floor_world(0, 0));
    }

    #[test]
    fn test_chunk_expansion_amortization() {
        let mut grid = DynDungeonGrid::new(32, 32);

        // Request just beyond current boundary
        let initial_width = grid.inner.width();
        grid.ensure_bounds(0, 0, 16, 0); // Right at boundary + 1

        // Should have expanded by EXPAND_CHUNK (16), not by 1
        assert_eq!(grid.inner.width(), initial_width + EXPAND_CHUNK as usize);
    }
}
