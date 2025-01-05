use crate::grid_coord_2d::{GetCoordinateBounds2D, GridCoord2D};
use crate::room4::Door4;
use crate::wall4_grid::Wall4Grid;

/// See [UnicodeRenderStyle::Thin] for a usage example.
#[allow(dead_code)]
const UNICODE_SET_THIN: &[char] = &[
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

/// See [UnicodeRenderStyle::Double] for a usage example.
#[allow(dead_code)]
const UNICODE_SET_DOUBLE: &[char] = &[
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

/// See [UnicodeRenderStyle::Heavy] for a usage example.
#[allow(dead_code)]
const UNICODE_SET_HEAVY: &[char] = &[
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

/// See [UnicodeRenderStyle::Hexadecimal] for a usage example.
#[allow(dead_code)]
const ASCII_SET_HEX: &[char] = &[
    '0', //   0b0000 - none
    '1', // ╨ 0b0001 -    N
    '2', // ╥ 0b0010 -   S
    '3', // ║ 0b0011 -   SN
    '4', // ╞ 0b0100 -  E
    '5', // ╚ 0b0101 -  E N
    '6', // ╔ 0b0110 -  ES
    '7', // ╠ 0b0111 -  ESN
    '8', // ╡ 0b1000 - W
    '9', // ╝ 0b1001 - W  N
    'A', // ╗ 0b1010 - W S
    'B', // ╣ 0b1011 - W SN
    'C', // ═ 0b1100 - WE
    'D', // ╩ 0b1101 - WE N
    'E', // ╦ 0b1110 - WES
    'F', // ╬ 0b1111 - WESN
];

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UnicodeRenderStyle {
    /// Renders the grid as thin unicode lines.
    ///
    /// ## Example
    /// ```
    /// use amaze::renderers::{UnicodeRenderer, UnicodeRenderStyle};
    ///
    /// let gen = amaze::generators::RecursiveBacktracker4::new_from_seed(0xdeadbeef);
    /// let grid = gen.generate(6, 6);
    ///
    /// let renderer = UnicodeRenderer::new(UnicodeRenderStyle::Thin, true);
    /// assert_eq!(renderer.render(&grid), indoc::indoc!(
    ///     "
    ///     ╷╶┬──┐
    ///     └┐│┌┐╵
    ///     ╷│├┘└┐
    ///     ├┘└─╴│
    ///     └─┐┌─┤
    ///     ╶─┴┘╶┘
    ///     "
    ///     ));
    /// ```
    Thin,
    /// Renders the grid as double unicode lines.
    ///
    /// ## Example
    /// ```
    /// use amaze::renderers::{UnicodeRenderer, UnicodeRenderStyle};
    ///
    /// let gen = amaze::generators::RecursiveBacktracker4::new_from_seed(0xdeadbeef);
    /// let grid = gen.generate(6, 6);
    ///
    /// let renderer = UnicodeRenderer::new(UnicodeRenderStyle::Double, true);
    /// assert_eq!(renderer.render(&grid), indoc::indoc!(
    ///     "
    ///     ╥╞╦══╗
    ///     ╚╗║╔╗╨
    ///     ╥║╠╝╚╗
    ///     ╠╝╚═╡║
    ///     ╚═╗╔═╣
    ///     ╞═╩╝╞╝
    ///     "
    ///     ));
    /// ```
    Double,
    /// Renders the grid as bold unicode lines.
    ///
    /// ## Example
    /// ```
    /// use amaze::renderers::{UnicodeRenderer, UnicodeRenderStyle};
    ///
    /// let gen = amaze::generators::RecursiveBacktracker4::new_from_seed(0xdeadbeef);
    /// let grid = gen.generate(6, 6);
    ///
    /// let renderer = UnicodeRenderer::new(UnicodeRenderStyle::Heavy, true);
    /// assert_eq!(renderer.render(&grid), indoc::indoc!(
    ///     "
    ///     ╻╺┳━━┓
    ///     ┗┓┃┏┓╹
    ///     ╻┃┣┛┗┓
    ///     ┣┛┗━╸┃
    ///     ┗━┓┏━┫
    ///     ╺━┻┛╺┛
    ///     "
    ///     ));
    /// ```
    Heavy,
    /// Renders the grid as hexadecimal values encoding the directions.
    ///
    /// ## Example
    /// ```
    /// use amaze::renderers::{UnicodeRenderer, UnicodeRenderStyle};
    ///
    /// let gen = amaze::generators::RecursiveBacktracker4::new_from_seed(0xdeadbeef);
    /// let grid = gen.generate(6, 6);
    ///
    /// // With line breaks:
    /// let renderer = UnicodeRenderer::new(UnicodeRenderStyle::Hexadecimal, true);
    /// assert_eq!(renderer.render(&grid), indoc::indoc!(
    ///     "
    ///     24ECCA
    ///     5A36A1
    ///     23795A
    ///     795C83
    ///     5CA6CB
    ///     4CD949
    ///     "
    ///     ));
    /// ```
    Hexadecimal,
}

#[derive(Debug, Clone)]
pub struct UnicodeRenderer {
    table: &'static [char],
    line_breaks: bool,
}

impl UnicodeRenderer {
    /// Creates a new renderer with the given style and optional line breaks.
    ///
    /// ## Example
    /// ```
    /// use amaze::renderers::{UnicodeRenderer, UnicodeRenderStyle};
    ///
    /// let gen = amaze::generators::RecursiveBacktracker4::new_from_seed(0xdeadbeef);
    /// let grid = gen.generate(6, 6);
    ///
    /// let renderer = UnicodeRenderer::new(UnicodeRenderStyle::Heavy, true);
    /// assert_eq!(renderer.render(&grid), indoc::indoc!(
    ///     "
    ///     ╻╺┳━━┓
    ///     ┗┓┃┏┓╹
    ///     ╻┃┣┛┗┓
    ///     ┣┛┗━╸┃
    ///     ┗━┓┏━┫
    ///     ╺━┻┛╺┛
    ///     "
    ///     ));
    ///
    /// // Without line breaks:
    /// let renderer = UnicodeRenderer::new(UnicodeRenderStyle::Heavy, false);
    /// assert_eq!(renderer.render(&grid), "╻╺┳━━┓┗┓┃┏┓╹╻┃┣┛┗┓┣┛┗━╸┃┗━┓┏━┫╺━┻┛╺┛");
    /// ```
    pub fn new(style: UnicodeRenderStyle, line_breaks: bool) -> Self {
        Self {
            table: match style {
                UnicodeRenderStyle::Thin => UNICODE_SET_THIN,
                UnicodeRenderStyle::Double => UNICODE_SET_DOUBLE,
                UnicodeRenderStyle::Heavy => UNICODE_SET_HEAVY,
                UnicodeRenderStyle::Hexadecimal => ASCII_SET_HEX,
            },
            line_breaks,
        }
    }

    fn lookup(&self, doors: Door4) -> char {
        self.table[*doors as usize]
    }

    /// Renders the specified grid.
    ///
    /// ## Example
    /// See [Self::new].
    pub fn render(&self, grid: &Wall4Grid) -> String {
        let mut output = String::default();
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                let coord = GridCoord2D::new(x, y);
                let wall: Door4 = !grid[coord];
                output.push(self.lookup(wall));
            }
            if self.line_breaks {
                output.push('\n');
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::RecursiveBacktracker4;
    use indoc::indoc;

    #[test]
    fn heavy_works() {
        let gen = RecursiveBacktracker4::new_from_seed(0xdeadbeef);
        let grid = gen.generate(6, 6);

        let renderer = UnicodeRenderer::new(UnicodeRenderStyle::Heavy, true);
        let str = renderer.render(&grid);
        let expected = indoc!(
            "
            ╻╺┳━━┓
            ┗┓┃┏┓╹
            ╻┃┣┛┗┓
            ┣┛┗━╸┃
            ┗━┓┏━┫
            ╺━┻┛╺┛
            "
        );
        assert_eq!(str, expected);
    }

    #[test]
    fn double_works() {
        let gen = RecursiveBacktracker4::new_from_seed(0xdeadbeef);
        let grid = gen.generate(6, 6);

        let renderer = UnicodeRenderer::new(UnicodeRenderStyle::Double, true);
        let str = renderer.render(&grid);
        let expected = indoc!(
            "
            ╥╞╦══╗
            ╚╗║╔╗╨
            ╥║╠╝╚╗
            ╠╝╚═╡║
            ╚═╗╔═╣
            ╞═╩╝╞╝
            "
        );
        assert_eq!(str, expected);
    }

    #[test]
    fn thin_works() {
        let gen = RecursiveBacktracker4::new_from_seed(0xdeadbeef);
        let grid = gen.generate(6, 6);

        let renderer = UnicodeRenderer::new(UnicodeRenderStyle::Thin, true);
        let str = renderer.render(&grid);
        let expected = indoc!(
            "
            ╷╶┬──┐
            └┐│┌┐╵
            ╷│├┘└┐
            ├┘└─╴│
            └─┐┌─┤
            ╶─┴┘╶┘
            "
        );
        assert_eq!(str, expected);
    }

    #[test]
    fn hex_works() {
        let gen = RecursiveBacktracker4::new_from_seed(0xdeadbeef);
        let grid = gen.generate(6, 6);

        let renderer = UnicodeRenderer::new(UnicodeRenderStyle::Hexadecimal, true);
        let str = renderer.render(&grid);
        let expected = indoc!(
            "
            24ECCA
            5A36A1
            23795A
            795C83
            5CA6CB
            4CD949
            "
        );
        assert_eq!(str, expected);

        let renderer = UnicodeRenderer::new(UnicodeRenderStyle::Hexadecimal, false);
        let str = renderer.render(&grid);
        let expected = "24ECCA5A36A123795A795C835CA6CB4CD949";
        assert_eq!(str, expected);
    }
}
