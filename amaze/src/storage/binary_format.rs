use crate::direction4::Direction4;
use crate::direction6::Direction6;
use crate::wall4_grid::Wall4Grid;
use crate::wall6_grid::Wall6Grid;
use std::io::{self, Write};

const MAGIC: [u8; 4] = *b"AMZE";
const VERSION: u8 = 1;
const TYPE_SQUARE: u8 = 0;
const TYPE_HEX: u8 = 1;

#[derive(Debug)]
pub enum BinaryError {
    Io(io::Error),
    InvalidHeader(String),
    InvalidData(String),
}

impl std::fmt::Display for BinaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryError::Io(e) => write!(f, "I/O error: {e}"),
            BinaryError::InvalidHeader(msg) => write!(f, "Invalid header: {msg}"),
            BinaryError::InvalidData(msg) => write!(f, "Invalid data: {msg}"),
        }
    }
}

impl std::error::Error for BinaryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BinaryError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for BinaryError {
    fn from(e: io::Error) -> Self {
        BinaryError::Io(e)
    }
}

pub trait ToBinary {
    fn to_binary(&self) -> Result<Vec<u8>, BinaryError>;
}

pub trait FromBinary: Sized {
    fn from_binary(data: &[u8]) -> Result<Self, BinaryError>;
}

impl ToBinary for Wall4Grid {
    fn to_binary(&self) -> Result<Vec<u8>, BinaryError> {
        let mut buf = Vec::with_capacity(8 + self.width() * self.height());
        buf.write_all(&MAGIC)?;
        buf.write_all(&[VERSION, TYPE_SQUARE])?;
        buf.write_all(&(self.width() as u16).to_le_bytes())?;
        buf.write_all(&(self.height() as u16).to_le_bytes())?;
        for cell in self.coords() {
            buf.push(*self[cell]);
        }
        Ok(buf)
    }
}

impl FromBinary for Wall4Grid {
    fn from_binary(data: &[u8]) -> Result<Self, BinaryError> {
        if data.len() < 8 {
            return Err(BinaryError::InvalidHeader("data too short".into()));
        }
        if data[0..4] != MAGIC {
            return Err(BinaryError::InvalidHeader("invalid magic bytes".into()));
        }
        if data[4] != VERSION {
            return Err(BinaryError::InvalidHeader(format!(
                "unsupported version {}",
                data[4]
            )));
        }
        if data[5] != TYPE_SQUARE {
            return Err(BinaryError::InvalidHeader("not a square maze".into()));
        }

        let width = u16::from_le_bytes([data[6], data[7]]) as usize;
        let height = u16::from_le_bytes([data[8], data[9]]) as usize;
        let expected = 10 + width * height;
        if data.len() < expected {
            return Err(BinaryError::InvalidData(format!(
                "expected {expected} bytes, got {}",
                data.len()
            )));
        }

        let mut grid = Wall4Grid::new(width, height);
        for (i, cell) in grid.coords().collect::<Vec<_>>().into_iter().enumerate() {
            let byte = data[10 + i];
            if byte & !0b00001111 != 0 {
                return Err(BinaryError::InvalidData(format!(
                    "invalid wall byte 0b{byte:08b}"
                )));
            }
            grid[cell] = Direction4::from_bits(byte);
        }
        Ok(grid)
    }
}

impl ToBinary for Wall6Grid {
    fn to_binary(&self) -> Result<Vec<u8>, BinaryError> {
        let mut buf = Vec::with_capacity(8 + self.width() * self.height());
        buf.write_all(&MAGIC)?;
        buf.write_all(&[VERSION, TYPE_HEX])?;
        buf.write_all(&(self.width() as u16).to_le_bytes())?;
        buf.write_all(&(self.height() as u16).to_le_bytes())?;
        for cell in self.coords() {
            buf.push(*self[cell]);
        }
        Ok(buf)
    }
}

impl FromBinary for Wall6Grid {
    fn from_binary(data: &[u8]) -> Result<Self, BinaryError> {
        if data.len() < 8 {
            return Err(BinaryError::InvalidHeader("data too short".into()));
        }
        if data[0..4] != MAGIC {
            return Err(BinaryError::InvalidHeader("invalid magic bytes".into()));
        }
        if data[4] != VERSION {
            return Err(BinaryError::InvalidHeader(format!(
                "unsupported version {}",
                data[4]
            )));
        }
        if data[5] != TYPE_HEX {
            return Err(BinaryError::InvalidHeader("not a hex maze".into()));
        }

        let width = u16::from_le_bytes([data[6], data[7]]) as usize;
        let height = u16::from_le_bytes([data[8], data[9]]) as usize;
        let expected = 10 + width * height;
        if data.len() < expected {
            return Err(BinaryError::InvalidData(format!(
                "expected {expected} bytes, got {}",
                data.len()
            )));
        }

        let mut grid = Wall6Grid::new(width, height);
        for (i, cell) in grid.coords().collect::<Vec<_>>().into_iter().enumerate() {
            let byte = data[10 + i];
            if byte & !0b00111111 != 0 {
                return Err(BinaryError::InvalidData(format!(
                    "invalid wall byte 0b{byte:08b}"
                )));
            }
            grid[cell] = Direction6::from_bits(byte);
        }
        Ok(grid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::RecursiveBacktracker4;

    #[test]
    fn roundtrip_square_maze() {
        let maze = RecursiveBacktracker4::new_from_seed(42).generate(10, 10);
        let bytes = maze.to_binary().unwrap();
        let restored = Wall4Grid::from_binary(&bytes).unwrap();

        assert_eq!(maze.width(), restored.width());
        assert_eq!(maze.height(), restored.height());
        for coord in maze.coords() {
            assert_eq!(maze[coord], restored[coord]);
        }
    }

    #[test]
    fn invalid_magic_returns_error() {
        let data = [0, 0, 0, 0, 1, 0, 0, 0, 0, 0];
        assert!(Wall4Grid::from_binary(&data).is_err());
    }

    #[test]
    fn too_short_returns_error() {
        assert!(Wall4Grid::from_binary(&[b'A', b'M', b'Z', b'E']).is_err());
    }

    #[test]
    #[cfg(feature = "generator-hex-recursive-backtracker")]
    fn hex_roundtrip() {
        use crate::generators::{MazeGenerator6D, RecursiveBacktracker6};
        let maze = RecursiveBacktracker6::new_from_seed(42).generate(5, 5);
        let bytes = maze.to_binary().unwrap();
        let restored = Wall6Grid::from_binary(&bytes).unwrap();

        assert_eq!(maze.width(), restored.width());
        assert_eq!(maze.height(), restored.height());
        for coord in maze.coords() {
            assert_eq!(*maze.get(coord).unwrap(), *restored.get(coord).unwrap());
        }
    }
}
