/// Dungeon generation algorithm type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DungeonType {
    /// Cavern-style dungeon using unconstrained random walk
    Caverns,
    /// Room-based dungeon with long corridors and stamped rooms
    Rooms,
    /// Winding corridors with probabilistic room suppression
    Winding,
}

impl DungeonType {
    /// Returns a human-readable name for this dungeon type.
    pub fn name(self) -> &'static str {
        match self {
            DungeonType::Caverns => "Caverns",
            DungeonType::Rooms => "Rooms",
            DungeonType::Winding => "Winding",
        }
    }

    /// Returns a description of this dungeon type.
    pub fn description(self) -> &'static str {
        match self {
            DungeonType::Caverns => "Unconstrained random walk creating organic caverns",
            DungeonType::Rooms => "Long corridors connecting rectangular rooms",
            DungeonType::Winding => "Winding corridors with occasional rooms",
        }
    }
}
