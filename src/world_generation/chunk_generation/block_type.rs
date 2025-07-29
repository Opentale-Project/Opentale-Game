use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum BlockType {
    Air,
    Stone,
    Grass,
    Log,
    Snow,
}

impl BlockType {
    pub fn get_texture_id(&self) -> u32 {
        match self {
            BlockType::Log => 2,
            BlockType::Grass => 0,
            BlockType::Stone => 1,
            BlockType::Snow => 3,
            _ => 0,
        }
    }
}
