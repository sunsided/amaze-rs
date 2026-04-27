//! A playground for maze and procedural dungeon generation in Rust.
//!
//! `amaze` provides a collection of algorithms and utilities for creating,
//! solving, rendering, and serializing mazes and procedural dungeons.
//! It is the core library behind the `amaze-cli` and `amaze-gui` binaries,
//! suitable for games, simulations, and procedural-content experiments.
//!
//! # Features
//!
//! - **Perfect-maze generators** for 4-connected grids: recursive backtracker,
//!   growing tree, Kruskal, Eller, Wilson, hunt-and-kill, sidewinder,
//!   binary tree, and Prim.
//! - **Hexagonal (6-connected) generators**: recursive backtracker,
//!   growing tree, and Aldous-Broder.
//! - **Procedural dungeons**: caverns, rooms, and winding layouts.
//! - **Pathfinding solvers**: BFS, DFS, A\*, and dead-end filling,
//!   all implementing the shared [`preamble::MazeSolver`] trait.
//! - **Renderers**: Unicode box-drawing characters and PGM images,
//!   plus statistics via [`preamble::MazeStats`].
//! - **Graph representations**: adjacency lists, edge lists, passability
//!   grids (with hex variants), and optional `petgraph` integration.
//! - **Serialization**: binary format, JSON, and file I/O support.
//!
//! # Quick Start
//!
//! Generate a small rectangular maze and render it with Unicode box-drawing
//! characters:
//!
//! ```
//! use amaze::generators::RecursiveBacktracker4;
//! use amaze::preamble::Wall4Grid;
//! use amaze::renderers::{UnicodeRenderStyle, UnicodeRenderer};
//!
//! let generator = RecursiveBacktracker4::new_from_seed(0xdeadbeef);
//! let grid: Wall4Grid = generator.generate(6, 6);
//!
//! let renderer = UnicodeRenderer::new(UnicodeRenderStyle::Heavy, true);
//! let output = renderer.render(&grid);
//! assert!(!output.is_empty());
//! ```
//!
//! # Feature Flags
//!
//! The crate ships with a rich set of optional features. The defaults are
//! `renderers`, `generators`, `solvers`, and `representations`.
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `generators` | All 4-connected grid maze generation algorithms |
//! | `generators-hex` | All hexagonal (6-connected) maze generation algorithms |
//! | `solvers` | All maze solving algorithms (BFS, DFS, A\*, dead-end filling) |
//! | `renderers` | All rendering backends (Unicode + PGM) |
//! | `representations` | Standard 4-connected graph representations |
//! | `hex-representations` | Hexagonal maze representations |
//! | `dungeon-representations` | Dungeon/cave representations |
//! | `binary-format` | Binary serialization format |
//! | `json-format` | JSON serialization format |
//! | `file-io` | File I/O combining binary and JSON formats |
//! | `petgraph` | `petgraph` integration for graph operations |
//! | `serde` | Serde serialization support |
//!
//! Fine-grained sub-flags (e.g. `generator-kruskal`, `solver-bfs`) are
//! documented in `Cargo.toml` and on docs.rs.
//!
//! # Crate Layout
//!
//! The [`preamble`] module re-exports the most commonly used types.
//! Key modules: [`generators`], [`solvers`], [`renderers`],
//! [`dungeon`], [`representations`], and `storage` (with `file-io`).
//!
//! # Companion Crates
//!
//! - `amaze-cli` — command-line interface for generating and rendering mazes.
//! - `amaze-gui` — graphical user interface with live preview.
//!
//! # License
//!
//! Licensed under EUPL-1.2 OR MIT OR Apache-2.0.

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
