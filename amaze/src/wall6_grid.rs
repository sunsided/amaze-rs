use crate::direction6::Direction6;
use crate::hex_coord::HexCoord;
use std::collections::VecDeque;
use std::ops::{Index, IndexMut};

pub type Wall6 = Direction6;

#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Wall6Grid {
    width: usize,
    height: usize,
    walls: Vec<Wall6>,
}

impl Wall6Grid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            walls: vec![Wall6::ALL; width * height],
        }
    }

    pub fn get(&self, coord: HexCoord) -> Option<&Wall6> {
        if coord.q < 0
            || coord.q >= self.width as isize
            || coord.r < 0
            || coord.r >= self.height as isize
        {
            return None;
        }
        Some(&self[coord])
    }

    pub fn get_mut(&mut self, coord: HexCoord) -> Option<&mut Wall6> {
        if coord.q < 0
            || coord.q >= self.width as isize
            || coord.r < 0
            || coord.r >= self.height as isize
        {
            return None;
        }
        Some(&mut self[coord])
    }

    pub fn remove_wall_between(&mut self, a: HexCoord, b: HexCoord) {
        let dq = b.q - a.q;
        let dr = b.r - a.r;
        let dir = Direction6::from_delta(dq, dr).expect("Cells must be adjacent hex neighbors");
        let opposite = dir.opposite();

        let a_idx = self.linearize_coord(a);
        let b_idx = self.linearize_coord(b);

        self.walls[a_idx] -= dir;
        self.walls[b_idx] -= opposite;
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn coords(&self) -> impl Iterator<Item = HexCoord> + '_ {
        (0..self.height)
            .flat_map(move |r| (0..self.width).map(move |q| HexCoord::new(q as isize, r as isize)))
    }

    pub fn neighbors(&self, cell: HexCoord) -> impl Iterator<Item = HexCoord> + '_ {
        let mut out = [None, None, None, None, None, None];
        let mut n = 0;

        for &dir in &Direction6::CARDINALS {
            if let Some(neighbor) = cell.try_neighbor(dir, self.width, self.height) {
                out[n] = Some(neighbor);
                n += 1;
            }
        }

        out.into_iter().flatten()
    }

    pub fn open_neighbors(&self, cell: HexCoord) -> impl Iterator<Item = HexCoord> + '_ {
        let mut out = [None, None, None, None, None, None];
        let mut n = 0;

        if let Some(walls) = self.get(cell) {
            let dirs = *walls;
            for &dir in &Direction6::CARDINALS {
                if !dirs.contains(dir) {
                    if let Some(neighbor) = cell.try_neighbor(dir, self.width, self.height) {
                        out[n] = Some(neighbor);
                        n += 1;
                    }
                }
            }
        }

        out.into_iter().flatten()
    }

    pub fn stats(&self) -> crate::stats::MazeStats {
        crate::stats::MazeStats::from_grid_hex(self)
    }

    pub(crate) fn bfs_distances(&self, start: HexCoord) -> Vec<Option<usize>> {
        let mut dist = vec![None; self.width * self.height];
        if self.get(start).is_none() {
            return dist;
        }

        let mut q = VecDeque::new();
        dist[self.linearize_coord(start)] = Some(0);
        q.push_back(start);

        while let Some(cell) = q.pop_front() {
            let base = dist[self.linearize_coord(cell)].unwrap_or(0);
            for n in self.open_neighbors(cell) {
                let idx = self.linearize_coord(n);
                if dist[idx].is_none() {
                    dist[idx] = Some(base + 1);
                    q.push_back(n);
                }
            }
        }

        dist
    }

    #[inline]
    fn linearize_coord(&self, coord: HexCoord) -> usize {
        let q = coord.q as usize;
        let r = coord.r as usize;
        debug_assert!(q < self.width && r < self.height, "HexCoord out of bounds");
        r * self.width + q
    }
}

impl Index<HexCoord> for Wall6Grid {
    type Output = Wall6;

    fn index(&self, index: HexCoord) -> &Self::Output {
        let index = self.linearize_coord(index);
        &self.walls[index]
    }
}

impl IndexMut<HexCoord> for Wall6Grid {
    fn index_mut(&mut self, index: HexCoord) -> &mut Self::Output {
        let index = self.linearize_coord(index);
        &mut self.walls[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_grid_has_all_walls() {
        let grid = Wall6Grid::new(3, 3);
        for coord in grid.coords() {
            assert!(grid[coord].is_all());
        }
    }

    #[test]
    fn remove_wall_between_works() {
        let mut grid = Wall6Grid::new(3, 3);
        let a = HexCoord::new(1, 1);
        let b = HexCoord::new(2, 1); // east of a
        grid.remove_wall_between(a, b);
        assert!(!grid[a].contains(Direction6::EAST));
        assert!(!grid[b].contains(Direction6::WEST));
    }

    #[test]
    fn neighbors_stays_in_bounds() {
        let grid = Wall6Grid::new(3, 3);
        let corner = HexCoord::new(0, 0);
        let neighbors: Vec<_> = grid.neighbors(corner).collect();
        for n in &neighbors {
            assert!(n.q >= 0 && n.q < 3 && n.r >= 0 && n.r < 3);
        }
        assert_eq!(neighbors.len(), 2); // corner has 2 neighbors in bounded axial region
    }

    #[test]
    fn open_neighbors_empty_when_all_walls() {
        let grid = Wall6Grid::new(3, 3);
        let center = HexCoord::new(1, 1);
        let open: Vec<_> = grid.open_neighbors(center).collect();
        assert!(open.is_empty());
    }

    #[test]
    fn open_neighbors_returns_passages() {
        let mut grid = Wall6Grid::new(3, 3);
        let center = HexCoord::new(1, 1);
        let east = HexCoord::new(2, 1);
        grid.remove_wall_between(center, east);
        let open: Vec<_> = grid.open_neighbors(center).collect();
        assert_eq!(open.len(), 1);
        assert_eq!(open[0], east);
    }

    #[test]
    fn bfs_distances_from_center() {
        let mut grid = Wall6Grid::new(3, 3);
        let center = HexCoord::new(1, 1);
        let neighbors: Vec<_> = grid.neighbors(center).collect();
        for n in neighbors {
            grid.remove_wall_between(center, n);
        }
        let dists = grid.bfs_distances(center);
        let center_idx = (center.r as usize) * grid.width() + center.q as usize;
        assert_eq!(dists[center_idx], Some(0));
    }

    #[test]
    fn get_out_of_bounds_returns_none() {
        let grid = Wall6Grid::new(3, 3);
        assert!(grid.get(HexCoord::new(-1, 0)).is_none());
        assert!(grid.get(HexCoord::new(3, 0)).is_none());
        assert!(grid.get(HexCoord::new(0, -1)).is_none());
        assert!(grid.get(HexCoord::new(0, 3)).is_none());
    }
}
