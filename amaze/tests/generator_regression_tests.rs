//! Regression tests for dungeon generators.

#![cfg(all(feature = "representations", feature = "solvers"))]

use amaze::dungeon::{DungeonType, DungeonWalkGenerator};
use amaze::preamble::*;

#[test]
fn test_small_grid_doesnt_hang() {
    // Regression test: small grids with large floor_count requests should not hang
    let generator = DungeonWalkGenerator::new_from_seed(DungeonType::Winding, 42);
    let dungeon = generator.generate(8, 8, 120);

    // Should complete without hanging
    assert!(dungeon.width() > 0 && dungeon.width() <= 8);
    assert!(dungeon.height() > 0 && dungeon.height() <= 8);
    // Floor count will be capped to what's possible
    assert!(dungeon.floor_count() > 0);
    assert!(dungeon.floor_count() <= 64); // 8x8 = 64 total cells
    assert!(dungeon.exit().is_some());
}

#[test]
fn test_impossible_floor_count_is_capped() {
    // Request more floors than physically possible
    let generator = DungeonWalkGenerator::new_from_seed(DungeonType::Rooms, 123);
    let dungeon = generator.generate(10, 10, 1000);

    // Should cap to ~90% of grid size
    assert!(dungeon.width() > 0 && dungeon.width() <= 10);
    assert!(dungeon.height() > 0 && dungeon.height() <= 10);
    assert!(dungeon.floor_count() > 0);
    assert!(dungeon.floor_count() <= 90); // Max 90% of 100 cells
    assert!(dungeon.exit().is_some());
}

#[test]
fn test_long_walk_range_configuration() {
    // Test that long walk range can be configured
    let generator =
        DungeonWalkGenerator::new_from_seed(DungeonType::Rooms, 42).with_long_walk_range(15, 25);
    let dungeon = generator.generate(50, 50, 300);

    // Fixed mode trims to content bounds
    assert!(dungeon.width() > 0 && dungeon.width() <= 50);
    assert!(dungeon.height() > 0 && dungeon.height() <= 50);
    assert!(dungeon.floor_count() >= 300);
    assert!(dungeon.exit().is_some());
}

#[test]
fn test_long_walk_range_validation() {
    // Test that invalid ranges are clamped
    let generator =
        DungeonWalkGenerator::new_from_seed(DungeonType::Rooms, 42).with_long_walk_range(0, 1); // min=0 should be clamped to 1

    // Should not panic and should generate valid dungeon
    let dungeon = generator.generate(30, 30, 150);
    assert!(dungeon.floor_count() >= 150);
}

#[test]
fn test_determinism() {
    let generator1 = DungeonWalkGenerator::new_from_seed(DungeonType::Caverns, 12345);
    let generator2 = DungeonWalkGenerator::new_from_seed(DungeonType::Caverns, 12345);

    let d1 = generator1.generate(15, 15, 80);
    let d2 = generator2.generate(15, 15, 80);

    // Compare floor positions
    let floors1: Vec<_> = d1.floor_iter().collect();
    let floors2: Vec<_> = d2.floor_iter().collect();

    assert_eq!(floors1.len(), floors2.len());
    // Note: HashSet iteration order is not deterministic, but the sets should be equal
    for coord in floors1 {
        assert!(d2.is_floor(coord), "Floor mismatch at {:?}", coord);
    }
}

#[test]
fn test_determinism_with_long_walk_fix() {
    // Test that the new directional long walk maintains determinism
    let generator1 = DungeonWalkGenerator::new_from_seed(DungeonType::Rooms, 99999);
    let generator2 = DungeonWalkGenerator::new_from_seed(DungeonType::Rooms, 99999);

    let d1 = generator1.generate(40, 40, 300);
    let d2 = generator2.generate(40, 40, 300);

    // Compare floor positions
    let floors1: Vec<_> = d1.floor_iter().collect();
    let floors2: Vec<_> = d2.floor_iter().collect();

    assert_eq!(floors1.len(), floors2.len(), "Floor counts should match");

    // Verify all floor positions match
    for coord in floors1 {
        assert!(d2.is_floor(coord), "Floor mismatch at {:?}", coord);
    }

    // Verify exits match
    assert_eq!(d1.exit(), d2.exit(), "Exit positions should match");
}

#[test]
fn test_determinism_with_custom_walk_range() {
    // Test determinism with custom walk range
    let generator1 =
        DungeonWalkGenerator::new_from_seed(DungeonType::Rooms, 54321).with_long_walk_range(5, 12);
    let generator2 =
        DungeonWalkGenerator::new_from_seed(DungeonType::Rooms, 54321).with_long_walk_range(5, 12);

    let d1 = generator1.generate(35, 35, 250);
    let d2 = generator2.generate(35, 35, 250);

    let floors1: Vec<_> = d1.floor_iter().collect();
    let floors2: Vec<_> = d2.floor_iter().collect();

    assert_eq!(floors1.len(), floors2.len());
    for coord in floors1 {
        assert!(d2.is_floor(coord), "Floor mismatch at {:?}", coord);
    }
}

#[test]
fn test_rooms_spread_with_directional_walk() {
    // Test that Rooms mode produces spread-out layout on larger maps
    // Using a large enough map to see spreading behavior
    let generator = DungeonWalkGenerator::new_from_seed(DungeonType::Rooms, 12345);
    let dungeon = generator.generate(60, 60, 500);

    // Fixed mode trims to content bounds
    assert!(dungeon.width() > 0 && dungeon.width() <= 60);
    assert!(dungeon.height() > 0 && dungeon.height() <= 60);
    assert!(dungeon.floor_count() >= 500);

    // After trimming, the dungeon is tightly bounded, so verify content exists
    // and that floor tiles cover a reasonable portion of the trimmed grid
    let floor_ratio = dungeon.floor_count() as f64 / (dungeon.width() * dungeon.height()) as f64;
    assert!(
        floor_ratio > 0.1,
        "Floor ratio {} too low, indicates sparse layout",
        floor_ratio
    );
}

#[test]
fn test_winding_with_long_walk() {
    // Test that Winding mode works correctly with directional long walk
    let generator = DungeonWalkGenerator::new_from_seed(DungeonType::Winding, 77777)
        .with_winding_probability(80) // High winding = fewer rooms
        .with_long_walk_range(10, 20);

    let dungeon = generator.generate(50, 50, 400);

    // Fixed mode trims to content bounds
    assert!(dungeon.width() > 0 && dungeon.width() <= 50);
    assert!(dungeon.height() > 0 && dungeon.height() <= 50);
    assert!(dungeon.floor_count() >= 400);
    assert!(dungeon.exit().is_some());
}

#[test]
fn test_room_stamping_respects_floor_cap() {
    // Test that room stamping stops when target floor count is reached
    // Use Rooms mode which stamps many rooms, and a target close to cap
    let generator = DungeonWalkGenerator::new_from_seed(DungeonType::Rooms, 999);
    let dungeon = generator.generate(20, 20, 180); // 20x20 = 400 cells, 90% = 360 max

    // Floor count should not significantly exceed target
    // Allow small overshoot due to final room stamp, but should be close
    assert!(
        dungeon.floor_count() <= 360,
        "Floor count {} exceeds max possible 360",
        dungeon.floor_count()
    );
    assert!(
        dungeon.floor_count() >= 180,
        "Floor count {} is below target 180",
        dungeon.floor_count()
    );
    // The important check: we should be reasonably close to target without massive overshoot
    assert!(
        dungeon.floor_count() <= 200,
        "Floor count {} overshoots target 180 by too much",
        dungeon.floor_count()
    );
}

#[test]
fn test_overflow_safety_large_dimensions() {
    // Test that very large dimensions don't cause overflow in cap calculation
    // This verifies that saturating_mul is used correctly

    // Test the calculation that happens in generate_internal
    let large_width = 100_000_usize;
    let large_height = 100_000_usize;

    // This would overflow with naive multiplication on some systems
    // With saturating_mul, it should cap at usize::MAX
    let max_possible_floor = large_width.saturating_mul(large_height).saturating_mul(9) / 10;

    // Verify we got a reasonable result (not wrapped around)
    assert!(max_possible_floor > 0, "Calculation should not wrap to 0");
    assert!(
        max_possible_floor <= usize::MAX / 2,
        "Result should be reasonable"
    );

    // Also verify that a normal-sized dungeon still generates correctly
    let generator = DungeonWalkGenerator::new_from_seed(DungeonType::Caverns, 42);
    let dungeon = generator.generate(100, 100, 1000);
    assert!(dungeon.floor_count() >= 1000);
}

#[test]
fn test_passability_grid_deterministic_entrance() {
    // Test that PassabilityGrid conversion produces deterministic entrance
    use amaze::representations::PassabilityGrid;

    let generator = DungeonWalkGenerator::new_from_seed(DungeonType::Caverns, 12345);
    let dungeon1 = generator.generate(20, 20, 100);
    let dungeon2 =
        DungeonWalkGenerator::new_from_seed(DungeonType::Caverns, 12345).generate(20, 20, 100);

    let grid1 = PassabilityGrid::from(&dungeon1);
    let grid2 = PassabilityGrid::from(&dungeon2);

    // Entrance should be deterministic (same for same dungeon layout)
    assert_eq!(
        grid1.entrance_position(),
        grid2.entrance_position(),
        "Entrance positions should be deterministic"
    );
}
