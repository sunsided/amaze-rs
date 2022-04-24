use crate::grid_coord_2d::{GetCoordinateBounds2D, GridCoord2D};
use crate::room4::{Door4, Wall4};
use crate::wall4_grid::Wall4Grid;

const N: Door4 = Door4::NORTH;
const NS: Door4 = Door4::NORTH.join(Door4::SOUTH);
const NE: Door4 = Door4::NORTH.join(Door4::EAST);
const NW: Door4 = Door4::NORTH.join(Door4::WEST);

const S: Door4 = Door4::SOUTH;
const SE: Door4 = Door4::SOUTH.join(Door4::EAST);
const SW: Door4 = Door4::SOUTH.join(Door4::WEST);

const E: Door4 = Door4::EAST;
const EW: Door4 = Door4::EAST.join(Door4::WEST);

const W: Door4 = Door4::WEST;

const NEW: Door4 = Door4::NORTH.join(Door4::EAST).join(Door4::WEST);
const NSW: Door4 = Door4::NORTH.join(Door4::SOUTH).join(Door4::WEST);
const NSE: Door4 = Door4::NORTH.join(Door4::SOUTH).join(Door4::EAST);
const SEW: Door4 = Door4::SOUTH.join(Door4::EAST).join(Door4::WEST);

pub fn render_paths_unicode(grid: &Wall4Grid) -> String {
    let mut output = String::default();
    for y in 0..grid.height() {
        for x in 0..grid.width() {
            let coord = GridCoord2D::new(x, y);
            let wall: Door4 = !grid[coord];
            let str = match wall {
                Door4::NONE => " ",
                Door4::ALL => "\u{256C}", // ╬
                N => "\u{2568}",          // ╨
                S => "\u{2565}",          // ╥
                E => "\u{255E}",          // ╞
                W => "\u{2561}",          // ╡
                NS => "\u{2551}",         // ║
                EW => "\u{2550}",         // ═
                NE => "\u{255A}",         // ╚
                NW => "\u{255D}",         // ╝
                SE => "\u{2554}",         // ╔
                SW => "\u{2557}",         // ╗
                NEW => "\u{2569}",        // ╩
                NSW => "\u{2563}",        // ╣
                NSE => "\u{2560}",        // ╠
                SEW => "\u{2566}",        // ╦
                _ => "\u{2573}",          // ╳
            };

            output.push_str(str);
        }
        output.push('\n');
    }

    output
}
