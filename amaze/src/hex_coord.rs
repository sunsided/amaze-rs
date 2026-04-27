use crate::direction6::Direction6;
use std::ops::{Add, Sub};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HexCoord {
    pub q: isize,
    pub r: isize,
}

impl HexCoord {
    #[inline]
    pub fn new(q: isize, r: isize) -> Self {
        Self { q, r }
    }

    #[inline]
    pub fn offset(&self, dq: isize, dr: isize) -> Self {
        Self {
            q: self.q + dq,
            r: self.r + dr,
        }
    }

    #[inline]
    pub fn neighbor(&self, dir: Direction6) -> Self {
        match dir {
            Direction6::EAST => self.offset(1, 0),
            Direction6::WEST => self.offset(-1, 0),
            Direction6::NE => self.offset(1, -1),
            Direction6::NW => self.offset(0, -1),
            Direction6::SE => self.offset(0, 1),
            Direction6::SW => self.offset(-1, 1),
            _ => *self,
        }
    }

    #[inline]
    pub fn try_neighbor(&self, dir: Direction6, width: usize, height: usize) -> Option<Self> {
        let n = self.neighbor(dir);
        if n.q >= 0 && n.q < width as isize && n.r >= 0 && n.r < height as isize {
            Some(n)
        } else {
            None
        }
    }
}

impl Add<HexCoord> for HexCoord {
    type Output = Self;

    fn add(self, rhs: HexCoord) -> Self::Output {
        Self::new(self.q + rhs.q, self.r + rhs.r)
    }
}

impl Sub<HexCoord> for HexCoord {
    type Output = Self;

    fn sub(self, rhs: HexCoord) -> Self::Output {
        Self::new(self.q - rhs.q, self.r - rhs.r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_coord() {
        let c = HexCoord::new(3, 5);
        assert_eq!(c.q, 3);
        assert_eq!(c.r, 5);
    }

    #[test]
    fn offset_works() {
        let c = HexCoord::new(2, 3);
        let n = c.offset(1, -1);
        assert_eq!(n.q, 3);
        assert_eq!(n.r, 2);
    }

    #[test]
    fn neighbor_east() {
        let c = HexCoord::new(2, 3);
        let n = c.neighbor(Direction6::EAST);
        assert_eq!(n.q, 3);
        assert_eq!(n.r, 3);
    }

    #[test]
    fn neighbor_west() {
        let c = HexCoord::new(2, 3);
        let n = c.neighbor(Direction6::WEST);
        assert_eq!(n.q, 1);
        assert_eq!(n.r, 3);
    }

    #[test]
    fn neighbor_ne() {
        let c = HexCoord::new(2, 3);
        let n = c.neighbor(Direction6::NE);
        assert_eq!(n.q, 3);
        assert_eq!(n.r, 2);
    }

    #[test]
    fn neighbor_nw() {
        let c = HexCoord::new(2, 3);
        let n = c.neighbor(Direction6::NW);
        assert_eq!(n.q, 2);
        assert_eq!(n.r, 2);
    }

    #[test]
    fn neighbor_se() {
        let c = HexCoord::new(2, 3);
        let n = c.neighbor(Direction6::SE);
        assert_eq!(n.q, 2);
        assert_eq!(n.r, 4);
    }

    #[test]
    fn neighbor_sw() {
        let c = HexCoord::new(2, 3);
        let n = c.neighbor(Direction6::SW);
        assert_eq!(n.q, 1);
        assert_eq!(n.r, 4);
    }

    #[test]
    fn try_neighbor_in_bounds() {
        let c = HexCoord::new(2, 2);
        assert!(c.try_neighbor(Direction6::EAST, 5, 5).is_some());
        assert!(c.try_neighbor(Direction6::WEST, 5, 5).is_some());
    }

    #[test]
    fn try_neighbor_out_of_bounds() {
        let c = HexCoord::new(0, 0);
        assert!(c.try_neighbor(Direction6::WEST, 5, 5).is_none());
        assert!(c.try_neighbor(Direction6::NW, 5, 5).is_none());
    }

    #[test]
    fn add_works() {
        let a = HexCoord::new(1, 2);
        let b = HexCoord::new(3, 4);
        let c = a + b;
        assert_eq!(c.q, 4);
        assert_eq!(c.r, 6);
    }

    #[test]
    fn sub_works() {
        let a = HexCoord::new(5, 7);
        let b = HexCoord::new(2, 3);
        let c = a - b;
        assert_eq!(c.q, 3);
        assert_eq!(c.r, 4);
    }
}
