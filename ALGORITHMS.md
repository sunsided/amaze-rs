# Algorithms in `amaze`

This document summarizes generation and solving algorithms shipped in `amaze`.

## Generators

- `RecursiveBacktracker4`: depth-first style, long corridors.
- `GrowingTree4<S>`: configurable frontier selector (`NewestCell`, `OldestCell`, `RandomCell`, `MixedCell`).
- `Kruskal4`: randomized spanning tree generation with union-find.
- `Eller4`: row-by-row generation with `O(width)` memory.
- `Wilson4`: unbiased loop-erased random walk spanning tree.
- `HuntAndKill4`: walk until dead end, then hunt next viable seed.
- `Sidewinder4`: fast horizontal runs with periodic northward links.
- `BinaryTree4`: very fast directional-bias algorithm.

## Solvers

- `BfsSolver`: shortest path in unweighted grids.
- `DfsSolver`: finds any path quickly.
- `AStarSolver`: shortest path with Manhattan heuristic.
- `DeadEndFillingSolver`: prunes dead-ends and extracts the surviving route.

## Animation API

All generators expose `generate_steps()` which yields `GenerationStep` events:

- `Visit`
- `Carve`
- `Backtrack`
- `AddToFrontier`
- `Complete`

This can be consumed by GUIs and visualizers for progressive rendering.
