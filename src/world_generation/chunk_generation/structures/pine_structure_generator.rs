use rand::rngs::StdRng;

use crate::world_generation::chunk_generation::{
    VOXEL_SIZE,
    block_type::BlockType,
    structures::{
        foliage_generation::{
            pine_l_system::PineLSystem, tree_l_system::LSystem,
        },
        structure_generator::VoxelStructureMetadata,
        tree_structure_generator::TreeStructureGenerator,
    },
};

pub struct PineStructureGenerator {
    pub fixed_structure_metadata: VoxelStructureMetadata,
}

const PINE_VOXEL_SIZE: usize = (32f32 / VOXEL_SIZE) as usize;
const PINE_VOXEL_HEIGHT: usize = (70f32 / VOXEL_SIZE) as usize;

impl TreeStructureGenerator for PineStructureGenerator {
    fn new(mut metadata: VoxelStructureMetadata) -> Self {
        Self::adjust_metadata(&mut metadata);

        Self {
            fixed_structure_metadata: metadata,
        }
    }

    fn get_structure_metadata(&self) -> &VoxelStructureMetadata {
        &self.fixed_structure_metadata
    }

    fn grow(&self, rng: &mut StdRng) -> Vec<Vec<Vec<BlockType>>> {
        PineLSystem::grow_new::<
            PINE_VOXEL_SIZE,
            PINE_VOXEL_HEIGHT,
            PINE_VOXEL_SIZE,
        >(rng)
    }
}
