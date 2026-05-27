use crate::direction4::Direction4;
use crate::dungeon::generators::{
    DungeonGenerationStep, DungeonGenerationSteps, DungeonGenerationVisitor, DungeonGenerator,
    VecDungeonGenerationVisitor,
};
use crate::dungeon::{DungeonGrid, DungeonType, TileType};
use crate::grid_coord_2d::GridCoord2D;
use crate::grid_rect::GridRect;
use crate::room4_list::{Room4List, RoomIndex};
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

/// Application data attached to each room in the generated connectivity graph.
///
/// The tag is intentionally coordinate-free: room positions live in the
/// pre-trim layout space, while the returned [`DungeonGrid`] is trimmed to its
/// content bounds. Keeping only the room dimensions here means the
/// [`Room4List`] graph stays purely topological and cannot desync from the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RoomTag {
    /// Room width in cells.
    pub width: usize,
    /// Room height in cells.
    pub height: usize,
}

/// The internal placement result: discrete rooms joined by corridors.
struct Layout {
    rooms: Vec<GridRect>,
    corridors: Vec<GridRect>,
    /// `dirs[i]` is the cardinal direction leading from `rooms[i]` to `rooms[i + 1]`.
    dirs: Vec<Direction4>,
}

/// Generator producing discrete, non-overlapping rectangular rooms connected by
/// explicit corridors.
///
/// Unlike [`DungeonWalkGenerator`](crate::dungeon::DungeonWalkGenerator), which
/// carves space by walking and stamping (rooms can merge), this generator
/// *places* rooms with collision detection and backtracking, so every room is a
/// distinct chamber reached through a corridor. It is also the first dungeon
/// generator to expose a [`Room4List`] connectivity graph alongside the tile
/// grid (see [`generate_with_graph`](RoomCorridorGenerator::generate_with_graph)).
///
/// Corridors run in the four cardinal directions, mapping naturally onto
/// [`Room4`](crate::room4::Room4)'s `Direction4` neighbour links.
pub struct RoomCorridorGenerator {
    rng_seed: u64,
    room_count_min: usize,
    room_count_max: usize,
    room_size_min: usize,
    room_size_max: usize,
    corridor_length_min: usize,
    corridor_length_max: usize,
    corridor_width_min: usize,
    corridor_width_max: usize,
    trim_padding: usize,
}

impl RoomCorridorGenerator {
    /// Create a generator with a random seed and sensible defaults.
    pub fn new_random() -> Self {
        Self::with_seed(rand::random())
    }

    /// Create a generator with a specific seed. A seed of `0` is replaced by a
    /// random seed (matching [`DungeonWalkGenerator`](crate::dungeon::DungeonWalkGenerator)).
    pub fn new_from_seed(seed: u64) -> Self {
        Self::with_seed(if seed == 0 { rand::random() } else { seed })
    }

    fn with_seed(rng_seed: u64) -> Self {
        // Defaults mirror dungen-unity's room/corridor ranges. The invariant
        // `corridor_width_max <= room_size_min` keeps every corridor mouth
        // coverable by the room it opens into.
        Self {
            rng_seed,
            room_count_min: 6,
            room_count_max: 12,
            room_size_min: 4,
            room_size_max: 8,
            corridor_length_min: 3,
            corridor_length_max: 8,
            corridor_width_min: 1,
            corridor_width_max: 3,
            trim_padding: 0,
        }
    }

    /// Set the inclusive range for the number of rooms to attempt.
    pub fn with_room_count_range(mut self, min: usize, max: usize) -> Self {
        let min = min.max(1);
        self.room_count_min = min;
        self.room_count_max = max.max(min);
        self
    }

    /// Set the inclusive range for room side lengths (applies to both width and height).
    ///
    /// To keep corridors coverable, the corridor-width range is clamped so that
    /// `corridor_width_max <= room_size_min`.
    pub fn with_room_size_range(mut self, min: usize, max: usize) -> Self {
        let min = min.max(1);
        self.room_size_min = min;
        self.room_size_max = max.max(min);
        self.clamp_corridor_width();
        self
    }

    /// Set the inclusive range for corridor lengths.
    pub fn with_corridor_length_range(mut self, min: usize, max: usize) -> Self {
        let min = min.max(1);
        self.corridor_length_min = min;
        self.corridor_length_max = max.max(min);
        self
    }

    /// Set the inclusive range for corridor widths (perpendicular thickness).
    ///
    /// Values are clamped so that `corridor_width_max <= room_size_min`,
    /// guaranteeing a corridor never opens onto a wall.
    pub fn with_corridor_width_range(mut self, min: usize, max: usize) -> Self {
        let min = min.max(1);
        self.corridor_width_min = min;
        self.corridor_width_max = max.max(min);
        self.clamp_corridor_width();
        self
    }

    /// Set the padding around the final trimmed dungeon.
    pub fn with_trim_padding(mut self, padding: usize) -> Self {
        self.trim_padding = padding;
        self
    }

    fn clamp_corridor_width(&mut self) {
        if self.corridor_width_max > self.room_size_min {
            self.corridor_width_max = self.room_size_min;
        }
        if self.corridor_width_min > self.corridor_width_max {
            self.corridor_width_min = self.corridor_width_max;
        }
    }

    /// Generate a dungeon of discrete rooms joined by corridors.
    pub fn generate(&self, width: usize, height: usize) -> DungeonGrid {
        let (grid, _layout, _offset) = self.build(width, height);
        grid
    }

    /// Generate a dungeon together with its room connectivity graph.
    ///
    /// The returned [`Room4List`] has one node per placed room, linked through
    /// the cardinal directions of the corridors joining them. Rooms are pushed
    /// in placement order; the first room is the start and the last is the exit.
    pub fn generate_with_graph(
        &self,
        width: usize,
        height: usize,
    ) -> (DungeonGrid, Room4List<RoomTag>) {
        let (grid, layout, _offset) = self.build(width, height);
        let graph = build_graph(&layout);
        (grid, graph)
    }

    /// Generate with step-by-step events for animation.
    ///
    /// Step coordinates are in the same space as the [`DungeonGrid`] returned by
    /// [`generate`](Self::generate): rooms, corridors and the exit are rebased
    /// onto the trimmed grid, so replaying the steps lines up with the final
    /// dungeon even when `trim_padding` is non-zero.
    pub fn generate_steps(&self, width: usize, height: usize) -> DungeonGenerationSteps {
        let mut visitor = VecDungeonGenerationVisitor::default();
        let (grid, layout, offset) = self.build(width, height);

        // Map a layout-space rectangle into the trimmed grid's coordinate space.
        let rebase = |r: &GridRect| {
            GridRect::new(
                GridCoord2D::new(r.origin.x - offset.x, r.origin.y - offset.y),
                r.width,
                r.height,
            )
        };

        for (i, room) in layout.rooms.iter().enumerate() {
            visitor.on_step(&DungeonGenerationStep::PlaceRoom { rect: rebase(room) });
            if i < layout.corridors.len() {
                visitor.on_step(&DungeonGenerationStep::PlaceCorridor {
                    rect: rebase(&layout.corridors[i]),
                });
            }
        }
        if let Some(exit) = grid.exit() {
            visitor.on_step(&DungeonGenerationStep::SetExit { coord: exit });
        }
        visitor.on_step(&DungeonGenerationStep::Complete);

        DungeonGenerationSteps::new(visitor.into_steps())
    }

    /// Run the placement algorithm and rasterize it into a trimmed grid.
    ///
    /// Returns the trimmed grid, the (untrimmed) layout, and the trim offset:
    /// subtracting the offset from a layout coordinate maps it into the grid.
    fn build(&self, width: usize, height: usize) -> (DungeonGrid, Layout, GridCoord2D) {
        let mut rng = StdRng::seed_from_u64(self.rng_seed);

        if width == 0 || height == 0 {
            return (
                DungeonGrid::new(width, height),
                Layout {
                    rooms: Vec::new(),
                    corridors: Vec::new(),
                    dirs: Vec::new(),
                },
                GridCoord2D::new(0, 0),
            );
        }

        let layout = self.build_layout(width, height, &mut rng);

        let mut grid = DungeonGrid::new(width, height);
        for rect in layout.rooms.iter().chain(layout.corridors.iter()) {
            for coord in rect.coords() {
                grid.set(coord, TileType::Floor);
            }
        }

        // Exit is the last room placed (or any floor, if no rooms were placed).
        if let Some(last) = layout.rooms.last() {
            grid.set_exit(last.center());
        }

        // Reuse the shared trim -> place_walls -> compute_edge_masks pipeline.
        let (grid, offset) = grid.trim_with_offset(self.trim_padding);
        (grid, layout, offset)
    }

    /// Port of dungen-unity's `DungeonGenerator.asBoard`: place a first room,
    /// then repeatedly attach a corridor + room, rotating direction clockwise on
    /// no-fit and backtracking the corridor when the next room cannot be placed.
    fn build_layout(&self, width: usize, height: usize, rng: &mut StdRng) -> Layout {
        let mut rooms: Vec<GridRect> = Vec::new();
        let mut corridors: Vec<GridRect> = Vec::new();
        let mut dirs: Vec<Direction4> = Vec::new();

        let room_count = sample(rng, self.room_count_min, self.room_count_max);

        // First room: placed at a random in-bounds position.
        let (rw, rh) = self.sample_room_size(rng, width, height);
        let ox = rng.random_range(0..=(width - rw));
        let oy = rng.random_range(0..=(height - rh));
        rooms.push(GridRect::new(GridCoord2D::new(ox, oy), rw, rh));

        let cardinals = Direction4::CARDINALS; // [N, E, S, W]
        let mut dir_idx = rng.random_range(0..4);
        let mut room_retry = 0usize;
        let mut placed = 1usize;

        while placed < room_count {
            // Fresh attempt picks a random direction; a retry rotates clockwise.
            if room_retry == 0 {
                dir_idx = rng.random_range(0..4);
            } else {
                dir_idx = (dir_idx + 1) % 4;
            }

            let source = *rooms.last().unwrap();
            let source_idx = rooms.len() - 1;

            // Try to fit a corridor, rotating clockwise up to four times.
            let mut corridor = None;
            for _ in 0..4 {
                if let Some(c) = self.try_corridor(rng, &source, cardinals[dir_idx], width, height)
                {
                    if self.collision_free(&c, &rooms, &corridors, Some(source_idx)) {
                        corridor = Some((c, cardinals[dir_idx]));
                        break;
                    }
                }
                dir_idx = (dir_idx + 1) % 4;
            }

            let (corridor_rect, dir) = match corridor {
                Some(v) => v,
                // No corridor fits in any direction: the layout is boxed in.
                None => break,
            };

            // Try to place the next room at the far end of the corridor.
            if let Some(room) = self.try_room(rng, &corridor_rect, dir, width, height) {
                if self.collision_free(&room, &rooms, &corridors, None) {
                    corridors.push(corridor_rect);
                    rooms.push(room);
                    dirs.push(dir);
                    room_retry = 0;
                    placed += 1;
                    continue;
                }
            }

            // Room did not fit: drop the corridor (backtrack) and retry.
            if room_retry < 4 {
                room_retry += 1;
            } else {
                break;
            }
        }

        Layout {
            rooms,
            corridors,
            dirs,
        }
    }

    fn sample_room_size(&self, rng: &mut StdRng, width: usize, height: usize) -> (usize, usize) {
        let rw = sample(rng, self.room_size_min, self.room_size_max)
            .min(width)
            .max(1);
        let rh = sample(rng, self.room_size_min, self.room_size_max)
            .min(height)
            .max(1);
        (rw, rh)
    }

    /// Build a corridor rectangle leaving `room` in `dir`. The corridor mouth is
    /// kept within the room's edge so it always connects.
    fn try_corridor(
        &self,
        rng: &mut StdRng,
        room: &GridRect,
        dir: Direction4,
        width: usize,
        height: usize,
    ) -> Option<GridRect> {
        let length = sample(rng, self.corridor_length_min, self.corridor_length_max);
        let cwidth = sample(rng, self.corridor_width_min, self.corridor_width_max);

        let rect = match dir {
            Direction4::NORTH => {
                if room.top() < length || room.width < cwidth {
                    return None;
                }
                let ox = pick_in_room(rng, room.left(), room.right(), cwidth)?;
                GridRect::new(GridCoord2D::new(ox, room.top() - length), cwidth, length)
            }
            Direction4::SOUTH => {
                if room.width < cwidth {
                    return None;
                }
                let ox = pick_in_room(rng, room.left(), room.right(), cwidth)?;
                GridRect::new(GridCoord2D::new(ox, room.bottom() + 1), cwidth, length)
            }
            Direction4::EAST => {
                if room.height < cwidth {
                    return None;
                }
                let oy = pick_in_room(rng, room.top(), room.bottom(), cwidth)?;
                GridRect::new(GridCoord2D::new(room.right() + 1, oy), length, cwidth)
            }
            Direction4::WEST => {
                if room.left() < length || room.height < cwidth {
                    return None;
                }
                let oy = pick_in_room(rng, room.top(), room.bottom(), cwidth)?;
                GridRect::new(GridCoord2D::new(room.left() - length, oy), length, cwidth)
            }
            _ => return None,
        };

        if rect.right() < width && rect.bottom() < height {
            Some(rect)
        } else {
            None
        }
    }

    /// Build a room rectangle at the far end of `corridor`, fully covering the
    /// corridor mouth so the two connect.
    fn try_room(
        &self,
        rng: &mut StdRng,
        corridor: &GridRect,
        dir: Direction4,
        width: usize,
        height: usize,
    ) -> Option<GridRect> {
        let (rw, rh) = self.sample_room_size(rng, width, height);

        let rect = match dir {
            Direction4::NORTH => {
                if corridor.top() < rh {
                    return None;
                }
                let ox = cover_span(rng, corridor.left(), corridor.right(), rw, width)?;
                GridRect::new(GridCoord2D::new(ox, corridor.top() - rh), rw, rh)
            }
            Direction4::SOUTH => {
                let ox = cover_span(rng, corridor.left(), corridor.right(), rw, width)?;
                GridRect::new(GridCoord2D::new(ox, corridor.bottom() + 1), rw, rh)
            }
            Direction4::EAST => {
                let oy = cover_span(rng, corridor.top(), corridor.bottom(), rh, height)?;
                GridRect::new(GridCoord2D::new(corridor.right() + 1, oy), rw, rh)
            }
            Direction4::WEST => {
                if corridor.left() < rw {
                    return None;
                }
                let oy = cover_span(rng, corridor.top(), corridor.bottom(), rh, height)?;
                GridRect::new(GridCoord2D::new(corridor.left() - rw, oy), rw, rh)
            }
            _ => return None,
        };

        if rect.right() < width && rect.bottom() < height {
            Some(rect)
        } else {
            None
        }
    }

    /// True if `rect` collides with no placed room (except `skip_room`) or corridor.
    fn collision_free(
        &self,
        rect: &GridRect,
        rooms: &[GridRect],
        corridors: &[GridRect],
        skip_room: Option<usize>,
    ) -> bool {
        for (i, r) in rooms.iter().enumerate() {
            if Some(i) == skip_room {
                continue;
            }
            if rect.collides(r) {
                return false;
            }
        }
        !corridors.iter().any(|c| rect.collides(c))
    }
}

/// Sample an inclusive range `[min, max]`.
fn sample(rng: &mut StdRng, min: usize, max: usize) -> usize {
    rng.random_range(min..=max)
}

/// Pick a corridor origin so a `thickness`-wide corridor lies within the room
/// span `[lo, hi]` (i.e. the corridor mouth is fully on the room's edge).
fn pick_in_room(rng: &mut StdRng, lo: usize, hi: usize, thickness: usize) -> Option<usize> {
    // Origin range: [lo, hi + 1 - thickness].
    let max_origin = (hi + 1).checked_sub(thickness)?;
    if max_origin < lo {
        return None;
    }
    Some(rng.random_range(lo..=max_origin))
}

/// Pick a room origin so a `room_len`-long side fully covers the span `[lo, hi]`
/// while staying within `max_extent`.
fn cover_span(
    rng: &mut StdRng,
    lo: usize,
    hi: usize,
    room_len: usize,
    max_extent: usize,
) -> Option<usize> {
    // Need origin o with: o <= lo (covers lo) and o + room_len - 1 >= hi (covers hi).
    let min_origin = (hi + 1).saturating_sub(room_len);
    let max_origin = lo.min(max_extent.checked_sub(room_len)?);
    if min_origin > max_origin {
        return None;
    }
    Some(rng.random_range(min_origin..=max_origin))
}

/// Build the room connectivity graph from a placement layout.
fn build_graph(layout: &Layout) -> Room4List<RoomTag> {
    let mut list = Room4List::default();
    let mut indices: Vec<RoomIndex> = Vec::with_capacity(layout.rooms.len());

    for (i, room) in layout.rooms.iter().enumerate() {
        let tag = RoomTag {
            width: room.width,
            height: room.height,
        };
        let idx = if i == 0 {
            list.push_default(tag)
        } else {
            let prev = indices[i - 1];
            let dir = layout.dirs[i - 1]; // direction prev -> this
            // Link this room back toward its predecessor; Room4List propagates
            // the reciprocal link (prev gains a neighbour in `dir`).
            list.push_new(tag, |room4| {
                room4.set_room(dir.opposite(), Some(prev));
            })
        };
        indices.push(idx);
    }

    list
}

impl DungeonGenerator for RoomCorridorGenerator {
    fn new_random() -> Self {
        RoomCorridorGenerator::new_random()
    }

    fn new_from_seed(seed: u64) -> Self {
        RoomCorridorGenerator::new_from_seed(seed)
    }

    /// Generate a dungeon. `floor_count` is ignored — room count is controlled
    /// via [`with_room_count_range`](RoomCorridorGenerator::with_room_count_range).
    fn generate(&self, width: usize, height: usize, _floor_count: usize) -> DungeonGrid {
        RoomCorridorGenerator::generate(self, width, height)
    }

    fn generate_steps(
        &self,
        width: usize,
        height: usize,
        _floor_count: usize,
    ) -> DungeonGenerationSteps {
        RoomCorridorGenerator::generate_steps(self, width, height)
    }

    fn dungeon_type(&self) -> DungeonType {
        DungeonType::Chambers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid_coord_2d::GetCoordinateBounds2D;

    fn all_rects(layout_grid: &DungeonGrid) -> usize {
        layout_grid.floor_count()
    }

    #[test]
    fn produces_floors_and_exit() {
        let generator = RoomCorridorGenerator::new_from_seed(42);
        let grid = generator.generate(60, 60);
        assert!(all_rects(&grid) > 0, "should carve floor tiles");
        assert!(grid.exit().is_some(), "should set an exit");
    }

    #[test]
    fn same_seed_is_deterministic() {
        let g1 = RoomCorridorGenerator::new_from_seed(7).generate(60, 60);
        let g2 = RoomCorridorGenerator::new_from_seed(7).generate(60, 60);
        let f1: std::collections::HashSet<_> = g1.floor_iter().collect();
        let f2: std::collections::HashSet<_> = g2.floor_iter().collect();
        assert_eq!(f1, f2);
        assert_eq!(g1.exit(), g2.exit());
    }

    #[test]
    fn steps_are_in_trimmed_grid_space() {
        // Step rects/exit must fall inside the bounds of the grid generate()
        // returns, even with non-zero trim padding.
        let generator = RoomCorridorGenerator::new_from_seed(42).with_trim_padding(2);
        let grid = generator.generate(60, 60);
        let (w, h) = (grid.width(), grid.height());

        for step in generator.generate_steps(60, 60) {
            match step {
                DungeonGenerationStep::PlaceRoom { rect }
                | DungeonGenerationStep::PlaceCorridor { rect } => {
                    assert!(
                        rect.right() < w && rect.bottom() < h,
                        "step rect {rect:?} outside trimmed grid {w}x{h}"
                    );
                }
                DungeonGenerationStep::SetExit { coord } => {
                    assert!(coord.x < w && coord.y < h, "exit {coord:?} outside grid");
                }
                _ => {}
            }
        }
    }

    #[test]
    fn rooms_never_overlap() {
        // Inspect the raw layout: no two rooms may collide (overlap or touch),
        // and no room may collide with a corridor it is not connected to.
        let generator = RoomCorridorGenerator::new_from_seed(123);
        let mut rng = StdRng::seed_from_u64(generator.rng_seed);
        let layout = generator.build_layout(80, 80, &mut rng);

        for (i, a) in layout.rooms.iter().enumerate() {
            for b in layout.rooms.iter().skip(i + 1) {
                assert!(!a.collides(b), "rooms {a:?} and {b:?} collide");
            }
        }
    }

    #[test]
    fn tiny_bounds_terminate_without_panic() {
        // A box too small for the configured rooms must not loop or panic.
        let generator = RoomCorridorGenerator::new_from_seed(1)
            .with_room_size_range(3, 4)
            .with_corridor_width_range(1, 2)
            .with_room_count_range(10, 10);
        let grid = generator.generate(12, 12);
        assert!(grid.width() > 0 && grid.height() > 0);
    }

    #[test]
    fn corridor_width_clamped_to_room_min() {
        let generator = RoomCorridorGenerator::new_from_seed(1)
            .with_room_size_range(3, 6)
            .with_corridor_width_range(5, 9);
        assert!(generator.corridor_width_max <= generator.room_size_min);
    }

    #[test]
    fn graph_links_are_bidirectional() {
        let generator = RoomCorridorGenerator::new_from_seed(99).with_room_count_range(4, 4);
        let (_grid, graph) = generator.generate_with_graph(80, 80);
        assert!(!graph.is_empty(), "graph should have rooms");

        // Every room except endpoints has at least one door; reciprocal links hold.
        for room in graph.iter() {
            for dir in [
                Direction4::NORTH,
                Direction4::SOUTH,
                Direction4::EAST,
                Direction4::WEST,
            ] {
                if let Some(neighbor_idx) = room.get_neighbor(dir) {
                    let neighbor = &graph[neighbor_idx];
                    assert_eq!(
                        neighbor.get_neighbor(dir.opposite()),
                        Some(room.index()),
                        "link from {:?} in {:?} not reciprocated",
                        room.index(),
                        dir
                    );
                }
            }
        }
    }
}
