use std::fmt::{Debug, Formatter};
use std::ops::{Add, AddAssign, Deref, Not, Sub, SubAssign};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Direction4(u8);

/// A 4-connected (Von Neumann) neighborhood direction.
impl Direction4 {
    /// The north/up direction.
    pub const NORTH: Self = Self(0b0001);
    /// The south/down direction.
    pub const SOUTH: Self = Self(0b0010);
    /// The east/right direction.
    pub const EAST: Self = Self(0b0100);
    /// The west/left direction.
    pub const WEST: Self = Self(0b1000);
    /// All directions.
    pub const ALL: Self = Self(0b1111);
    /// No direction.
    pub const NONE: Self = Self(0b0000);
    /// A bit mask used for obtaining valid values from arbitrary inputs.
    const MASK: u8 = Self::ALL.0;

    /// Tests whether this direction value "contains" a specified set of directions.
    ///
    /// ## Example
    /// ```
    /// use amaze::direction4::Direction4;
    /// assert!(Direction4::ALL.contains(Direction4::NORTH + Direction4::SOUTH));
    /// assert!(Direction4::NORTH.contains(Direction4::NORTH));
    /// assert!(!Direction4::NORTH.contains(Direction4::WEST));
    /// assert!(!Direction4::NONE.contains(Direction4::WEST));
    /// ```
    #[inline]
    pub fn contains(&self, other: Direction4) -> bool {
        &self.0 & other.0 == other.0
    }

    /// Tests whether this direction encodes "all" directions.
    ///
    /// ## Example
    /// ```
    /// use amaze::direction4::Direction4;
    /// assert!(Direction4::ALL.is_all());
    /// assert!(!Direction4::NORTH.is_all());
    /// assert!(!Direction4::NONE.is_all());
    /// ```
    #[inline]
    pub fn is_all(&self) -> bool {
        self.0 == Self::ALL.0
    }

    /// Tests whether this direction encodes "no" directions.
    ///
    /// ## Example
    /// ```
    /// use amaze::direction4::Direction4;
    /// assert!(Direction4::NONE.is_none());
    /// assert!(!Direction4::NORTH.is_none());
    /// ```
    #[inline]
    pub fn is_none(&self) -> bool {
        self.0 == Self::NONE.0
    }

    /// Tests whether this direction encodes a trivial direction.
    ///
    /// ## Example
    /// ```
    /// use amaze::direction4::Direction4;
    /// assert!(Direction4::NORTH.is_trivial());
    /// assert!(!Direction4::ALL.is_trivial());
    /// assert!(!Direction4::NONE.is_trivial());
    /// ```
    pub fn is_trivial(&self) -> bool {
        match *self {
            Self::NORTH => true,
            Self::SOUTH => true,
            Self::EAST => true,
            Self::WEST => true,
            _ => false,
        }
    }

    /// Adds a new direction to this value.
    ///
    /// ## Example
    /// ```
    /// use amaze::direction4::Direction4;
    /// let mut direction = Direction4::NORTH;
    /// direction
    ///     .include(Direction4::SOUTH)
    ///     .include(Direction4::EAST)
    ///     .include(Direction4::WEST);
    /// assert_eq!(direction, Direction4::ALL);
    /// assert_eq!(direction, Direction4::NORTH + Direction4::SOUTH + Direction4::EAST + Direction4::WEST);
    /// ```
    #[inline]
    pub fn include(&mut self, rhs: Self) -> &mut Self {
        *self += rhs;
        self
    }

    /// Removes a direction from this value.
    ///
    /// ## Example
    /// ```
    /// use amaze::direction4::Direction4;
    /// let mut direction = Direction4::ALL;
    /// direction
    ///     .remove(Direction4::SOUTH)
    ///     .remove(Direction4::EAST)
    ///     .remove(Direction4::WEST);
    /// assert_eq!(direction, Direction4::NORTH);
    /// assert_eq!(direction, Direction4::ALL - Direction4::SOUTH - Direction4::EAST - Direction4::WEST);
    /// ```
    #[inline]
    pub fn remove(&mut self, rhs: Self) -> &mut Self {
        *self -= rhs;
        self
    }

    /// Joins two direction values
    ///
    /// ## Example
    /// ```
    /// use amaze::direction4::Direction4;
    /// let direction = Direction4::NORTH
    ///     .join(Direction4::SOUTH)
    ///     .join(Direction4::EAST)
    ///     .join(Direction4::WEST);
    /// assert_eq!(direction, Direction4::ALL);
    /// ```
    #[inline]
    pub const fn join(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl Default for Direction4 {
    #[inline]
    fn default() -> Self {
        Self::NONE
    }
}

impl Add for Direction4 {
    type Output = Self;

    /// Joins two direction values.
    ///
    /// ## Example
    /// ```
    /// use amaze::direction4::Direction4;
    /// let direction = Direction4::NORTH + Direction4::SOUTH + Direction4::EAST + Direction4::WEST;
    /// assert_eq!(direction, Direction4::ALL);
    /// ```
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl AddAssign for Direction4 {
    /// Joins two direction values while assigning.
    ///
    /// ## Example
    /// ```
    /// use amaze::direction4::Direction4;
    /// let mut direction = Direction4::NORTH;
    /// direction += Direction4::SOUTH;
    /// direction += Direction4::EAST;
    /// direction += Direction4::WEST;
    /// assert_eq!(direction, Direction4::ALL);
    /// ```
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 = &self.0 | rhs.0;
    }
}

impl Sub for Direction4 {
    type Output = Self;

    /// Removes a direction value from a direction.
    ///
    /// ## Example
    /// ```
    /// use amaze::direction4::Direction4;
    /// let direction = Direction4::ALL - Direction4::SOUTH - Direction4::EAST - Direction4::WEST;
    /// assert_eq!(direction, Direction4::NORTH);
    /// ```
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self((self.0 & !rhs.0) & Direction4::MASK)
    }
}

impl SubAssign for Direction4 {
    /// Removes a direction value from a direction while assigning.
    ///
    /// ## Example
    /// ```
    /// use amaze::direction4::Direction4;
    /// let mut direction = Direction4::ALL;
    /// direction -= Direction4::SOUTH;
    /// direction -= Direction4::EAST;
    /// direction -= Direction4::WEST;
    /// assert_eq!(direction, Direction4::NORTH);
    /// ```
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = (&self.0 & !rhs.0) & Direction4::MASK;
    }
}

impl Not for Direction4 {
    type Output = Direction4;

    /// Negates a direction value.
    ///
    /// ## Example
    /// ```
    /// use amaze::direction4::Direction4;
    /// assert_eq!(!Direction4::ALL, Direction4::NONE);
    /// assert_eq!(!Direction4::NONE, Direction4::ALL);
    /// assert_eq!(!(Direction4::NORTH + Direction4::SOUTH), Direction4::EAST + Direction4::WEST);
    /// ```
    fn not(self) -> Self::Output {
        Self((!self.0) & Self::MASK)
    }
}

impl Deref for Direction4 {
    type Target = u8;

    /// Gets the byte representation of a direction.
    ///
    /// ## Example
    /// ```
    /// use amaze::direction4::Direction4;
    /// assert_eq!(*Direction4::ALL, 0b1111u8);
    /// assert_eq!(*Direction4::NONE, 0u8);
    /// ```
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Direction4Iterator {
    direction: Direction4,
    step: u8,
}

impl Iterator for Direction4Iterator {
    type Item = Direction4;

    fn next(&mut self) -> Option<Self::Item> {
        while self.step < 4 {
            let current = Direction4(1 << &self.step);
            self.step += 1;
            if self.direction.contains(current) {
                return Some(current);
            }
        }

        None
    }
}

impl IntoIterator for Direction4 {
    type Item = Direction4;
    type IntoIter = Direction4Iterator;

    fn into_iter(self) -> Self::IntoIter {
        Direction4Iterator {
            direction: self,
            step: 0,
        }
    }
}

impl Debug for Direction4 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "0b{0:04b} ", self.0)?;

        match *self {
            Self::NONE => {
                write!(f, "(none)")?;
            }
            dir => {
                write!(f, "(")?;
                if dir.contains(Self::NORTH) {
                    write!(f, "N")?;
                }
                if dir.contains(Self::SOUTH) {
                    write!(f, "S")?;
                }
                if dir.contains(Self::EAST) {
                    write!(f, "E")?;
                }
                if dir.contains(Self::WEST) {
                    write!(f, "W")?;
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
        assert_eq!(Direction4::default(), Direction4::NONE);
        assert!(Direction4::default().is_none());
        assert_eq!(*Direction4::default(), 0);
    }

    #[test]
    fn all_contains_all_directions() {
        assert!(Direction4::ALL.contains(Direction4::NORTH));
        assert!(Direction4::ALL.contains(Direction4::SOUTH));
        assert!(Direction4::ALL.contains(Direction4::EAST));
        assert!(Direction4::ALL.contains(Direction4::WEST));

        assert!(Direction4::ALL.contains(Direction4::NONE));
        assert!(Direction4::ALL.contains(Direction4::ALL));
        assert!(Direction4::ALL.is_all());

        assert_eq!(*Direction4::ALL, 0b1111);
    }

    #[test]
    fn any_direction_is_not_none() {
        assert!(!Direction4::NORTH.is_none());
        assert!(!Direction4::SOUTH.is_none());
        assert!(!Direction4::EAST.is_none());
        assert!(!Direction4::WEST.is_none());
    }

    #[test]
    fn all_directions_is_all() {
        assert!(Direction4::ALL.is_all());
    }

    #[test]
    fn any_direction_is_not_all() {
        assert!(!Direction4::NORTH.is_all());
        assert!(!Direction4::SOUTH.is_all());
        assert!(!Direction4::EAST.is_all());
        assert!(!Direction4::WEST.is_all());
        assert!(!Direction4::NONE.is_all());
    }

    #[test]
    fn add_combines_directions() {
        let direction = Direction4::NORTH + Direction4::SOUTH;
        assert!(direction.contains(Direction4::NORTH));
        assert!(direction.contains(Direction4::SOUTH));
        assert!(!direction.contains(Direction4::EAST));
        assert!(!direction.contains(Direction4::WEST));
    }

    #[test]
    fn sub_removes_directions() {
        let direction = Direction4::ALL - Direction4::EAST - Direction4::WEST;
        assert!(direction.contains(Direction4::NORTH));
        assert!(direction.contains(Direction4::SOUTH));
        assert!(!direction.contains(Direction4::EAST));
        assert!(!direction.contains(Direction4::WEST));
    }

    #[test]
    fn include_combines_directions() {
        let mut direction = Direction4::NONE;
        direction
            .include(Direction4::NORTH)
            .include(Direction4::SOUTH);
        assert_eq!(direction, Direction4::NORTH + Direction4::SOUTH);
    }

    #[test]
    fn add_assign_combines_directions() {
        let mut direction = Direction4::NONE;
        direction += Direction4::NORTH;
        direction += Direction4::SOUTH;
        assert!(direction.contains(Direction4::NORTH));
        assert!(direction.contains(Direction4::SOUTH));
        assert!(!direction.contains(Direction4::EAST));
        assert!(!direction.contains(Direction4::WEST));
    }

    #[test]
    fn sub_assign_removes_directions() {
        let mut direction = Direction4::ALL;
        direction -= Direction4::EAST;
        direction -= Direction4::WEST;
        assert!(direction.contains(Direction4::NORTH));
        assert!(direction.contains(Direction4::SOUTH));
        assert!(!direction.contains(Direction4::EAST));
        assert!(!direction.contains(Direction4::WEST));
    }

    #[test]
    fn remove_removes_directions() {
        let mut direction = Direction4::ALL;
        direction.remove(Direction4::EAST).remove(Direction4::WEST);
        assert_eq!(direction, Direction4::NORTH + Direction4::SOUTH);
    }

    #[test]
    fn into_iter_for_all_enumerates_all() {
        let dirs: Vec<_> = Direction4::ALL.into_iter().collect();
        assert!(dirs.contains(&Direction4::NORTH));
        assert!(dirs.contains(&Direction4::SOUTH));
        assert!(dirs.contains(&Direction4::EAST));
        assert!(dirs.contains(&Direction4::WEST));
    }

    #[test]
    fn into_iter_for_combination_enumerates_contained() {
        let dirs: Vec<_> = (Direction4::NORTH + Direction4::WEST).into_iter().collect();
        assert!(dirs.contains(&Direction4::NORTH));
        assert!(dirs.contains(&Direction4::WEST));
        assert!(!dirs.contains(&Direction4::SOUTH));
        assert!(!dirs.contains(&Direction4::EAST));
    }

    #[test]
    fn debug_lists_directions() {
        assert_eq!(format!("{:?}", Direction4::NONE), "0b0000 (none)");
        assert_eq!(format!("{:?}", Direction4::ALL), "0b1111 (NSEW)");
        assert_eq!(format!("{:?}", Direction4::NORTH), "0b0001 (N)");
        assert_eq!(format!("{:?}", Direction4::SOUTH), "0b0010 (S)");
        assert_eq!(format!("{:?}", Direction4::EAST), "0b0100 (E)");
        assert_eq!(format!("{:?}", Direction4::WEST), "0b1000 (W)");

        assert_eq!(
            format!("{:?}", Direction4::NORTH + Direction4::EAST),
            "0b0101 (NE)"
        );
        assert_eq!(
            format!(
                "{:?}",
                Direction4::NORTH + Direction4::EAST + Direction4::WEST
            ),
            "0b1101 (NEW)"
        );
    }

    #[test]
    fn not_all_is_none() {
        assert_eq!(!Direction4::ALL, Direction4::NONE);
    }

    #[test]
    fn not_inverts_selection() {
        assert_eq!(
            !(Direction4::NORTH + Direction4::SOUTH),
            Direction4::WEST + Direction4::EAST
        );
    }

    #[test]
    fn is_trivial_work() {
        assert!(Direction4::NORTH.is_trivial());
        assert!(Direction4::SOUTH.is_trivial());
        assert!(Direction4::EAST.is_trivial());
        assert!(Direction4::WEST.is_trivial());

        assert!(!Direction4::NONE.is_trivial());
        assert!(!Direction4::ALL.is_trivial());
        assert!(!(Direction4::NORTH + Direction4::EAST).is_trivial());
    }
}
