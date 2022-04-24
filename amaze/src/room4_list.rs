use crate::room4::Room4;
use std::fmt::{Debug, Display, Formatter};
use std::num::NonZeroUsize;
use std::ops::{Deref, Index, IndexMut};

/// An index for addressing the room in the room list.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RoomIndex(NonZeroUsize);

#[derive(Debug, Default)]
pub struct Room4List<Tag = ()> {
    rooms: Vec<Room4<Tag>>,
}

pub(crate) trait EnsureIndexConsistency<Tag> {
    /// Verifies and ensures that neighbor indexes are correctly set in the list.
    fn propagate_index_to_neighbors(&self, list: &mut Room4List<Tag>);
}

impl<Tag> Room4List<Tag> {
    /// Adds a new, empty room to the list.
    ///
    /// ## Arguments
    /// * `tag` - The tag to pass to the room.
    ///
    /// ## Returns
    /// The index of the newly created room.
    pub fn push_default(&mut self, tag: Tag) -> RoomIndex {
        self.push_new(tag, |_| {})
    }

    /// Adds a new room to the list and configures it.
    ///
    /// ## Remarks
    /// This method ensures that all neighboring indexes are correctly configured.
    ///
    /// ## Arguments
    /// * `tag` - The tag to pass to the room.
    /// * `f` - A function to configure the room after creation.
    ///
    /// ## Returns
    /// The index of the newly created room.
    pub fn push_new<F>(&mut self, tag: Tag, f: F) -> RoomIndex
    where
        F: FnOnce(&mut Room4<Tag>) -> (),
    {
        // SAFETY:
        // By increasing the current size of the index by 1,
        // the resulting value is always nonzero.
        let index = RoomIndex(unsafe { NonZeroUsize::new_unchecked(self.rooms.len() + 1) });

        let mut room = Room4::new_empty(index, tag);
        f(&mut room);

        // Update indexes in all neighbors.
        if room.has_neighbors() {
            room.propagate_index_to_neighbors(self);
        }

        self.rooms.push(room);
        index
    }

    /// Returns the number of rooms in this list.
    pub fn len(&self) -> usize {
        self.rooms.len()
    }

    /// Indicates whether this list is empty.
    pub fn is_empty(&self) -> bool {
        self.rooms.is_empty()
    }

    pub fn get(&self, index: RoomIndex) -> Option<&Room4<Tag>> {
        let index = (*index).get() - 1;
        self.rooms.get(index)
    }

    pub fn get_mut(&mut self, index: RoomIndex) -> Option<&mut Room4<Tag>> {
        let index = (*index).get() - 1;
        self.rooms.get_mut(index)
    }
}

impl<Tag> Index<RoomIndex> for Room4List<Tag> {
    type Output = Room4<Tag>;

    fn index(&self, index: RoomIndex) -> &Self::Output {
        let index = (*index).get() - 1;
        &self.rooms[index]
    }
}

impl<Tag> Index<usize> for Room4List<Tag> {
    type Output = Room4<Tag>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.rooms[index]
    }
}

impl<Tag> IndexMut<RoomIndex> for Room4List<Tag> {
    fn index_mut(&mut self, index: RoomIndex) -> &mut Self::Output {
        let index = (*index).get() - 1;
        &mut self.rooms[index]
    }
}

impl<Tag> IndexMut<usize> for Room4List<Tag> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.rooms[index]
    }
}

impl RoomIndex {
    pub fn from(value: usize) -> Option<RoomIndex> {
        if value == usize::MAX {
            return None;
        }
        match NonZeroUsize::new(value + 1) {
            Some(index) => Some(RoomIndex(index)),
            None => None,
        }
    }
}

impl Deref for RoomIndex {
    type Target = NonZeroUsize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Debug for RoomIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.get() - 1)
    }
}

impl Display for RoomIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct Room4ListIntoIterator<Tag> {
    list: Vec<Room4<Tag>>,
}

impl<Tag> Iterator for Room4ListIntoIterator<Tag> {
    type Item = Room4<Tag>;

    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop()
    }
}

impl<Tag> IntoIterator for Room4List<Tag> {
    type Item = Room4<Tag>;
    type IntoIter = Room4ListIntoIterator<Tag>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter { list: self.rooms }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_returns_some() {
        let mut list = Room4List::default();
        let idx_a = list.push_default("a");
        let idx_b = list.push_default("b");
        assert_eq!(list.get(idx_a).unwrap().tag, "a");
        assert_eq!(list.get(idx_b).unwrap().tag, "b");
    }

    #[test]
    fn index_returns_some() {
        let mut list = Room4List::default();
        let idx_a = list.push_default("a");
        let idx_b = list.push_default("b");
        assert_ne!(idx_a, idx_b);

        // index via RoomIndex
        assert_eq!(list[idx_a].tag, "a");
        assert_eq!(list[idx_b].tag, "b");

        // index via usize
        assert_eq!(list[0].tag, "a");
        assert_eq!(list[1].tag, "b");
    }

    #[test]
    fn get_mut_returns_some() {
        let mut list = Room4List::default();
        let idx_a = list.push_default("a");
        let idx_b = list.push_default("b");
        assert_eq!(list.get_mut(idx_a).unwrap().tag, "a");
        assert_eq!(list.get_mut(idx_b).unwrap().tag, "b");
    }

    #[test]
    fn index_mut_returns_some() {
        let mut list = Room4List::default();
        let idx_a = list.push_default("a");
        let idx_b = list.push_default("b");

        // index via RoomIndex
        let room_a = &mut list[idx_a];
        assert_eq!(room_a.tag, "a");
        let room_b = &mut list[idx_b];
        assert_eq!(room_b.tag, "b");

        // index via usize
        let room_a = &mut list[0];
        assert_eq!(room_a.tag, "a");
        let room_b = &mut list[1];
        assert_eq!(room_b.tag, "b");
    }

    #[test]
    fn get_with_invalid_index_returns_none() {
        let mut list = Room4List::default();
        let _ = list.push_default("a");

        let invalid_idx = RoomIndex(unsafe { NonZeroUsize::new_unchecked(2) });
        assert!(list.get(invalid_idx).is_none());
    }
}
