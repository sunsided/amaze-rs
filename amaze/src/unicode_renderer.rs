use crate::grid_coord_2d::{GetCoordinateBounds2D, GridCoord2D};
use crate::room4::Door4;
use crate::wall4_grid::Wall4Grid;

#[allow(dead_code)]
const UNICODE_SET_THIN: &'static [char] = &[
    ' ',        //   0b0000 - none
    '\u{2575}', // ╵ 0b0001 -    N
    '\u{2577}', // ╷ 0b0010 -   S
    '\u{2502}', // │ 0b0011 -   SN
    '\u{2576}', // ╶ 0b0100 -  E
    '\u{2514}', // └ 0b0101 -  E N
    '\u{250C}', // ┌ 0b0110 -  ES
    '\u{251C}', // ├ 0b0111 -  ESN
    '\u{2574}', // ╴ 0b1000 - W
    '\u{2518}', // ┘ 0b1001 - W  N
    '\u{2510}', // ┐ 0b1010 - W S
    '\u{2524}', // ┤ 0b1011 - W SN
    '\u{2500}', // ─ 0b1100 - WE
    '\u{2534}', // ┴ 0b1101 - WE N
    '\u{252C}', // ┬ 0b1110 - WES
    '\u{253C}', // ┼ 0b1111 - WESN
];

#[allow(dead_code)]
const UNICODE_SET_DOUBLE: &'static [char] = &[
    ' ',        //   0b0000 - none
    '\u{2568}', // ╨ 0b0001 -    N
    '\u{2565}', // ╥ 0b0010 -   S
    '\u{2551}', // ║ 0b0011 -   SN
    '\u{255E}', // ╞ 0b0100 -  E
    '\u{255A}', // ╚ 0b0101 -  E N
    '\u{2554}', // ╔ 0b0110 -  ES
    '\u{2560}', // ╠ 0b0111 -  ESN
    '\u{2561}', // ╡ 0b1000 - W
    '\u{255D}', // ╝ 0b1001 - W  N
    '\u{2557}', // ╗ 0b1010 - W S
    '\u{2563}', // ╣ 0b1011 - W SN
    '\u{2550}', // ═ 0b1100 - WE
    '\u{2569}', // ╩ 0b1101 - WE N
    '\u{2566}', // ╦ 0b1110 - WES
    '\u{256C}', // ╬ 0b1111 - WESN
];

#[allow(dead_code)]
const UNICODE_SET_HEAVY: &'static [char] = &[
    ' ',        //   0b0000 - none
    '\u{2579}', // ╹ 0b0001 -    N
    '\u{257B}', // ╻ 0b0010 -   S
    '\u{2503}', // ┃ 0b0011 -   SN
    '\u{257A}', // ╺ 0b0100 -  E
    '\u{2517}', // ┗ 0b0101 -  E N
    '\u{250F}', // ┏ 0b0110 -  ES
    '\u{2523}', // ┣ 0b0111 -  ESN
    '\u{2578}', // ╸ 0b1000 - W
    '\u{251B}', // ┛ 0b1001 - W  N
    '\u{2513}', // ┓ 0b1010 - W S
    '\u{252B}', // ┫ 0b1011 - W SN
    '\u{2501}', // ━ 0b1100 - WE
    '\u{253B}', // ┻ 0b1101 - WE N
    '\u{2533}', // ┳ 0b1110 - WES
    '\u{254B}', // ╋ 0b1111 - WESN
];

pub struct UnicodeRenderer {
    table: &'static [char],
}

impl UnicodeRenderer {
    pub fn new_thin() -> Self {
        Self {
            table: UNICODE_SET_THIN,
        }
    }

    pub fn new_double() -> Self {
        Self {
            table: UNICODE_SET_DOUBLE,
        }
    }

    pub fn new_heavy() -> Self {
        Self {
            table: UNICODE_SET_HEAVY,
        }
    }

    fn lookup(&self, doors: Door4) -> char {
        self.table[*doors as usize]
    }

    pub fn render(&self, grid: &Wall4Grid) -> String {
        let mut output = String::default();
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                let coord = GridCoord2D::new(x, y);
                let wall: Door4 = !grid[coord];
                output.push(self.lookup(wall));
            }
            output.push('\n');
        }

        output
    }
}
