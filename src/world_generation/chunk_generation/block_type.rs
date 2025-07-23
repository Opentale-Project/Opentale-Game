#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum BlockType {
    Air,
    Stone,
    Grass,
    Path,
    Snow,
}

impl BlockType {
    pub fn get_texture_id(&self) -> u32 {
        match self {
            BlockType::Path => 2,
            BlockType::Grass => 0,
            BlockType::Stone => 1,
            BlockType::Snow => 3,
            _ => 0,
        }
    }
}
