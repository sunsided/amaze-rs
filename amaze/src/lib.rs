pub mod direction4;
pub mod direction6;
pub mod dungeon;
pub mod generators;
mod grid_coord_2d;
mod hex_coord;
pub mod path;
#[cfg(feature = "representations")]
pub mod representations;

#[cfg(any(feature = "unicode-renderer", feature = "pgm-renderer"))]
pub mod renderers;
pub mod room4;
pub mod room4_list;
#[cfg(feature = "solvers")]
pub mod solvers;
pub mod stats;
#[cfg(any(
    feature = "binary-format",
    feature = "json-format",
    feature = "file-io"
))]
pub mod storage;
mod visit_map_2d;
mod wall4_grid;
mod wall6_grid;

pub mod preamble {
    pub use crate::direction4::{Direction4, Direction4Iterator};
    pub use crate::direction6::{Direction6, Direction6Iterator};
    pub use crate::dungeon::{DungeonGrid, DungeonType, TileType};
    pub use crate::grid_coord_2d::{GetCoordinateBounds2D, GridCoord2D, LinearizeCoords2D};
    pub use crate::hex_coord::HexCoord;
    pub use crate::path::Path;
    #[cfg(feature = "representations")]
    pub use crate::representations::{
        AdjacencyList, Edge, EdgeList, HexAdjacencyList, HexEdge, HexEdgeList, HexWeightedEdge,
        PassabilityGrid, WeightedAdjacencyList, WeightedEdge, WeightedEdgeList,
        WeightedHexAdjacencyList, WeightedHexEdgeList,
    };
    pub use crate::room4::{Door4, Room4, Wall4};
    pub use crate::room4_list::{Room4List, RoomIndex};
    #[cfg(feature = "solvers")]
    pub use crate::solvers::{AStarSolver, BfsSolver, DeadEndFillingSolver, DfsSolver, MazeSolver};
    pub use crate::stats::MazeStats;
    pub use crate::wall4_grid::Wall4Grid;
    pub use crate::wall6_grid::{Wall6, Wall6Grid};
}
