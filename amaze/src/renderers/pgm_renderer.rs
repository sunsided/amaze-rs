use crate::grid_coord_2d::GridCoord2D;
use crate::room4::Wall4;
use crate::wall4_grid::Wall4Grid;
use std::ops::Index;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ImageFormat {
    /// Portable Pixmap (black white)
    PPM,
    /// Portable bitmap (colors)
    PBM,
}

/// Renderer for generating PPM or PBM images from a maze.
pub struct ImageRenderer {
    format: ImageFormat,
    // Optional: Customize colors for PPM
    wall_color: (u8, u8, u8), // RGB for walls
    path_color: (u8, u8, u8), // RGB for paths
}

impl ImageRenderer {
    /// Creates a new ImageRenderer with the specified format.
    ///
    /// For PPM, you can optionally specify wall and path colors.
    ///
    /// ## Example
    /// ```
    /// use amaze::renderers::{ImageRenderer, ImageFormat};
    ///
    /// let renderer = ImageRenderer::new(ImageFormat::PPM);
    /// ```
    pub fn new(format: ImageFormat) -> Self {
        Self {
            format,
            wall_color: (12, 12, 72),    // Default: (Almost) Black walls
            path_color: (255, 255, 255), // Default: White paths
        }
    }

    /// Sets custom colors for walls and paths (only applicable for PPM).
    ///
    /// ## Example
    /// ```
    /// # use amaze::renderers::{ImageRenderer, ImageFormat};
    /// let mut renderer = ImageRenderer::new(ImageFormat::PPM);
    /// renderer.set_colors((255, 0, 0), (255, 255, 255)); // Red walls, White paths
    /// ```
    pub fn set_colors(&mut self, wall_color: (u8, u8, u8), path_color: (u8, u8, u8)) {
        self.wall_color = wall_color;
        self.path_color = path_color;
    }

    /// Renders the maze into the specified image format.
    ///
    /// Returns the image data as a `String`.
    ///
    /// ## Example
    /// ```
    /// # use amaze::renderers::{ImageRenderer, ImageFormat};
    /// # use amaze::generators::RecursiveBacktracker4;
    /// # let gen = RecursiveBacktracker4::default();
    /// # let grid = gen.generate(6, 6);
    /// let renderer = ImageRenderer::new(ImageFormat::PBM);
    /// let image_data = renderer.render(&grid);
    /// ```
    pub fn render(&self, grid: &Wall4Grid) -> String {
        match self.format {
            ImageFormat::PPM => self.render_ppm(grid),
            ImageFormat::PBM => self.render_pbm(grid),
        }
    }

    /// Renders the maze as a PPM image.
    fn render_ppm(&self, grid: &Wall4Grid) -> String {
        let image_width = grid.width() * 2 + 1;
        let image_height = grid.height() * 2 + 1;

        let mut ppm = String::new();

        // PPM Header
        ppm.push_str(&format!("P3\n{} {}\n255\n", image_width, image_height));

        // Initialize all pixels to walls (black)
        let mut pixels = vec![vec![self.wall_color; image_width]; image_height];

        // Carve out paths
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                let cell = GridCoord2D::new(x, y);
                let img_x = x * 2 + 1;
                let img_y = y * 2 + 1;

                // Set the cell position to path color
                pixels[img_y][img_x] = self.path_color;

                // Check and carve passages
                let wall = grid.index(cell);
                if !wall.contains(Wall4::NORTH) && y > 0 {
                    pixels[img_y - 1][img_x] = self.path_color;
                }
                if !wall.contains(Wall4::SOUTH) && y < grid.height() - 1 {
                    pixels[img_y + 1][img_x] = self.path_color;
                }
                if !wall.contains(Wall4::EAST) && x < grid.width() - 1 {
                    pixels[img_y][img_x + 1] = self.path_color;
                }
                if !wall.contains(Wall4::WEST) && x > 0 {
                    pixels[img_y][img_x - 1] = self.path_color;
                }
            }
        }

        // Convert pixel data to PPM format
        for row in pixels {
            for (i, pixel) in row.iter().enumerate() {
                ppm.push_str(&format!("{} {} {} ", pixel.0, pixel.1, pixel.2));
                // Optional: Add line breaks every 5 pixels for readability
                if i % 5 == 4 {
                    ppm.push('\n');
                }
            }
            ppm.push('\n');
        }

        ppm
    }

    /// Renders the maze as a PBM image.
    fn render_pbm(&self, grid: &Wall4Grid) -> String {
        let image_width = grid.width() * 2 + 1;
        let image_height = grid.height() * 2 + 1;

        let mut pbm = String::new();

        // PBM Header
        pbm.push_str(&format!("P1\n{} {}\n", image_width, image_height));

        // Initialize all pixels to walls (1)
        let mut pixels = vec![vec![1; image_width]; image_height];

        // Carve out paths
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                let cell = GridCoord2D::new(x, y);
                let img_x = x * 2 + 1;
                let img_y = y * 2 + 1;

                // Set the cell position to path (0)
                pixels[img_y][img_x] = 0;

                // Check and carve passages
                let wall = grid.index(cell);
                if !wall.contains(Wall4::NORTH) && y > 0 {
                    pixels[img_y - 1][img_x] = 0;
                }
                if !wall.contains(Wall4::SOUTH) && y < grid.height() - 1 {
                    pixels[img_y + 1][img_x] = 0;
                }
                if !wall.contains(Wall4::EAST) && x < grid.width() - 1 {
                    pixels[img_y][img_x + 1] = 0;
                }
                if !wall.contains(Wall4::WEST) && x > 0 {
                    pixels[img_y][img_x - 1] = 0;
                }
            }
        }

        // Convert pixel data to PBM format
        for row in pixels {
            for pixel in row {
                pbm.push_str(&format!("{} ", pixel));
            }
            pbm.push('\n');
        }

        pbm
    }
}
