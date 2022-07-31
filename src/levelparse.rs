use serde::{Deserialize, Serialize};

use crate::board;

#[derive(Serialize, Deserialize, Debug)]
pub struct Level {
    pub tiles: Vec<Tile>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tile {
    pub position: Position,
    pub sprite: board::TileSprite,
    #[serde(default)]
    pub variant: board::TileVariant,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl std::convert::From<&board::Position> for Position {
    fn from(p: &board::Position) -> Self {
        Self {
            x: p.x,
            y: p.y,
            z: p.z,
        }
    }
}

impl Level {
    pub fn serialize_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self)
    }
    pub fn deserialize_json(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }
}
