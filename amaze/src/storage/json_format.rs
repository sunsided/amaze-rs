use crate::direction4::Direction4;
use crate::direction6::Direction6;
use crate::wall4_grid::Wall4Grid;
use crate::wall6_grid::Wall6Grid;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct MazeJson {
    version: u8,
    #[serde(rename = "type")]
    maze_type: String,
    width: usize,
    height: usize,
    cells: Vec<u8>,
}

#[derive(Debug)]
pub enum JsonError {
    Serialize(serde_json::Error),
    Deserialize(serde_json::Error),
    InvalidType(String),
    InvalidVersion(u8),
    InvalidCellCount { expected: usize, got: usize },
}

impl std::fmt::Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonError::Serialize(e) => write!(f, "Serialization error: {e}"),
            JsonError::Deserialize(e) => write!(f, "Deserialization error: {e}"),
            JsonError::InvalidType(msg) => write!(f, "Invalid maze type: {msg}"),
            JsonError::InvalidVersion(v) => write!(f, "Unsupported version: {v}"),
            JsonError::InvalidCellCount { expected, got } => {
                write!(f, "Invalid cell count: expected {expected}, got {got}")
            }
        }
    }
}

impl std::error::Error for JsonError {}

impl From<serde_json::Error> for JsonError {
    fn from(e: serde_json::Error) -> Self {
        JsonError::Deserialize(e)
    }
}

pub trait ToJson {
    fn to_json(&self) -> Result<String, JsonError>;
}

pub trait FromJson: Sized {
    fn from_json(json: &str) -> Result<Self, JsonError>;
}

impl ToJson for Wall4Grid {
    fn to_json(&self) -> Result<String, JsonError> {
        let cells: Vec<u8> = self.coords().map(|c| *self[c]).collect();
        let maze = MazeJson {
            version: 1,
            maze_type: "square".into(),
            width: self.width(),
            height: self.height(),
            cells,
        };
        serde_json::to_string(&maze).map_err(JsonError::Serialize)
    }
}

impl FromJson for Wall4Grid {
    fn from_json(json: &str) -> Result<Self, JsonError> {
        let maze: MazeJson = serde_json::from_str(json).map_err(JsonError::Deserialize)?;

        if maze.version != 1 {
            return Err(JsonError::InvalidVersion(maze.version));
        }
        if maze.maze_type != "square" {
            return Err(JsonError::InvalidType(maze.maze_type));
        }

        let expected = maze.width * maze.height;
        if maze.cells.len() != expected {
            return Err(JsonError::InvalidCellCount {
                expected,
                got: maze.cells.len(),
            });
        }

        let mut grid = Wall4Grid::new(maze.width, maze.height);
        for (i, coord) in grid.coords().collect::<Vec<_>>().into_iter().enumerate() {
            let byte = maze.cells[i];
            if byte & !0b00001111 != 0 {
                return Err(JsonError::InvalidType(format!(
                    "invalid wall byte 0b{byte:08b}"
                )));
            }
            grid[coord] = Direction4::from_bits(byte);
        }
        Ok(grid)
    }
}

impl ToJson for Wall6Grid {
    fn to_json(&self) -> Result<String, JsonError> {
        let cells: Vec<u8> = self.coords().map(|c| *self[c]).collect();
        let maze = MazeJson {
            version: 1,
            maze_type: "hex".into(),
            width: self.width(),
            height: self.height(),
            cells,
        };
        serde_json::to_string(&maze).map_err(JsonError::Serialize)
    }
}

impl FromJson for Wall6Grid {
    fn from_json(json: &str) -> Result<Self, JsonError> {
        let maze: MazeJson = serde_json::from_str(json).map_err(JsonError::Deserialize)?;

        if maze.version != 1 {
            return Err(JsonError::InvalidVersion(maze.version));
        }
        if maze.maze_type != "hex" {
            return Err(JsonError::InvalidType(maze.maze_type));
        }

        let expected = maze.width * maze.height;
        if maze.cells.len() != expected {
            return Err(JsonError::InvalidCellCount {
                expected,
                got: maze.cells.len(),
            });
        }

        let mut grid = Wall6Grid::new(maze.width, maze.height);
        for (i, coord) in grid.coords().collect::<Vec<_>>().into_iter().enumerate() {
            let byte = maze.cells[i];
            if byte & !0b00111111 != 0 {
                return Err(JsonError::InvalidType(format!(
                    "invalid wall byte 0b{byte:08b}"
                )));
            }
            grid[coord] = Direction6::from_bits(byte);
        }
        Ok(grid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::RecursiveBacktracker4;

    #[test]
    fn roundtrip_square_json() {
        let maze = RecursiveBacktracker4::new_from_seed(42).generate(10, 10);
        let json = maze.to_json().unwrap();
        let restored = Wall4Grid::from_json(&json).unwrap();

        assert_eq!(maze.width(), restored.width());
        assert_eq!(maze.height(), restored.height());
        for coord in maze.coords() {
            assert_eq!(maze[coord], restored[coord]);
        }
    }

    #[test]
    fn json_contains_expected_fields() {
        let maze = RecursiveBacktracker4::new_from_seed(42).generate(5, 5);
        let json = maze.to_json().unwrap();
        let parsed: MazeJson = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.maze_type, "square");
        assert_eq!(parsed.width, 5);
        assert_eq!(parsed.height, 5);
        assert_eq!(parsed.cells.len(), 25);
    }

    #[test]
    #[cfg(feature = "generator-hex-recursive-backtracker")]
    fn hex_json_roundtrip() {
        use crate::generators::RecursiveBacktracker6;
        let maze = RecursiveBacktracker6::new_from_seed(42).generate(5, 5);
        let json = maze.to_json().unwrap();
        let restored = Wall6Grid::from_json(&json).unwrap();

        assert_eq!(maze.width(), restored.width());
        assert_eq!(maze.height(), restored.height());
        for coord in maze.coords() {
            assert_eq!(*maze.get(coord).unwrap(), *restored.get(coord).unwrap());
        }
    }
}
