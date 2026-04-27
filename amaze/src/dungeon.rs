//! Procedural dungeon generation and representation.
//!
//! This module provides dungeon-specific data structures and algorithms,
//! distinct from perfect-maze generators. Dungeons consist of floor and wall
//! tiles with optional metadata (exit markers, edge masks for rendering).

mod dungeon_grid;
mod dungeon_type;
mod dyn_dungeon_grid;
pub mod generators;
#[cfg(all(feature = "representations", feature = "solvers"))]
pub mod solvers;
mod tile_type;

pub use dungeon_grid::DungeonGrid;
pub use dungeon_type::DungeonType;
pub use dyn_dungeon_grid::DynDungeonGrid;
pub use generators::{
    DungeonGenerationStep, DungeonGenerationSteps, DungeonGenerationVisitor, DungeonGenerator,
    DungeonWalkGenerator, VecDungeonGenerationVisitor,
};
#[cfg(all(feature = "representations", feature = "solvers"))]
pub use solvers::{solve_astar, solve_bfs};
pub use tile_type::TileType;
