use crate::wall4_grid::Wall4Grid;
use crate::wall6_grid::Wall6Grid;
use std::fs;
use std::io;
use std::path::Path;

#[cfg(feature = "binary-format")]
use super::binary_format::{FromBinary, ToBinary};
#[cfg(feature = "json-format")]
use super::json_format::{FromJson, ToJson};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MazeFormat {
    Auto,
    Binary,
    Json,
}

impl MazeFormat {
    fn from_extension(path: &Path) -> Self {
        match path.extension().and_then(|e| e.to_str()) {
            Some("bin") => MazeFormat::Binary,
            Some("json") => MazeFormat::Json,
            _ => MazeFormat::Auto,
        }
    }
}

#[derive(Debug)]
pub enum MazeIoError {
    Io(io::Error),
    #[cfg(feature = "binary-format")]
    Binary(super::BinaryError),
    #[cfg(feature = "json-format")]
    Json(super::JsonError),
    UnsupportedFormat(String),
}

impl std::fmt::Display for MazeIoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MazeIoError::Io(e) => write!(f, "I/O error: {e}"),
            #[cfg(feature = "binary-format")]
            MazeIoError::Binary(e) => write!(f, "Binary format error: {e}"),
            #[cfg(feature = "json-format")]
            MazeIoError::Json(e) => write!(f, "JSON format error: {e}"),
            MazeIoError::UnsupportedFormat(msg) => write!(f, "Unsupported format: {msg}"),
        }
    }
}

impl std::error::Error for MazeIoError {}

impl From<io::Error> for MazeIoError {
    fn from(e: io::Error) -> Self {
        MazeIoError::Io(e)
    }
}

#[cfg(feature = "binary-format")]
impl From<super::BinaryError> for MazeIoError {
    fn from(e: super::BinaryError) -> Self {
        MazeIoError::Binary(e)
    }
}

#[cfg(feature = "json-format")]
impl From<super::JsonError> for MazeIoError {
    fn from(e: super::JsonError) -> Self {
        MazeIoError::Json(e)
    }
}

pub fn save_wall4_grid(
    path: impl AsRef<Path>,
    maze: &Wall4Grid,
    format: MazeFormat,
) -> Result<(), MazeIoError> {
    let format = if format == MazeFormat::Auto {
        MazeFormat::from_extension(path.as_ref())
    } else {
        format
    };

    match format {
        #[cfg(feature = "binary-format")]
        MazeFormat::Binary => {
            let bytes = maze.to_binary()?;
            fs::write(path, bytes)?;
            Ok(())
        }
        #[cfg(feature = "json-format")]
        MazeFormat::Json => {
            let json = maze.to_json()?;
            fs::write(path, json)?;
            Ok(())
        }
        other => Err(MazeIoError::UnsupportedFormat(format!(
            "cannot save square maze as {:?} (enable binary-format or json-format feature)",
            other
        ))),
    }
}

pub fn load_wall4_grid(
    path: impl AsRef<Path>,
    format: MazeFormat,
) -> Result<Wall4Grid, MazeIoError> {
    let format = if format == MazeFormat::Auto {
        MazeFormat::from_extension(path.as_ref())
    } else {
        format
    };

    let data = fs::read(path)?;

    match format {
        #[cfg(feature = "binary-format")]
        MazeFormat::Binary => Ok(Wall4Grid::from_binary(&data)?),
        #[cfg(feature = "json-format")]
        MazeFormat::Json => Ok(Wall4Grid::from_json(std::str::from_utf8(&data).map_err(
            |e| MazeIoError::Io(io::Error::new(io::ErrorKind::InvalidData, e)),
        )?)?),
        other => Err(MazeIoError::UnsupportedFormat(format!(
            "cannot load square maze as {:?} (enable binary-format or json-format feature)",
            other
        ))),
    }
}

pub fn save_wall6_grid(
    path: impl AsRef<Path>,
    maze: &Wall6Grid,
    format: MazeFormat,
) -> Result<(), MazeIoError> {
    let format = if format == MazeFormat::Auto {
        MazeFormat::from_extension(path.as_ref())
    } else {
        format
    };

    match format {
        #[cfg(feature = "binary-format")]
        MazeFormat::Binary => {
            let bytes = maze.to_binary()?;
            fs::write(path, bytes)?;
            Ok(())
        }
        #[cfg(feature = "json-format")]
        MazeFormat::Json => {
            let json = maze.to_json()?;
            fs::write(path, json)?;
            Ok(())
        }
        other => Err(MazeIoError::UnsupportedFormat(format!(
            "cannot save hex maze as {:?} (enable binary-format or json-format feature)",
            other
        ))),
    }
}

pub fn load_wall6_grid(
    path: impl AsRef<Path>,
    format: MazeFormat,
) -> Result<Wall6Grid, MazeIoError> {
    let format = if format == MazeFormat::Auto {
        MazeFormat::from_extension(path.as_ref())
    } else {
        format
    };

    let data = fs::read(path)?;

    match format {
        #[cfg(feature = "binary-format")]
        MazeFormat::Binary => Ok(Wall6Grid::from_binary(&data)?),
        #[cfg(feature = "json-format")]
        MazeFormat::Json => Ok(Wall6Grid::from_json(std::str::from_utf8(&data).map_err(
            |e| MazeIoError::Io(io::Error::new(io::ErrorKind::InvalidData, e)),
        )?)?),
        other => Err(MazeIoError::UnsupportedFormat(format!(
            "cannot load hex maze as {:?} (enable binary-format or json-format feature)",
            other
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::RecursiveBacktracker4;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn temp_path(ext: &str) -> PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        std::env::temp_dir().join(format!("amaze_test_{}_{}.{}", std::process::id(), id, ext))
    }

    #[test]
    fn save_and_load_square_binary() {
        let maze = RecursiveBacktracker4::new_from_seed(42).generate(10, 10);
        let path = temp_path("bin");

        save_wall4_grid(&path, &maze, MazeFormat::Binary).unwrap();
        let loaded = load_wall4_grid(&path, MazeFormat::Binary).unwrap();

        assert_eq!(maze.width(), loaded.width());
        assert_eq!(maze.height(), loaded.height());
        for coord in maze.coords() {
            assert_eq!(maze[coord], loaded[coord]);
        }

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn save_and_load_square_json() {
        let maze = RecursiveBacktracker4::new_from_seed(42).generate(10, 10);
        let path = temp_path("json");

        save_wall4_grid(&path, &maze, MazeFormat::Json).unwrap();
        let loaded = load_wall4_grid(&path, MazeFormat::Json).unwrap();

        assert_eq!(maze.width(), loaded.width());
        assert_eq!(maze.height(), loaded.height());
        for coord in maze.coords() {
            assert_eq!(maze[coord], loaded[coord]);
        }

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn auto_detect_format_from_extension() {
        let maze = RecursiveBacktracker4::new_from_seed(42).generate(5, 5);

        let json_path = temp_path("json");
        save_wall4_grid(&json_path, &maze, MazeFormat::Auto).unwrap();
        let loaded = load_wall4_grid(&json_path, MazeFormat::Auto).unwrap();
        assert_eq!(maze.width(), loaded.width());
        let _ = std::fs::remove_file(&json_path);

        let bin_path = temp_path("bin");
        save_wall4_grid(&bin_path, &maze, MazeFormat::Auto).unwrap();
        let loaded = load_wall4_grid(&bin_path, MazeFormat::Auto).unwrap();
        assert_eq!(maze.width(), loaded.width());
        let _ = std::fs::remove_file(&bin_path);
    }
}
