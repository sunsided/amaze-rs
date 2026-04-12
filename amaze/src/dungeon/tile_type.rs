/// Tile type for dungeon cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TileType {
    /// Empty/uninitialized tile
    Empty,
    /// Floor tile (passable)
    Floor,
    /// Wall tile (impassable)
    Wall,
}

impl Default for TileType {
    fn default() -> Self {
        TileType::Empty
    }
}

impl TileType {
    /// Returns true if this tile is passable (floor).
    #[inline]
    pub fn is_passable(self) -> bool {
        matches!(self, TileType::Floor)
    }

    /// Returns true if this tile is a wall.
    #[inline]
    pub fn is_wall(self) -> bool {
        matches!(self, TileType::Wall)
    }

    /// Returns true if this tile is empty.
    #[inline]
    pub fn is_empty(self) -> bool {
        matches!(self, TileType::Empty)
    }
}
