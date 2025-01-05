#[cfg(feature = "pgm-renderer")]
mod pgm_renderer;
#[cfg(feature = "unicode-renderer")]
mod unicode_renderer;

#[cfg(feature = "pgm-renderer")]
pub use pgm_renderer::{ImageFormat, ImageRenderer};
use std::str::FromStr;
#[cfg(feature = "unicode-renderer")]
pub use unicode_renderer::{UnicodeRenderStyle, UnicodeRenderer};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RenderStyle {
    #[cfg(feature = "unicode-renderer")]
    Unicode(UnicodeRenderStyle),

    #[cfg(feature = "pgm-renderer")]
    Image(ImageFormat),
}

#[cfg(feature = "unicode-renderer")]
impl Default for RenderStyle {
    fn default() -> Self {
        RenderStyle::Unicode(UnicodeRenderStyle::Heavy)
    }
}

impl FromStr for RenderStyle {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            #[cfg(feature = "unicode-renderer")]
            "heavy" => Ok(RenderStyle::Unicode(UnicodeRenderStyle::Heavy)),
            #[cfg(feature = "unicode-renderer")]
            "thin" => Ok(RenderStyle::Unicode(UnicodeRenderStyle::Thin)),
            #[cfg(feature = "unicode-renderer")]
            "double" => Ok(RenderStyle::Unicode(UnicodeRenderStyle::Double)),
            #[cfg(feature = "unicode-renderer")]
            "hex" => Ok(RenderStyle::Unicode(UnicodeRenderStyle::Hexadecimal)),
            #[cfg(feature = "pgm-renderer")]
            "ppm" => Ok(RenderStyle::Image(ImageFormat::PPM)),
            #[cfg(feature = "pgm-renderer")]
            "pbm" => Ok(RenderStyle::Image(ImageFormat::PBM)),
            _ => Err(format!(
                "Invalid style '{}'. Valid styles are: heavy, thin, double, hex (with feature unicode-renderer); ppm, pbm (with feature pgm-renderer).",
                input
            )),
        }
    }
}
