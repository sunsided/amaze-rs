use crate::direction4::Direction4;
use crate::grid_coord_2d::{GetCoordinateBounds2D, GridCoord2D, LinearizeCoords2D};
use crate::room4::Wall4;
use crate::room4_list::{Room4List, RoomIndex};
use crate::stats::MazeStats;
use std::collections::VecDeque;
use std::ops::{Index, IndexMut};

#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Wall4Grid {
    width: usize,
    height: usize,
    walls: Vec<Wall4>,
}

impl Wall4Grid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            walls: vec![Wall4::ALL; width * height],
        }
    }

    pub fn get(&self, coords: GridCoord2D) -> Option<&Wall4> {
        if coords.x >= self.width || coords.y >= self.height {
            return None;
        }
        Some(&self[coords])
    }

    pub fn get_mut(&mut self, coords: GridCoord2D) -> Option<&mut Wall4> {
        if coords.x >= self.width || coords.y >= self.height {
            return None;
        }
        Some(&mut self[coords])
    }

    pub fn remove_wall_between(&mut self, current: GridCoord2D, selected: GridCoord2D) {
        // Check adjacency
        let dx = current.x as isize - selected.x as isize;
        let dy = current.y as isize - selected.y as isize;
        assert!(
            (dx.abs() == 1 && dy == 0) || (dy.abs() == 1 && dx == 0),
            "Cells are not adjacent: {:?}, {:?}",
            current,
            selected
        );

        let cur = self.linearize_coords(current);
        let sel = self.linearize_coords(selected);

        if current.x > selected.x {
            self.walls[sel] -= Wall4::EAST;
            self.walls[cur] -= Wall4::WEST;
        } else if current.x < selected.x {
            self.walls[sel] -= Wall4::WEST;
            self.walls[cur] -= Wall4::EAST;
        }

        if current.y > selected.y {
            self.walls[sel] -= Wall4::SOUTH;
            self.walls[cur] -= Wall4::NORTH;
        } else if current.y < selected.y {
            self.walls[sel] -= Wall4::NORTH;
            self.walls[cur] -= Wall4::SOUTH;
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn coords(&self) -> impl Iterator<Item = GridCoord2D> + '_ {
        (0..self.height).flat_map(move |y| (0..self.width).map(move |x| GridCoord2D::new(x, y)))
    }

    pub fn neighbors(&self, cell: GridCoord2D) -> impl Iterator<Item = GridCoord2D> + '_ {
        let mut out = [None, None, None, None];
        let mut n = 0;
        if let Some(c) = cell.up().filter(|c| c.y < self.height) {
            out[n] = Some(c);
            n += 1;
        }
        if let Some(c) = cell.right().filter(|c| c.x < self.width) {
            out[n] = Some(c);
            n += 1;
        }
        if let Some(c) = cell.down().filter(|c| c.y < self.height) {
            out[n] = Some(c);
            n += 1;
        }
        if let Some(c) = cell.left().filter(|c| c.x < self.width) {
            out[n] = Some(c);
        }
        out.into_iter().flatten()
    }

    pub fn open_neighbors(&self, cell: GridCoord2D) -> Vec<GridCoord2D> {
        let Some(walls) = self.get(cell) else {
            return Vec::new();
        };

        let mut out = Vec::with_capacity(4);
        if !walls.contains(Direction4::NORTH) {
            if let Some(n) = cell.up().filter(|c| c.y < self.height) {
                out.push(n);
            }
        }
        if !walls.contains(Direction4::EAST) {
            if let Some(e) = cell.right().filter(|c| c.x < self.width) {
                out.push(e);
            }
        }
        if !walls.contains(Direction4::SOUTH) {
            if let Some(s) = cell.down().filter(|c| c.y < self.height) {
                out.push(s);
            }
        }
        if !walls.contains(Direction4::WEST) {
            if let Some(w) = cell.left().filter(|c| c.x < self.width) {
                out.push(w);
            }
        }
        out
    }

    pub fn to_room_list<Tag, F>(&self, tag_fn: F) -> Room4List<Tag>
    where
        F: Fn(GridCoord2D) -> Tag,
    {
        let mut list = Room4List::default();
        let mut index_of_cell =
            vec![RoomIndex::from(0).expect("0 is valid"); self.width * self.height];

        for cell in self.coords() {
            let index = list.push_default(tag_fn(cell));
            index_of_cell[self.linearize_coords(cell)] = index;
        }

        for cell in self.coords() {
            let idx = index_of_cell[self.linearize_coords(cell)];
            let doors = !self[cell];
            let room = list.get_mut(idx).expect("new room index must exist");
            if doors.contains(Direction4::NORTH) {
                if let Some(n) = cell.up().filter(|c| c.y < self.height) {
                    room.set_north(index_of_cell[self.linearize_coords(n)]);
                }
            }
            if doors.contains(Direction4::EAST) {
                if let Some(e) = cell.right().filter(|c| c.x < self.width) {
                    room.set_east(index_of_cell[self.linearize_coords(e)]);
                }
            }
            if doors.contains(Direction4::SOUTH) {
                if let Some(s) = cell.down().filter(|c| c.y < self.height) {
                    room.set_south(index_of_cell[self.linearize_coords(s)]);
                }
            }
            if doors.contains(Direction4::WEST) {
                if let Some(w) = cell.left().filter(|c| c.x < self.width) {
                    room.set_west(index_of_cell[self.linearize_coords(w)]);
                }
            }
        }

        list
    }

    pub fn stats(&self) -> MazeStats {
        MazeStats::from_grid(self)
    }

    pub(crate) fn bfs_distances(&self, start: GridCoord2D) -> Vec<Option<usize>> {
        let mut dist = vec![None; self.width * self.height];
        if self.get(start).is_none() {
            return dist;
        }

        let mut q = VecDeque::new();
        dist[self.linearize_coords(start)] = Some(0);
        q.push_back(start);

        while let Some(cell) = q.pop_front() {
            let base = dist[self.linearize_coords(cell)].unwrap_or(0);
            for n in self.open_neighbors(cell) {
                let idx = self.linearize_coords(n);
                if dist[idx].is_none() {
                    dist[idx] = Some(base + 1);
                    q.push_back(n);
                }
            }
        }

        dist
    }
}

impl From<&Wall4Grid> for Room4List<()> {
    fn from(value: &Wall4Grid) -> Self {
        value.to_room_list(|_| ())
    }
}

impl GetCoordinateBounds2D for Wall4Grid {
    #[inline]
    fn width(&self) -> usize {
        self.width()
    }

    #[inline]
    fn height(&self) -> usize {
        self.height()
    }
}

impl Index<GridCoord2D> for Wall4Grid {
    type Output = Wall4;

    fn index(&self, index: GridCoord2D) -> &Self::Output {
        let index = self.linearize_coords(index);
        &self.walls[index]
    }
}

impl IndexMut<GridCoord2D> for Wall4Grid {
    fn index_mut(&mut self, index: GridCoord2D) -> &mut Self::Output {
        let index = self.linearize_coords(index);
        &mut self.walls[index]
    }
}
