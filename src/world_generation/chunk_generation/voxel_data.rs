use bevy::math::IVec3;

use crate::world_generation::chunk_generation::{
    ambient_occlusion::AmbiantOcclusion, block_type::BlockType,
    mesh_generation::rotate_into_direction,
};

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

    pub fn get_ambiant_occlusion<T: Into<IVec3>>(
        &self,
        position: T,
        direction: IVec3,
    ) -> AmbiantOcclusion {
        let position: IVec3 = position.into() + direction;
        let right = rotate_into_direction(IVec3::Y, direction);
        let front = rotate_into_direction(IVec3::Z, direction);

        let get_corner_value = |right: IVec3, front: IVec3| -> u8 {
            let side_1 = self.get_block(position + right) != BlockType::Air;
            let side_2 = self.get_block(position + front) != BlockType::Air;
            let corner =
                self.get_block(position + right + front) != BlockType::Air;

            if side_1 && side_2 {
                return 0;
            }

            let mut count = 3;

            if side_1 {
                count -= 1;
            }
            if side_2 {
                count -= 1;
            }
            if corner {
                count -= 1;
            }

            count
        };

        let corner_1 = get_corner_value(-right, -front);
        let corner_2 = get_corner_value(-right, front);
        let corner_3 = get_corner_value(right, front);
        let corner_4 = get_corner_value(right, -front);

        AmbiantOcclusion {
            corner_1,
            corner_2,
            corner_3,
            corner_4,
        }
    }
}
