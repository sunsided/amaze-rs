pub mod direction4;
pub mod dungeon;
pub mod generators;
mod grid_coord_2d;
pub mod path;
#[cfg(feature = "representations")]
pub mod representations;

#[cfg(any(feature = "unicode-renderer", feature = "pgm-renderer"))]
pub mod renderers;
pub mod room4;
pub mod room4_list;
#[cfg(feature = "solvers")]
pub mod solvers;
mod stats;
mod visit_map_2d;
mod wall4_grid;

pub mod preamble {
    pub use crate::direction4::{Direction4, Direction4Iterator};
    pub use crate::dungeon::{DungeonGrid, DungeonType, TileType};
    pub use crate::grid_coord_2d::{GetCoordinateBounds2D, GridCoord2D, LinearizeCoords2D};
    pub use crate::path::Path;
    #[cfg(feature = "representations")]
    pub use crate::representations::{
        AdjacencyList, Edge, EdgeList, PassabilityGrid, WeightedAdjacencyList, WeightedEdge,
        WeightedEdgeList,
    };
    pub use crate::room4::{Door4, Room4, Wall4};
    pub use crate::room4_list::{Room4List, RoomIndex};
    #[cfg(feature = "solvers")]
    pub use crate::solvers::{AStarSolver, BfsSolver, DeadEndFillingSolver, DfsSolver, MazeSolver};
    pub use crate::stats::MazeStats;
    pub use crate::wall4_grid::Wall4Grid;
}
