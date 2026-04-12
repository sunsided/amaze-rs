//! Integration tests for dungeon generation and solving.

#![cfg(all(feature = "representations", feature = "solvers"))]

use amaze::dungeon::{DungeonType, DungeonWalkGenerator, solve_astar, solve_bfs};
use amaze::preamble::*;
use std::collections::{HashSet, VecDeque};

/// Helper to check if all floor tiles are reachable from entrance.
fn assert_fully_connected(dungeon: &DungeonGrid) {
    if dungeon.floor_count() == 0 {
        return;
    }

    let passability = PassabilityGrid::from(dungeon);
    let entrance = passability.entrance_position();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    queue.push_back(entrance);
    visited.insert(entrance);

    while let Some((x, y)) = queue.pop_front() {
        // Check 4 neighbors
        for (dx, dy) in [(0, -1), (1, 0), (0, 1), (-1, 0)] {
            let nx = (x as isize + dx) as usize;
            let ny = (y as isize + dy) as usize;

            if nx < passability.width()
                && ny < passability.height()
                && passability.is_passable(nx, ny)
                && !visited.contains(&(nx, ny))
            {
                visited.insert((nx, ny));
                queue.push_back((nx, ny));
            }
        }
    }

    // All floor tiles should be reachable
    assert_eq!(
        visited.len(),
        dungeon.floor_count(),
        "Not all floor tiles are reachable from entrance"
    );
}

#[test]
fn test_caverns_connectivity() {
    let generator = DungeonWalkGenerator::new_from_seed(DungeonType::Caverns, 42);
    let dungeon = generator.generate(30, 30, 100);

    assert_fully_connected(&dungeon);
    assert!(dungeon.exit().is_some());
}

#[test]
fn test_rooms_connectivity() {
    let generator = DungeonWalkGenerator::new_from_seed(DungeonType::Rooms, 42);
    let dungeon = generator.generate(30, 30, 150);

    assert_fully_connected(&dungeon);
    assert!(dungeon.exit().is_some());
}

#[test]
fn test_winding_connectivity() {
    let generator =
        DungeonWalkGenerator::new_from_seed(DungeonType::Winding, 42).with_winding_probability(70);
    let dungeon = generator.generate(30, 30, 120);

    assert_fully_connected(&dungeon);
    assert!(dungeon.exit().is_some());
}

#[test]
fn test_bfs_finds_path_to_exit() {
    let generator = DungeonWalkGenerator::new_from_seed(DungeonType::Caverns, 123);
    let dungeon = generator.generate(25, 25, 100);

    let passability = PassabilityGrid::from(&dungeon);
    let (ex, ey) = passability.entrance_position();
    let (xx, xy) = passability.exit_position();

    let start = GridCoord2D::new(ex, ey);
    let end = GridCoord2D::new(xx, xy);

    let path = solve_bfs(&passability, start, end);

    assert!(path.is_some(), "BFS should find a path to the exit");
    let path = path.unwrap();
    assert!(path.length > 0, "Path should have at least one step");
}

#[test]
fn test_astar_finds_path_to_exit() {
    let generator = DungeonWalkGenerator::new_from_seed(DungeonType::Rooms, 456);
    let dungeon = generator.generate(25, 25, 150);

    let passability = PassabilityGrid::from(&dungeon);
    let (ex, ey) = passability.entrance_position();
    let (xx, xy) = passability.exit_position();

    let start = GridCoord2D::new(ex, ey);
    let end = GridCoord2D::new(xx, xy);

    let path = solve_astar(&passability, start, end);

    assert!(path.is_some(), "A* should find a path to the exit");
    let path = path.unwrap();
    assert!(path.length > 0, "Path should have at least one step");
}

#[test]
fn test_bfs_astar_parity() {
    // BFS and A* should find paths of the same length (shortest path).
    let generator = DungeonWalkGenerator::new_from_seed(DungeonType::Caverns, 789);
    let dungeon = generator.generate(20, 20, 80);

    let passability = PassabilityGrid::from(&dungeon);
    let (ex, ey) = passability.entrance_position();
    let (xx, xy) = passability.exit_position();
    let start = GridCoord2D::new(ex, ey);
    let end = GridCoord2D::new(xx, xy);

    let bfs_path = solve_bfs(&passability, start, end);
    let astar_path = solve_astar(&passability, start, end);

    assert!(bfs_path.is_some());
    assert!(astar_path.is_some());

    let bfs_len = bfs_path.unwrap().length;
    let astar_len = astar_path.unwrap().length;

    assert_eq!(
        bfs_len, astar_len,
        "BFS and A* should find paths of equal length"
    );
}

#[test]
fn test_multiple_seeds_produce_different_dungeons() {
    let gen1 = DungeonWalkGenerator::new_from_seed(DungeonType::Caverns, 111);
    let gen2 = DungeonWalkGenerator::new_from_seed(DungeonType::Caverns, 222);

    let d1 = gen1.generate(20, 20, 100);
    let d2 = gen2.generate(20, 20, 100);

    let floors1: HashSet<_> = d1.floor_iter().collect();
    let floors2: HashSet<_> = d2.floor_iter().collect();

    // Probability of collision is extremely low
    assert_ne!(
        floors1, floors2,
        "Different seeds should produce different dungeons"
    );
}

#[test]
fn test_same_seed_produces_identical_dungeons() {
    let gen1 = DungeonWalkGenerator::new_from_seed(DungeonType::Rooms, 999);
    let gen2 = DungeonWalkGenerator::new_from_seed(DungeonType::Rooms, 999);

    let d1 = gen1.generate(25, 25, 120);
    let d2 = gen2.generate(25, 25, 120);

    let floors1: HashSet<_> = d1.floor_iter().collect();
    let floors2: HashSet<_> = d2.floor_iter().collect();

    assert_eq!(
        floors1, floors2,
        "Same seed should produce identical dungeons"
    );
    assert_eq!(d1.exit(), d2.exit(), "Exit positions should match");
}

#[test]
fn test_dungeon_has_walls() {
    let generator = DungeonWalkGenerator::new_from_seed(DungeonType::Caverns, 42);
    let dungeon = generator.generate(15, 15, 50);

    // Check that there are walls in the dungeon (placed around floor tiles)
    let mut wall_count = 0;
    for y in 0..dungeon.height() {
        for x in 0..dungeon.width() {
            if dungeon.get(GridCoord2D::new(x, y)).unwrap().is_wall() {
                wall_count += 1;
            }
        }
    }

    assert!(wall_count > 0, "Dungeon should have walls");
}

#[test]
fn test_passability_conversion_preserves_floor_count() {
    let generator = DungeonWalkGenerator::new_from_seed(DungeonType::Winding, 333);
    let dungeon = generator.generate(20, 20, 90);

    let passability = PassabilityGrid::from(&dungeon);

    let passable_count = passability
        .cells
        .iter()
        .filter(|&&is_passable| is_passable)
        .count();

    assert_eq!(
        passable_count,
        dungeon.floor_count(),
        "PassabilityGrid should have same number of passable cells as floor tiles"
    );
}

#[test]
fn test_edge_masks_computed() {
    let generator = DungeonWalkGenerator::new_from_seed(DungeonType::Rooms, 555);
    let dungeon = generator.generate(15, 15, 80);

    // Check that edge masks are non-zero for at least some walls
    let mut has_edge_mask = false;
    for y in 0..dungeon.height() {
        for x in 0..dungeon.width() {
            let coord = GridCoord2D::new(x, y);
            if dungeon.get(coord).unwrap().is_wall() && dungeon.edge_mask(coord) != 0 {
                has_edge_mask = true;
                break;
            }
        }
        if has_edge_mask {
            break;
        }
    }

    assert!(has_edge_mask, "Some walls should have edge masks computed");
}
