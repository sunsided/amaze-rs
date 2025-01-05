mod pgm_renderer;
mod unicode_renderer;

pub use pgm_renderer::{ImageFormat, ImageRenderer};
use std::str::FromStr;
pub use unicode_renderer::{UnicodeRenderStyle, UnicodeRenderer};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RenderStyle {
    Unicode(UnicodeRenderStyle),
    Image(ImageFormat),
}

impl Default for RenderStyle {
    fn default() -> Self {
        RenderStyle::Unicode(UnicodeRenderStyle::Heavy)
    }
}

impl FromStr for RenderStyle {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "heavy" => Ok(RenderStyle::Unicode(UnicodeRenderStyle::Heavy)),
            "thin" => Ok(RenderStyle::Unicode(UnicodeRenderStyle::Thin)),
            "double" => Ok(RenderStyle::Unicode(UnicodeRenderStyle::Double)),
            "hex" => Ok(RenderStyle::Unicode(UnicodeRenderStyle::Hexadecimal)),
            "ppm" => Ok(RenderStyle::Image(ImageFormat::PPM)),
            "pbm" => Ok(RenderStyle::Image(ImageFormat::PBM)),
            _ => Err(format!(
                "Invalid style '{}'. Valid styles are: heavy, thin, double, hex, ppm, pbm.",
                input
            )),
        }
    }
}
