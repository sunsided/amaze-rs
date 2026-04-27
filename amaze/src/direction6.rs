use std::fmt::{Debug, Formatter};
use std::ops::{Add, AddAssign, Deref, Not, Sub, SubAssign};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Direction6(u8);

/// A 6-connected (hexagonal) neighborhood direction using axial coordinates.
impl Direction6 {
    /// The east direction.
    pub const EAST: Self = Self(0b000001);
    /// The west direction.
    pub const WEST: Self = Self(0b000010);
    /// The northeast direction.
    pub const NE: Self = Self(0b000100);
    /// The northwest direction.
    pub const NW: Self = Self(0b001000);
    /// The southeast direction.
    pub const SE: Self = Self(0b010000);
    /// The southwest direction.
    pub const SW: Self = Self(0b100000);
    /// All directions.
    pub const ALL: Self = Self(0b111111);
    /// No direction.
    pub const NONE: Self = Self(0b000000);
    /// A bit mask used for obtaining valid values from arbitrary inputs.
    const MASK: u8 = Self::ALL.0;

    /// Tests whether this direction value "contains" a specified set of directions.
    #[inline]
    pub fn contains(&self, other: Direction6) -> bool {
        self.0 & other.0 == other.0
    }

    /// Tests whether this direction encodes "all" directions.
    #[inline]
    pub fn is_all(&self) -> bool {
        self.0 == Self::ALL.0
    }

    /// Tests whether this direction encodes "no" directions.
    #[inline]
    pub fn is_none(&self) -> bool {
        self.0 == Self::NONE.0
    }

    /// Tests whether this direction encodes a trivial (single) direction.
    pub fn is_trivial(&self) -> bool {
        matches!(
            *self,
            Self::EAST | Self::WEST | Self::NE | Self::NW | Self::SE | Self::SW
        )
    }

    #[inline]
    pub fn opposite(self) -> Self {
        match self {
            Self::EAST => Self::WEST,
            Self::WEST => Self::EAST,
            Self::NE => Self::SW,
            Self::SW => Self::NE,
            Self::NW => Self::SE,
            Self::SE => Self::NW,
            _ => Self::NONE,
        }
    }

    #[inline]
    pub fn from_delta(dq: isize, dr: isize) -> Option<Self> {
        match (dq, dr) {
            (1, 0) => Some(Self::EAST),
            (-1, 0) => Some(Self::WEST),
            (1, -1) => Some(Self::NE),
            (0, -1) => Some(Self::NW),
            (0, 1) => Some(Self::SE),
            (-1, 1) => Some(Self::SW),
            _ => None,
        }
    }

    pub const CARDINALS: [Self; 6] = [
        Self::EAST,
        Self::WEST,
        Self::NE,
        Self::NW,
        Self::SE,
        Self::SW,
    ];

    /// Adds a new direction to this value.
    #[inline]
    pub fn include(&mut self, rhs: Self) -> &mut Self {
        *self += rhs;
        self
    }

    /// Removes a direction from this value.
    #[inline]
    pub fn remove(&mut self, rhs: Self) -> &mut Self {
        *self -= rhs;
        self
    }

    /// Joins two direction values
    #[inline]
    pub const fn join(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl Default for Direction6 {
    #[inline]
    fn default() -> Self {
        Self::NONE
    }
}

impl Add for Direction6 {
    type Output = Self;

    /// Joins two direction values.
    #[inline]
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl AddAssign for Direction6 {
    /// Joins two direction values while assigning.
    #[inline]
    #[allow(clippy::suspicious_op_assign_impl)]
    fn add_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl Sub for Direction6 {
    type Output = Self;

    /// Removes a direction value from a direction.
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self((self.0 & !rhs.0) & Direction6::MASK)
    }
}

impl SubAssign for Direction6 {
    /// Removes a direction value from a direction while assigning.
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = (self.0 & !rhs.0) & Direction6::MASK;
    }
}

impl Not for Direction6 {
    type Output = Direction6;

    /// Negates a direction value.
    fn not(self) -> Self::Output {
        Self((!self.0) & Self::MASK)
    }
}

impl Deref for Direction6 {
    type Target = u8;

    /// Gets the byte representation of a direction.
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Direction6Iterator {
    direction: Direction6,
    step: u8,
}

impl Iterator for Direction6Iterator {
    type Item = Direction6;

    fn next(&mut self) -> Option<Self::Item> {
        while self.step < 6 {
            let current = Direction6(1 << self.step);
            self.step += 1;
            if self.direction.contains(current) {
                return Some(current);
            }
        }

        None
    }
}

impl IntoIterator for Direction6 {
    type Item = Direction6;
    type IntoIter = Direction6Iterator;

    fn into_iter(self) -> Self::IntoIter {
        Direction6Iterator {
            direction: self,
            step: 0,
        }
    }
}

impl Debug for Direction6 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "0b{0:06b} ", self.0)?;

        match *self {
            Self::NONE => {
                write!(f, "(none)")?;
            }
            dir => {
                write!(f, "(")?;
                if dir.contains(Self::EAST) {
                    write!(f, "E")?;
                }
                if dir.contains(Self::WEST) {
                    write!(f, "W")?;
                }
                if dir.contains(Self::NE) {
                    write!(f, "NE")?;
                }
                if dir.contains(Self::NW) {
                    write!(f, "NW")?;
                }
                if dir.contains(Self::SE) {
                    write!(f, "SE")?;
                }
                if dir.contains(Self::SW) {
                    write!(f, "SW")?;
                }
                write!(f, ")")?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_none() {
        assert_eq!(Direction6::default(), Direction6::NONE);
        assert!(Direction6::default().is_none());
        assert_eq!(*Direction6::default(), 0);
    }

    #[test]
    fn all_contains_all_directions() {
        assert!(Direction6::ALL.contains(Direction6::EAST));
        assert!(Direction6::ALL.contains(Direction6::WEST));
        assert!(Direction6::ALL.contains(Direction6::NE));
        assert!(Direction6::ALL.contains(Direction6::NW));
        assert!(Direction6::ALL.contains(Direction6::SE));
        assert!(Direction6::ALL.contains(Direction6::SW));

        assert!(Direction6::ALL.contains(Direction6::NONE));
        assert!(Direction6::ALL.contains(Direction6::ALL));
        assert!(Direction6::ALL.is_all());

        assert_eq!(*Direction6::ALL, 0b111111);
    }

    #[test]
    fn any_direction_is_not_none() {
        assert!(!Direction6::EAST.is_none());
        assert!(!Direction6::WEST.is_none());
        assert!(!Direction6::NE.is_none());
        assert!(!Direction6::NW.is_none());
        assert!(!Direction6::SE.is_none());
        assert!(!Direction6::SW.is_none());
    }

    #[test]
    fn all_directions_is_all() {
        assert!(Direction6::ALL.is_all());
    }

    #[test]
    fn any_direction_is_not_all() {
        assert!(!Direction6::EAST.is_all());
        assert!(!Direction6::WEST.is_all());
        assert!(!Direction6::NE.is_all());
        assert!(!Direction6::NW.is_all());
        assert!(!Direction6::SE.is_all());
        assert!(!Direction6::SW.is_all());
        assert!(!Direction6::NONE.is_all());
    }

    #[test]
    fn add_combines_directions() {
        let direction = Direction6::EAST + Direction6::WEST;
        assert!(direction.contains(Direction6::EAST));
        assert!(direction.contains(Direction6::WEST));
        assert!(!direction.contains(Direction6::NE));
    }

    #[test]
    fn sub_removes_directions() {
        let direction = Direction6::ALL - Direction6::NE - Direction6::NW;
        assert!(direction.contains(Direction6::EAST));
        assert!(direction.contains(Direction6::WEST));
        assert!(!direction.contains(Direction6::NE));
        assert!(!direction.contains(Direction6::NW));
    }

    #[test]
    fn include_combines_directions() {
        let mut direction = Direction6::NONE;
        direction
            .include(Direction6::EAST)
            .include(Direction6::WEST);
        assert_eq!(direction, Direction6::EAST + Direction6::WEST);
    }

    #[test]
    fn add_assign_combines_directions() {
        let mut direction = Direction6::NONE;
        direction += Direction6::EAST;
        direction += Direction6::WEST;
        assert!(direction.contains(Direction6::EAST));
        assert!(direction.contains(Direction6::WEST));
        assert!(!direction.contains(Direction6::NE));
    }

    #[test]
    fn sub_assign_removes_directions() {
        let mut direction = Direction6::ALL;
        direction -= Direction6::NE;
        direction -= Direction6::NW;
        assert!(direction.contains(Direction6::EAST));
        assert!(direction.contains(Direction6::WEST));
        assert!(!direction.contains(Direction6::NE));
        assert!(!direction.contains(Direction6::NW));
    }

    #[test]
    fn remove_removes_directions() {
        let mut direction = Direction6::ALL;
        direction.remove(Direction6::NE).remove(Direction6::NW);
        assert_eq!(
            direction,
            Direction6::EAST + Direction6::WEST + Direction6::SE + Direction6::SW
        );
    }

    #[test]
    fn into_iter_for_all_enumerates_all() {
        let dirs: Vec<_> = Direction6::ALL.into_iter().collect();
        assert!(dirs.contains(&Direction6::EAST));
        assert!(dirs.contains(&Direction6::WEST));
        assert!(dirs.contains(&Direction6::NE));
        assert!(dirs.contains(&Direction6::NW));
        assert!(dirs.contains(&Direction6::SE));
        assert!(dirs.contains(&Direction6::SW));
    }

    #[test]
    fn into_iter_for_combination_enumerates_contained() {
        let dirs: Vec<_> = (Direction6::EAST + Direction6::SW).into_iter().collect();
        assert!(dirs.contains(&Direction6::EAST));
        assert!(dirs.contains(&Direction6::SW));
        assert!(!dirs.contains(&Direction6::WEST));
        assert!(!dirs.contains(&Direction6::NE));
    }

    #[test]
    fn debug_lists_directions() {
        assert_eq!(format!("{:?}", Direction6::NONE), "0b000000 (none)");
        assert_eq!(format!("{:?}", Direction6::ALL), "0b111111 (EWNENWSESW)");
        assert_eq!(format!("{:?}", Direction6::EAST), "0b000001 (E)");
        assert_eq!(format!("{:?}", Direction6::WEST), "0b000010 (W)");
        assert_eq!(format!("{:?}", Direction6::NE), "0b000100 (NE)");
        assert_eq!(format!("{:?}", Direction6::NW), "0b001000 (NW)");
        assert_eq!(format!("{:?}", Direction6::SE), "0b010000 (SE)");
        assert_eq!(format!("{:?}", Direction6::SW), "0b100000 (SW)");
    }

    #[test]
    fn not_all_is_none() {
        assert_eq!(!Direction6::ALL, Direction6::NONE);
    }

    #[test]
    fn not_inverts_selection() {
        assert_eq!(
            !(Direction6::EAST + Direction6::WEST),
            Direction6::NE + Direction6::NW + Direction6::SE + Direction6::SW
        );
    }

    #[test]
    fn is_trivial_works() {
        assert!(Direction6::EAST.is_trivial());
        assert!(Direction6::WEST.is_trivial());
        assert!(Direction6::NE.is_trivial());
        assert!(Direction6::NW.is_trivial());
        assert!(Direction6::SE.is_trivial());
        assert!(Direction6::SW.is_trivial());

        assert!(!Direction6::NONE.is_trivial());
        assert!(!Direction6::ALL.is_trivial());
        assert!(!(Direction6::EAST + Direction6::NE).is_trivial());
    }

    #[test]
    fn opposite_works() {
        assert_eq!(Direction6::EAST.opposite(), Direction6::WEST);
        assert_eq!(Direction6::WEST.opposite(), Direction6::EAST);
        assert_eq!(Direction6::NE.opposite(), Direction6::SW);
        assert_eq!(Direction6::SW.opposite(), Direction6::NE);
        assert_eq!(Direction6::NW.opposite(), Direction6::SE);
        assert_eq!(Direction6::SE.opposite(), Direction6::NW);
        assert_eq!(Direction6::NONE.opposite(), Direction6::NONE);
        assert_eq!(Direction6::ALL.opposite(), Direction6::NONE);
    }

    #[test]
    fn from_delta_works() {
        assert_eq!(Direction6::from_delta(1, 0), Some(Direction6::EAST));
        assert_eq!(Direction6::from_delta(-1, 0), Some(Direction6::WEST));
        assert_eq!(Direction6::from_delta(1, -1), Some(Direction6::NE));
        assert_eq!(Direction6::from_delta(0, -1), Some(Direction6::NW));
        assert_eq!(Direction6::from_delta(0, 1), Some(Direction6::SE));
        assert_eq!(Direction6::from_delta(-1, 1), Some(Direction6::SW));
        assert_eq!(Direction6::from_delta(0, 0), None);
        assert_eq!(Direction6::from_delta(2, 0), None);
    }

    #[test]
    fn join_works() {
        let direction = Direction6::EAST.join(Direction6::WEST).join(Direction6::NE);
        assert!(direction.contains(Direction6::EAST));
        assert!(direction.contains(Direction6::WEST));
        assert!(direction.contains(Direction6::NE));
        assert!(!direction.contains(Direction6::NW));
    }
}
