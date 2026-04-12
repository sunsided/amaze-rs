use crate::grid_coord_2d::{GetCoordinateBounds2D, GridCoord2D, LinearizeCoords2D};
use crate::wall4_grid::Wall4Grid;
use std::ops::Index;

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PassabilityGrid {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<bool>,
    entrance: (usize, usize),
    exit: (usize, usize),
}

impl PassabilityGrid {
    pub fn is_passable(&self, x: usize, y: usize) -> bool {
        self.get(x, y).unwrap_or(false)
    }

    pub fn get(&self, x: usize, y: usize) -> Option<bool> {
        if x >= self.width || y >= self.height {
            return None;
        }

        Some(self.cells[self.linearize_coords(GridCoord2D::new(x, y))])
    }

    pub fn entrance_position(&self) -> (usize, usize) {
        self.entrance
    }

    pub fn exit_position(&self) -> (usize, usize) {
        self.exit
    }

    pub fn set_entrance(&mut self, x: usize, y: usize) -> bool {
        if self.is_passable(x, y) {
            self.entrance = (x, y);
            true
        } else {
            false
        }
    }

    pub fn set_exit(&mut self, x: usize, y: usize) -> bool {
        if self.is_passable(x, y) {
            self.exit = (x, y);
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn maze_to_passability(coord: GridCoord2D) -> (usize, usize) {
        (coord.x * 2 + 1, coord.y * 2 + 1)
    }

    pub fn passability_to_maze(x: usize, y: usize) -> Option<GridCoord2D> {
        if x % 2 == 1 && y % 2 == 1 {
            Some(GridCoord2D::new(x / 2, y / 2))
        } else {
            None
        }
    }

    fn set_cell(&mut self, x: usize, y: usize, is_passable: bool) {
        let index = self.linearize_coords(GridCoord2D::new(x, y));
        self.cells[index] = is_passable;
    }
}

impl GetCoordinateBounds2D for PassabilityGrid {
    #[inline]
    fn width(&self) -> usize {
        self.width
    }

    #[inline]
    fn height(&self) -> usize {
        self.height
    }
}

impl Index<(usize, usize)> for PassabilityGrid {
    type Output = bool;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let idx = self.linearize_coords(GridCoord2D::new(index.0, index.1));
        &self.cells[idx]
    }
}

impl From<&Wall4Grid> for PassabilityGrid {
    fn from(value: &Wall4Grid) -> Self {
        let width = value.width() * 2 + 1;
        let height = value.height() * 2 + 1;

        let mut grid = Self {
            width,
            height,
            cells: vec![false; width * height],
            entrance: if value.width() > 0 && value.height() > 0 {
                (1, 1)
            } else {
                (0, 0)
            },
            exit: if value.width() > 0 && value.height() > 0 {
                (width - 2, height - 2)
            } else {
                (0, 0)
            },
        };

        for cell in value.coords() {
            let (px, py) = Self::maze_to_passability(cell);
            grid.set_cell(px, py, true);

            for neighbor in value.open_neighbors(cell) {
                if neighbor.x > cell.x {
                    grid.set_cell(px + 1, py, true);
                } else if neighbor.x < cell.x {
                    grid.set_cell(px - 1, py, true);
                } else if neighbor.y > cell.y {
                    grid.set_cell(px, py + 1, true);
                } else {
                    grid.set_cell(px, py - 1, true);
                }
            }
        }

        grid
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::direction4::Direction4;
    use crate::generators::RecursiveBacktracker4;

    #[test]
    fn passability_uses_inflated_dimensions() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let passability = PassabilityGrid::from(&maze);

        assert_eq!(passability.width, 17);
        assert_eq!(passability.height, 17);
    }

    #[test]
    fn room_cells_are_passable() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let passability = PassabilityGrid::from(&maze);

        for cell in maze.coords() {
            let (x, y) = PassabilityGrid::maze_to_passability(cell);
            assert!(passability.is_passable(x, y));
            assert_eq!(PassabilityGrid::passability_to_maze(x, y), Some(cell));
        }
    }

    #[test]
    fn open_walls_map_to_passable_connections() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let passability = PassabilityGrid::from(&maze);

        for cell in maze.coords() {
            let (x, y) = PassabilityGrid::maze_to_passability(cell);
            let walls = maze[cell];

            if !walls.contains(Direction4::NORTH) {
                assert!(passability.is_passable(x, y - 1));
            }
            if !walls.contains(Direction4::EAST) {
                assert!(passability.is_passable(x + 1, y));
            }
            if !walls.contains(Direction4::SOUTH) {
                assert!(passability.is_passable(x, y + 1));
            }
            if !walls.contains(Direction4::WEST) {
                assert!(passability.is_passable(x - 1, y));
            }
        }
    }

    #[test]
    fn entrance_and_exit_defaults_and_updates() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let mut passability = PassabilityGrid::from(&maze);

        assert_eq!(passability.entrance_position(), (1, 1));
        assert_eq!(passability.exit_position(), (15, 15));

        assert!(passability.set_entrance(1, 1));
        assert!(passability.set_exit(15, 15));
        assert!(!passability.set_entrance(0, 0));
        assert!(!passability.set_exit(16, 16));
    }

    #[test]
    fn boundary_cells_are_walls() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let passability = PassabilityGrid::from(&maze);

        for x in 0..passability.width {
            assert!(!passability.is_passable(x, 0));
            assert!(!passability.is_passable(x, passability.height - 1));
        }
        for y in 0..passability.height {
            assert!(!passability.is_passable(0, y));
            assert!(!passability.is_passable(passability.width - 1, y));
        }
    }

    #[test]
    fn closed_walls_map_to_impassable() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let passability = PassabilityGrid::from(&maze);

        for cell in maze.coords() {
            let (x, y) = PassabilityGrid::maze_to_passability(cell);
            let walls = maze[cell];

            if walls.contains(Direction4::NORTH) {
                assert!(!passability.is_passable(x, y - 1));
            }
            if walls.contains(Direction4::EAST) {
                assert!(!passability.is_passable(x + 1, y));
            }
            if walls.contains(Direction4::SOUTH) {
                assert!(!passability.is_passable(x, y + 1));
            }
            if walls.contains(Direction4::WEST) {
                assert!(!passability.is_passable(x - 1, y));
            }
        }
    }

    #[test]
    fn get_returns_none_out_of_bounds() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let passability = PassabilityGrid::from(&maze);

        assert!(passability.get(passability.width, 0).is_none());
        assert!(passability.get(0, passability.height).is_none());
    }

    #[test]
    fn index_trait_matches_get() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let passability = PassabilityGrid::from(&maze);

        for y in 0..passability.height {
            for x in 0..passability.width {
                assert_eq!(passability[(x, y)], passability.get(x, y).unwrap());
            }
        }
    }

    #[test]
    fn coordinate_round_trip() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);

        for cell in maze.coords() {
            let (x, y) = PassabilityGrid::maze_to_passability(cell);
            assert_eq!(PassabilityGrid::passability_to_maze(x, y), Some(cell));
        }
    }

    #[test]
    fn passability_to_maze_returns_none_for_walls() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let passability = PassabilityGrid::from(&maze);

        for y in 0..passability.height {
            for x in 0..passability.width {
                if x % 2 == 0 || y % 2 == 0 {
                    assert_eq!(PassabilityGrid::passability_to_maze(x, y), None);
                }
            }
        }
    }
}
