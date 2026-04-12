//! Procedural dungeon generation and representation.
//!
//! This module provides dungeon-specific data structures and algorithms,
//! distinct from perfect-maze generators. Dungeons consist of floor and wall
//! tiles with optional metadata (exit markers, edge masks for rendering).

mod dungeon_grid;
mod dungeon_type;
pub mod generators;
pub mod solvers;
mod tile_type;

pub use dungeon_grid::DungeonGrid;
pub use dungeon_type::DungeonType;
pub use generators::{
    DungeonGenerationStep, DungeonGenerationSteps, DungeonGenerationVisitor, DungeonGenerator,
    DungeonWalkGenerator, VecDungeonGenerationVisitor,
};
pub use solvers::{solve_astar, solve_bfs};
pub use tile_type::TileType;
