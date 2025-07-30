use bevy::math::IVec3;

use crate::world_generation::chunk_generation::block_type::BlockType;

use super::CHUNK_SIZE;

pub type VoxelArray =
    [BlockType; (CHUNK_SIZE + 2) * (CHUNK_SIZE + 2) * (CHUNK_SIZE + 2)];

pub struct VoxelData {
    pub array: VoxelArray,
}

impl Default for VoxelData {
    fn default() -> Self {
        Self {
            array: [BlockType::Air;
                (CHUNK_SIZE + 2) * (CHUNK_SIZE + 2) * (CHUNK_SIZE + 2)],
        }
    }
}

impl VoxelData {
    pub fn get_block<T: Into<IVec3>>(&self, position: T) -> BlockType {
        let index = Self::position_to_indexes(position);
        self.array[index]
    }

    pub fn set_block<T: Into<IVec3>>(&mut self, position: T, block: BlockType) {
        let index = Self::position_to_indexes(position);
        self.array[index] = block;
    }

    fn position_to_indexes<T: Into<IVec3>>(position: T) -> usize {
        let position: IVec3 = position.into();
        let index = position.x as usize
            + (position.y as usize * (CHUNK_SIZE + 2))
            + (position.z as usize * (CHUNK_SIZE + 2) * (CHUNK_SIZE + 2));
        index
    }
}
