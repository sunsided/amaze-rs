# Algorithms in `amaze`

This document summarizes generation and solving algorithms shipped in `amaze`.

## Maze Generators

- `RecursiveBacktracker4`: depth-first style, long corridors.
- `GrowingTree4<S>`: configurable frontier selector (`NewestCell`, `OldestCell`, `RandomCell`, `MixedCell`).
- `Kruskal4`: randomized spanning tree generation with union-find.
- `Eller4`: row-by-row generation with `O(width)` memory.
- `Wilson4`: unbiased loop-erased random walk spanning tree.
- `HuntAndKill4`: walk until dead end, then hunt next viable seed.
- `Sidewinder4`: fast horizontal runs with periodic northward links.
- `BinaryTree4`: very fast directional-bias algorithm.
- `Prim4`: randomized Prim's algorithm with many short corridors and branches.

## Dungeon Generators

`DungeonWalkGenerator` implements procedural dungeon generation using random walk algorithms, supporting three distinct types:

### Caverns
Random walk with unconstrained step direction and distance, creating organic, cave-like structures.
- Walk continues until target floor count is reached
- Exit placed at the last walker position

### Rooms
Random walk with room stamping at regular intervals:
- Walk lengths: 9-17 steps (Unity parity: `Random.Range(9, 18)` → 9..=17)
- Room half-sizes: 1-4 (Unity parity: `Random.Range(1, 5)` → 1..=4)
- Rooms are stamped as `(2w+1)x(2h+1)` rectangular areas
- Creates dungeons with distinct chambers connected by corridors

### Winding
Variant of Rooms with probabilistic room suppression:
- Uses winding probability (0-99) to control room frequency
- Roll value in 0..100, room created when `roll > winding_probability`
- Higher values (e.g., 80) create more hallway-dominant layouts
- Lower values (e.g., 20) create more room-heavy layouts
- Default: 50% winding probability

### Generation Features
- Deterministic seeded RNG for reproducible results
- Automatic wall placement around floor tiles
- Exit marker at last accepted floor position
- Edge mask computation for rendering optimization
- Animation step events for progressive visualization

## Solvers

### Maze Solvers
- `BfsSolver`: shortest path in unweighted grids.
- `DfsSolver`: finds any path quickly.
- `AStarSolver`: shortest path with Manhattan heuristic.
- `DeadEndFillingSolver`: prunes dead-ends and extracts the surviving route.

### Dungeon Solvers
Dungeons are converted to `PassabilityGrid` (1:1 mapping, no inflation) and solved using:
- `solve_bfs()`: Breadth-first search for shortest paths
- `solve_astar()`: A* search with Manhattan heuristic

Both solvers operate on the passability representation where floor tiles map to passable cells and walls are impassable.

## Animation API

### Maze Animation
All maze generators expose `generate_steps()` which yields `GenerationStep` events:

- `Visit`
- `Carve`
- `Backtrack`
- `AddToFrontier`
- `Complete`

### Dungeon Animation
`DungeonWalkGenerator.generate_steps()` yields `DungeonGenerationStep` events:

- `PlaceFloor { coord }` - Place a single floor tile
- `PlaceWall { coord }` - Place a wall tile during post-processing
- `StampRoom { center, half_width, half_height }` - Stamp a rectangular room
- `SetExit { coord }` - Mark the exit position
- `Complete` - Generation finished

These events can be consumed by GUIs and visualizers for progressive rendering.
