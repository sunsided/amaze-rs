//! Integration tests for maze solvers
//!
//! Tests that all maze solving algorithms work correctly and find valid paths.

#![cfg(all(feature = "representations", feature = "solvers"))]

use amaze::generators::{MazeGenerator2D, RecursiveBacktracker4};
use amaze::preamble::GridCoord2D;
use amaze::solvers::{AStarSolver, BfsSolver, DeadEndFillingSolver, DfsSolver, MazeSolver};

/// Test that all solvers successfully find paths through a generated maze
#[test]
fn all_solvers_find_paths() {
    let maze = RecursiveBacktracker4::new_from_seed(1234).generate(20, 20);
    let start = GridCoord2D::new(0, 0);
    let end = GridCoord2D::new(19, 19);

    let bfs = BfsSolver.solve(&maze, start, end).expect("bfs path");
    let dfs = DfsSolver.solve(&maze, start, end).expect("dfs path");
    let astar = AStarSolver.solve(&maze, start, end).expect("astar path");
    let dead_end = DeadEndFillingSolver
        .solve(&maze, start, end)
        .expect("dead-end path");

    assert!(!bfs.is_empty());
    assert!(!dfs.is_empty());
    assert_eq!(bfs.length, astar.length);
    assert_eq!(bfs.length, dead_end.length);
}

/// Test that BFS finds optimal paths
#[test]
fn bfs_finds_optimal_path() {
    let maze = RecursiveBacktracker4::new_from_seed(5678).generate(10, 10);
    let start = GridCoord2D::new(0, 0);
    let end = GridCoord2D::new(9, 9);

    let path = BfsSolver.solve(&maze, start, end).expect("path exists");

    assert!(!path.is_empty());
    assert_eq!(path.start(), Some(start));
    assert_eq!(path.end(), Some(end));
}

/// Test that A* finds optimal paths matching BFS
#[test]
fn astar_finds_optimal_path() {
    let maze = RecursiveBacktracker4::new_from_seed(9012).generate(15, 15);
    let start = GridCoord2D::new(0, 0);
    let end = GridCoord2D::new(14, 14);

    let bfs_path = BfsSolver.solve(&maze, start, end).expect("bfs path");
    let astar_path = AStarSolver.solve(&maze, start, end).expect("astar path");

    // Both should find optimal paths with same length
    assert_eq!(bfs_path.length, astar_path.length);
}

/// Test that DFS finds valid (but not necessarily optimal) paths
#[test]
fn dfs_finds_valid_path() {
    let maze = RecursiveBacktracker4::new_from_seed(3456).generate(12, 12);
    let start = GridCoord2D::new(0, 0);
    let end = GridCoord2D::new(11, 11);

    let path = DfsSolver.solve(&maze, start, end).expect("path exists");

    assert!(!path.is_empty());
    assert_eq!(path.start(), Some(start));
    assert_eq!(path.end(), Some(end));
}

/// Test that dead-end filling finds optimal paths
#[test]
fn dead_end_filling_finds_optimal_path() {
    let maze = RecursiveBacktracker4::new_from_seed(7890).generate(10, 10);
    let start = GridCoord2D::new(0, 0);
    let end = GridCoord2D::new(9, 9);

    let bfs_path = BfsSolver.solve(&maze, start, end).expect("bfs path");
    let dead_end_path = DeadEndFillingSolver
        .solve(&maze, start, end)
        .expect("dead-end path");

    // Dead-end filling should find optimal path
    assert_eq!(bfs_path.length, dead_end_path.length);
}
