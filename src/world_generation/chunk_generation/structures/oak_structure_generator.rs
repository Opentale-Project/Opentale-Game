use rand::rngs::StdRng;

use crate::world_generation::chunk_generation::{
    VOXEL_SIZE,
    block_type::BlockType,
    structures::{
        foliage_generation::{
            oak_l_system::OakLSystem, tree_l_system::LSystem,
        },
        structure_generator::VoxelStructureMetadata,
        tree_structure_generator::TreeStructureGenerator,
    },
};

pub struct OakStructureGenerator {
    pub fixed_structure_metadata: VoxelStructureMetadata,
}

const OAK_VOXEL_SIZE: usize = (27f32 / VOXEL_SIZE) as usize;

impl TreeStructureGenerator for OakStructureGenerator {
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
        OakLSystem::grow_new::<OAK_VOXEL_SIZE, OAK_VOXEL_SIZE, OAK_VOXEL_SIZE>(
            rng,
        )
    }
}
