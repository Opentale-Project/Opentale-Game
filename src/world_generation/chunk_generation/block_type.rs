use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum BlockType {
    Air,
    Stone,
    Grass,
    Log,
    Snow,
    Leaf,
    Dirt,
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
            BlockType::Grass => match block_face {
                BlockFace::Top => 0,
                BlockFace::Bottom => 7,
                _ => 6,
            },
            BlockType::Stone => 1,
            BlockType::Snow => 2,
            BlockType::Leaf => 5,
            BlockType::Dirt => 7,
            _ => 0,
        }
    }

    pub fn is_covering_for(&self, other: &BlockType) -> bool {
        if self == other {
            return true;
        }

        match self {
            BlockType::Air | BlockType::Leaf => false,
            _ => true,
        }
    }
}
