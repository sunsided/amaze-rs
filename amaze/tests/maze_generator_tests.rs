//! Integration tests for maze generators and Wall4Grid behavior
//!
//! Tests that maze generators produce valid spanning trees and that
//! Wall4Grid provides correct statistics and conversions.

#![cfg(feature = "representations")]

use amaze::generators::{
    BinaryTree4, Eller4, GrowingTree4, HuntAndKill4, Kruskal4, MazeGenerator2D, Prim4,
    RecursiveBacktracker4, Sidewinder4, Wilson4,
};
use amaze::preamble::{GridCoord2D, Wall4Grid};

#[cfg(feature = "solvers")]
use amaze::solvers::{BfsSolver, MazeSolver};

#[cfg(feature = "generator-hex")]
use amaze::direction6::Direction6;
#[cfg(feature = "generator-hex")]
use amaze::generators::{AldousBroder6, GrowingTree6, MazeGenerator6D, RecursiveBacktracker6};
#[cfg(feature = "generator-hex")]
use amaze::preamble::{HexCoord, Wall6Grid};

/// Helper function to verify a maze is a connected spanning tree
fn assert_connected_tree(grid: &Wall4Grid) {
    let total_cells = grid.width() * grid.height();

    // Count the number of edges (passages) in the maze
    let mut edge_count = 0;
    for y in 0..grid.height() {
        for x in 0..grid.width() {
            let coord = GridCoord2D::new(x, y);
            let walls = grid.get(coord).expect("coord in bounds");

            // Count each passage (avoid double-counting by only checking right and down)
            if x + 1 < grid.width() && !walls.contains(amaze::direction4::Direction4::EAST) {
                edge_count += 1;
            }
            if y + 1 < grid.height() && !walls.contains(amaze::direction4::Direction4::SOUTH) {
                edge_count += 1;
            }
        }
    }

    // A spanning tree has exactly n-1 edges for n vertices
    assert_eq!(
        edge_count,
        total_cells - 1,
        "Maze should be a spanning tree with {} edges",
        total_cells - 1
    );

    // Additional check: verify the maze is connected by ensuring all cells are reachable
    #[cfg(feature = "solvers")]
    {
        let start = GridCoord2D::new(0, 0);
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                let end = GridCoord2D::new(x, y);
                if start != end {
                    let path = BfsSolver.solve(grid, start, end);
                    assert!(
                        path.is_some(),
                        "Cell ({}, {}) should be reachable from (0, 0)",
                        x,
                        y
                    );
                }
            }
        }
    }
}

/// Test that all maze generators produce valid spanning trees
#[test]
fn all_generators_produce_spanning_trees() {
    let size = (12, 12);
    let mut grids = Vec::new();

    grids.push(RecursiveBacktracker4::new_from_seed(42).generate(size.0, size.1));
    grids.push(<GrowingTree4 as MazeGenerator2D>::new_from_seed(42).generate(size.0, size.1));
    grids.push(<Kruskal4 as MazeGenerator2D>::new_from_seed(42).generate(size.0, size.1));
    grids.push(<Eller4 as MazeGenerator2D>::new_from_seed(42).generate(size.0, size.1));
    grids.push(<Wilson4 as MazeGenerator2D>::new_from_seed(42).generate(size.0, size.1));
    grids.push(<HuntAndKill4 as MazeGenerator2D>::new_from_seed(42).generate(size.0, size.1));
    grids.push(<Sidewinder4 as MazeGenerator2D>::new_from_seed(42).generate(size.0, size.1));
    grids.push(<BinaryTree4 as MazeGenerator2D>::new_from_seed(42).generate(size.0, size.1));
    grids.push(<Prim4 as MazeGenerator2D>::new_from_seed(42).generate(size.0, size.1));

    for grid in grids {
        assert_connected_tree(&grid);
    }
}

/// Test that Wall4Grid converts to room list correctly
#[test]
fn wall4grid_converts_to_room_list() {
    let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
    let rooms = maze.to_room_list(|coord| coord);
    assert_eq!(rooms.len(), 64);
}

/// Test that Wall4Grid stats include non-zero longest path
#[test]
fn wall4grid_stats_has_longest_path() {
    let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
    let stats = maze.stats();
    assert!(stats.longest_path > 0);
}

/// Test that RecursiveBacktracker produces spanning trees of various sizes
#[test]
fn recursive_backtracker_produces_valid_trees() {
    for size in [5, 10, 15, 20] {
        let maze = RecursiveBacktracker4::new_from_seed(123).generate(size, size);
        assert_connected_tree(&maze);
    }
}

/// Test that GrowingTree produces spanning trees
#[test]
fn growing_tree_produces_valid_trees() {
    let maze = <GrowingTree4 as MazeGenerator2D>::new_from_seed(456).generate(10, 10);
    assert_connected_tree(&maze);
}

/// Test that Kruskal produces spanning trees
#[test]
fn kruskal_produces_valid_trees() {
    let maze = <Kruskal4 as MazeGenerator2D>::new_from_seed(789).generate(10, 10);
    assert_connected_tree(&maze);
}

/// Test that Eller produces spanning trees
#[test]
fn eller_produces_valid_trees() {
    let maze = <Eller4 as MazeGenerator2D>::new_from_seed(321).generate(10, 10);
    assert_connected_tree(&maze);
}

/// Test that Wilson produces spanning trees
#[test]
fn wilson_produces_valid_trees() {
    let maze = <Wilson4 as MazeGenerator2D>::new_from_seed(654).generate(10, 10);
    assert_connected_tree(&maze);
}

/// Test that HuntAndKill produces spanning trees
#[test]
fn hunt_and_kill_produces_valid_trees() {
    let maze = <HuntAndKill4 as MazeGenerator2D>::new_from_seed(987).generate(10, 10);
    assert_connected_tree(&maze);
}

/// Test that Sidewinder produces spanning trees
#[test]
fn sidewinder_produces_valid_trees() {
    let maze = <Sidewinder4 as MazeGenerator2D>::new_from_seed(147).generate(10, 10);
    assert_connected_tree(&maze);
}

/// Test that BinaryTree produces spanning trees
#[test]
fn binary_tree_produces_valid_trees() {
    let maze = <BinaryTree4 as MazeGenerator2D>::new_from_seed(258).generate(10, 10);
    assert_connected_tree(&maze);
}

/// Test that Prim produces spanning trees
#[test]
fn prim_produces_valid_trees() {
    let maze = <Prim4 as MazeGenerator2D>::new_from_seed(369).generate(10, 10);
    assert_connected_tree(&maze);
}

#[cfg(feature = "generator-hex")]
fn assert_connected_tree_hex(grid: &Wall6Grid) {
    let total_cells = grid.width() * grid.height();

    let mut edge_count = 0;
    for coord in grid.coords() {
        let walls = grid.get(coord).expect("coord in bounds");
        for dir in [Direction6::EAST, Direction6::NE, Direction6::SE] {
            if coord
                .try_neighbor(dir, grid.width(), grid.height())
                .is_some()
                && !walls.contains(dir)
            {
                edge_count += 1;
            }
        }
    }

    assert_eq!(
        edge_count,
        total_cells - 1,
        "Hex maze should be a spanning tree with {} edges",
        total_cells - 1
    );

    let start = HexCoord::new(0, 0);
    let mut visited = vec![false; total_cells];
    let mut queue = std::collections::VecDeque::new();
    visited[0] = true;
    queue.push_back(start);
    let mut count = 0;

    while let Some(cell) = queue.pop_front() {
        count += 1;
        for n in grid.open_neighbors(cell) {
            let idx = (n.r as usize) * grid.width() + n.q as usize;
            if !visited[idx] {
                visited[idx] = true;
                queue.push_back(n);
            }
        }
    }

    assert_eq!(
        count, total_cells,
        "All {} cells should be reachable from (0,0), but only {} were",
        total_cells, count
    );
}

#[cfg(feature = "generator-hex")]
#[test]
fn hex_generators_produce_spanning_trees() {
    let size = (10, 10);

    let grid1 = RecursiveBacktracker6::new_from_seed(42).generate(size.0, size.1);
    assert_connected_tree_hex(&grid1);

    let grid2 = <GrowingTree6 as MazeGenerator6D>::new_from_seed(42).generate(size.0, size.1);
    assert_connected_tree_hex(&grid2);

    let grid3 = AldousBroder6::new_from_seed(42).generate(size.0, size.1);
    assert_connected_tree_hex(&grid3);
}

#[cfg(feature = "generator-hex")]
#[test]
fn recursive_backtracker6_produces_valid_trees() {
    for size in [5, 10, 15] {
        let maze = RecursiveBacktracker6::new_from_seed(123).generate(size, size);
        assert_connected_tree_hex(&maze);
    }
}

#[cfg(feature = "generator-hex")]
#[test]
fn growing_tree6_produces_valid_trees() {
    let maze = <GrowingTree6 as MazeGenerator6D>::new_from_seed(456).generate(10, 10);
    assert_connected_tree_hex(&maze);
}

#[cfg(feature = "generator-hex")]
#[test]
fn aldous_broder6_produces_valid_trees() {
    let maze = AldousBroder6::new_from_seed(789).generate(10, 10);
    assert_connected_tree_hex(&maze);
}
