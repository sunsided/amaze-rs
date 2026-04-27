# AMAZING.md — Amaze-rs Improvement Tasks

Ordered improvement tasks for the amaze-rs maze & dungeon generator project.

## Current State

- **Maze generators (9):** RecursiveBacktracker, GrowingTree, Kruskal, Eller, Wilson, HuntAndKill, Sidewinder, BinaryTree, Prim
- **Solvers (4):** BFS, DFS, A*, Dead-End Filling
- **Dungeon types (3):** Caverns, Rooms, Winding
- **Renderers:** Unicode (thin/double/heavy/hex), PPM, PBM
- **Extras:** Animation API, stats, representations, petgraph integration

---

## 1. Prim's Algorithm (`Prim4`) ✅

Randomized Prim's algorithm for maze generation. Produces mazes with short corridors and many branches — one of the classic "big 4" algorithms not yet implemented.

**Tasks:**
- [x] Create `amaze/src/generators/prim4.rs`
- [x] Implement `MazeGenerator2D` trait with `new_random()`, `new_from_seed()`, `generate()`, `generate_steps()`
- [x] Use a frontier `Vec<(GridCoord2D, GridCoord2D)>` (cell, parent) with random selection
- [x] Emit `GenerationStep::Visit`, `Carve`, `AddToFrontier`, `Complete` events
- [x] Add feature flag `generator-prim` in `Cargo.toml`
- [x] Register in `amaze/src/generators.rs` module
- [x] Add CLI subcommand option `--algorithm prim`
- [x] Add GUI `AlgorithmChoice::Prim` variant
- [x] Add unit test verifying perfect maze properties (connected, no cycles, N-1 passages)

## 2. `open_neighbors()` Iterator Optimization ✅

Change `Wall4Grid::open_neighbors()` from returning `Vec<GridCoord2D>` to `impl Iterator<Item = GridCoord2D>`. Eliminates allocation in hot paths.

**Tasks:**
- [x] Use array-based iterator pattern (like existing `neighbors()`)
- [x] Update `Wall4Grid::open_neighbors()` signature
- [x] Update all callers: solvers, stats, representations, tests
- [x] Remove unnecessary `.into_iter()` calls flagged by clippy
- [x] Run tests to confirm correctness

## 3. Expand Benchmarks ✅

Benchmark all generators, solvers, and multiple sizes.

**Tasks:**
- [x] Rewrite `amaze/benches/generation.rs` with parameterized benches
- [x] All 9 generators × sizes: 16x16, 64x64, 256x256
- [x] Add `amaze/benches/solvers.rs` for BFS, DFS, A*, DeadEndFilling
- [x] Add `amaze/benches/stats.rs` for `MazeStats::from_grid()`
- [x] Use Criterion groups: `generators`, `solvers`, `stats`
- [x] Make `MazeStats::from_grid()` and `stats` module public
- [x] Fix clippy warnings (useless `.into_iter()` calls)

## 4. Hexagonal Maze Support

Full hexagonal (6-connected) maze generation.

**Tasks:**
- [ ] 4.1 Create `Direction6` type (6-direction bitmask, mirrors `Direction4`)
- [ ] 4.2 Create `Wall6Grid` (hex grid with offset coordinates)
- [ ] 4.3 Create `RecursiveBacktracker6` and `GrowingTree6` generators
- [ ] 4.4 Create hex renderer (unicode box-drawing / geometric characters)
- [ ] 4.5 Add CLI and GUI integration, feature flags, tests

## 5. Maze Serialization (`serde`)

Complete `serde` support for all core types.

**Tasks:**
- [ ] Add `#[cfg_attr(feature = "serde", derive(...))]` to `Wall4Grid`, `DungeonGrid`, `Path`, `VisitMap2D`
- [ ] Add `amaze/src/serialization.rs` with JSON/binary save/load helpers
- [ ] Add CLI `save` / `load` subcommands
- [ ] Add round-trip serialization tests

## 6. Solver Animation (GUI)

Animate solver exploration visually — show visited cells, open set, and final path.

**Tasks:**
- [ ] Create `SolverVisitor` trait and `solve_with_visitor()` methods
- [ ] Emit events: `Discover`, `Visit`, `Backtrack`, `PathFound`, `NoPath`
- [ ] Add "Animate Solver" button to GUI
- [ ] Render visited cells in different color, animate path reveal

## 7. Aldous-Broder Algorithm (`AldousBroder4`)

Unbiased loop-erased random walk spanning tree.

**Tasks:**
- [ ] Create `amaze/src/generators/aldous_broder4.rs`
- [ ] Implement `MazeGenerator2D` trait
- [ ] Add feature flag, CLI option, GUI variant
- [ ] Add statistical test for unbiased distribution

## 8. Stats Sampling Mode

Optimize `average_shortest_path_length()` for large mazes.

**Tasks:**
- [ ] Create `MazeStatsOptions` with `sample_count` and `use_sampling_threshold`
- [ ] Add `MazeStats::from_grid_with_options()`
- [ ] Auto-switch to sampling when n > 2500 cells
- [ ] Add `MazeStats::longest_path_endpoints()`
- [ ] Benchmark old vs new approach

## 9. Property-Based Testing

Add `proptest` for generator correctness invariants.

**Tasks:**
- [ ] Add `proptest` to dev-dependencies
- [ ] Create `amaze/tests/property_tests.rs`
- [ ] Properties: connectivity, perfect maze (N-1 passages), no cycles, boundary
- [ ] Test multiple sizes with random seeds
- [ ] Add dungeon property tests

## 10. GUI Statistics Display

Show maze stats in the GUI sidebar.

**Tasks:**
- [ ] Add "Statistics" section in GUI left panel
- [ ] Display: dead ends, corridors, junctions, longest path, average path length
- [ ] Add "Compute Stats" button
- [ ] Show computation time
- [ ] Dungeon stats: floor count, wall count, exit position
