use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum BlockType {
    Air,
    Stone,
    Grass,
    Log,
    Snow,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum BlockFace {
    Top,
    Bottom,
    Front,
    Back,
    Right,
    Left,
}

impl BlockType {
    pub fn get_texture_id(&self, block_face: BlockFace) -> u32 {
        match self {
            BlockType::Log => match block_face {
                BlockFace::Top | BlockFace::Bottom => 4,
                _ => 3,
            },
            BlockType::Grass => 0,
            BlockType::Stone => 1,
            BlockType::Snow => 2,
            _ => 0,
        }
    }
}
