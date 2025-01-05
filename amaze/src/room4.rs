use crate::direction4::Direction4;
use crate::room4_list::{EnsureIndexConsistency, Room4List, RoomIndex};

/// Encodes the walls in a four-sided room.
pub type Wall4 = Direction4;

/// Encodes the doors in a four-sided room.
pub type Door4 = Direction4;

/// A four-sided room.
#[derive(Debug)]
pub struct Room4<Tag = ()> {
    index: RoomIndex,
    north: Option<RoomIndex>,
    south: Option<RoomIndex>,
    east: Option<RoomIndex>,
    west: Option<RoomIndex>,
    pub tag: Tag,
}

impl<Tag> Room4<Tag> {
    pub(crate) fn new_empty(index: RoomIndex, tag: Tag) -> Self {
        Self {
            tag,
            index,
            north: None,
            south: None,
            east: None,
            west: None,
        }
    }

    pub fn set_room(&mut self, direction: Direction4, room: Option<RoomIndex>) {
        match direction {
            Direction4::NORTH => self.set_north(room),
            Direction4::SOUTH => self.set_south(room),
            Direction4::EAST => self.set_east(room),
            Direction4::WEST => self.set_west(room),
            _ => panic!("Must specify a trivial direction, got {:?}", direction),
        }
    }

    pub fn set_north<R: Into<Option<RoomIndex>>>(&mut self, room: R) {
        self.north = room.into();
    }

    pub fn set_south<R: Into<Option<RoomIndex>>>(&mut self, room: R) {
        self.south = room.into();
    }

    pub fn set_east<R: Into<Option<RoomIndex>>>(&mut self, room: R) {
        self.east = room.into();
    }

    pub fn set_west<R: Into<Option<RoomIndex>>>(&mut self, room: R) {
        self.west = room.into();
    }

    pub fn north(&self) -> Option<RoomIndex> {
        self.north
    }

    pub fn east(&self) -> Option<RoomIndex> {
        self.east
    }

    pub fn south(&self) -> Option<RoomIndex> {
        self.south
    }

    pub fn west(&self) -> Option<RoomIndex> {
        self.west
    }

    pub fn has_neighbor(&mut self, direction: Direction4) -> bool {
        self.get_neighbor(direction).is_some()
    }

    pub fn has_neighbors(&mut self) -> bool {
        self.north.is_some() | self.south.is_some() | self.east.is_some() | self.west.is_some()
    }

    pub fn get_neighbor(&mut self, direction: Direction4) -> Option<RoomIndex> {
        match direction {
            Direction4::NORTH => self.north,
            Direction4::SOUTH => self.south,
            Direction4::EAST => self.east,
            Direction4::WEST => self.west,
            _ => panic!("Must specify a trivial direction, got {:?}", direction),
        }
    }

    pub fn doors(&self) -> Door4 {
        let mut doors = Door4::NONE;
        if self.north.is_some() {
            doors += Door4::NORTH;
        }
        if self.south.is_some() {
            doors += Door4::SOUTH;
        }
        if self.east.is_some() {
            doors += Door4::EAST;
        }
        if self.west.is_some() {
            doors += Door4::WEST;
        }
        doors
    }

    #[inline]
    pub fn walls(&self) -> Wall4 {
        !self.doors()
    }

    #[inline]
    pub fn index(&self) -> RoomIndex {
        self.index
    }
}

fn ensure_index_matches_and_apply(
    name: &str,
    self_index: RoomIndex,
    other_index: RoomIndex,
    existing_value: &mut Option<RoomIndex>,
) {
    debug_assert!(
        existing_value.is_none() || existing_value.unwrap() == self_index,
        "Attempted to set '{name}' index of {other_index} to {self_index}, but it was already set to a {conflicting_value}",
        name = name,
        other_index = other_index,
        self_index = self_index,
        conflicting_value = existing_value.unwrap()
    );

    *existing_value = self_index.into();
}

impl<Tag> EnsureIndexConsistency<Tag> for Room4<Tag> {
    fn propagate_index_to_neighbors(&self, list: &mut Room4List<Tag>) {
        if let Some(north) = self.north {
            let other = &mut list[north];
            ensure_index_matches_and_apply("south", self.index, other.index, &mut other.south);
        }
        if let Some(south) = self.south {
            let other = &mut list[south];
            ensure_index_matches_and_apply("north", self.index, other.index, &mut other.north);
        }
        if let Some(east) = self.east {
            let other = &mut list[east];
            ensure_index_matches_and_apply("west", self.index, other.index, &mut other.west);
        }
        if let Some(west) = self.west {
            let other = &mut list[west];
            ensure_index_matches_and_apply("east", self.index, other.index, &mut other.east);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::room4_list::Room4List;

    #[test]
    fn empty_room_has_no_neighbors() {
        let index = RoomIndex::from(0).unwrap();
        let empty = Room4::new_empty(index, 0);
        assert!(empty.north().is_none());
        assert!(empty.south().is_none());
        assert!(empty.east().is_none());
        assert!(empty.west().is_none());
    }

    #[test]
    fn it_works() {
        let mut list = Room4List::default();
        let n = list.push_default(0);
        let s = list.push_default(1);
        let e = list.push_default(2);
        let w = list.push_default(3);

        let c = list.push_new(42, |room| {
            room.set_north(n);
            room.set_south(s);
            room.set_east(e);
            room.set_west(w);
        });

        // The center node is now fully 4-connected.
        let center = &list[c];
        assert_eq!(center.index(), c);
        assert_eq!(center.north(), Some(n));
        assert_eq!(center.south(), Some(s));
        assert_eq!(center.east(), Some(e));
        assert_eq!(center.west(), Some(w));
        assert_eq!(center.doors(), Door4::ALL);
        assert_eq!(center.walls(), Wall4::NONE);

        // The north node is only connected to the south.
        let north = &list[n];
        assert_eq!(north.index(), n);
        assert_eq!(north.north(), None);
        assert_eq!(north.south(), Some(c));
        assert_eq!(north.east(), None);
        assert_eq!(north.west(), None);
        assert_eq!(north.doors(), Door4::SOUTH);
        assert_eq!(north.walls(), Wall4::NORTH + Wall4::EAST + Wall4::WEST);

        // The south node is only connected to the north.
        let south = &list[s];
        assert_eq!(south.index(), s);
        assert_eq!(south.north(), Some(c));
        assert_eq!(south.south(), None);
        assert_eq!(south.east(), None);
        assert_eq!(south.west(), None);
        assert_eq!(south.doors(), Door4::NORTH);
        assert_eq!(south.walls(), Wall4::SOUTH + Wall4::EAST + Wall4::WEST);

        // The east node is only connected to the west.
        let east = &list[e];
        assert_eq!(east.index(), e);
        assert_eq!(east.north(), None);
        assert_eq!(east.south(), None);
        assert_eq!(east.east(), None);
        assert_eq!(east.west(), Some(c));
        assert_eq!(east.doors(), Door4::WEST);
        assert_eq!(east.walls(), Wall4::NORTH + Wall4::SOUTH + Wall4::EAST);

        // The east node is only connected to the west.
        let west = &list[w];
        assert_eq!(west.index(), w);
        assert_eq!(west.north(), None);
        assert_eq!(west.south(), None);
        assert_eq!(west.east(), Some(c));
        assert_eq!(west.west(), None);
        assert_eq!(west.doors(), Door4::EAST);
        assert_eq!(west.walls(), Wall4::NORTH + Wall4::SOUTH + Wall4::WEST);
    }
}
