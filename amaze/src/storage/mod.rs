#[cfg(feature = "binary-format")]
mod binary_format;
#[cfg(feature = "file-io")]
mod file_io;
#[cfg(feature = "json-format")]
mod json_format;

#[cfg(feature = "binary-format")]
pub use binary_format::BinaryError;
#[cfg(feature = "binary-format")]
pub use binary_format::{FromBinary, ToBinary};
#[cfg(feature = "file-io")]
pub use file_io::{
    MazeFormat, MazeIoError, load_wall4_grid, load_wall6_grid, save_wall4_grid, save_wall6_grid,
};
#[cfg(feature = "json-format")]
pub use json_format::JsonError;
#[cfg(feature = "json-format")]
pub use json_format::{FromJson, ToJson};
