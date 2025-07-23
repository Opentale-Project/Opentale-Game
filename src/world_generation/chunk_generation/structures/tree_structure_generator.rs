use std::rc::Rc;

use bevy::math::IVec2;
use rand::{rngs::StdRng, SeedableRng};

use crate::world_generation::{
    chunk_generation::{
        BlockType, 
        structures::structure_generator::{
            StructureGenerator, 
            VoxelStructureMetadata
        },
        VOXEL_SIZE,
    },
    chunk_lod::ChunkLod,
};

pub trait TreeStructureGenerator {
    fn adjust_metadata(metadata: &mut VoxelStructureMetadata) {
        let model_size = [
            (metadata.model_size[0] as f32 / VOXEL_SIZE) as i32,
            (metadata.model_size[1] as f32 / VOXEL_SIZE) as i32,
            (metadata.model_size[2] as f32 / VOXEL_SIZE) as i32,
        ];

        let generation_size = [
            (metadata.generation_size[0] as f32 / VOXEL_SIZE) as i32,
            (metadata.generation_size[1] as f32 / VOXEL_SIZE) as i32,
        ];

        let grid_offset = [
            (metadata.grid_offset[0] as f32 / VOXEL_SIZE) as i32,
            (metadata.grid_offset[1] as f32 / VOXEL_SIZE) as i32,
        ];

        metadata.model_size = model_size;
        metadata.generation_size = generation_size;
        metadata.grid_offset = grid_offset;
    }

    fn new(metadata: VoxelStructureMetadata) -> Self;
    fn get_structure_metadata(&self) -> &VoxelStructureMetadata;
    fn grow(&self, rng: &mut StdRng) -> Vec<Vec<Vec<BlockType>>>;
}

impl<T: TreeStructureGenerator> StructureGenerator for T {
    fn get_structure_metadata(&self) -> &VoxelStructureMetadata {
        return &self.get_structure_metadata();
    }

    fn get_structure_model(
        &self,
        structure_position: IVec2,
        _: ChunkLod,
    ) -> Rc<Vec<Vec<Vec<BlockType>>>> {
        let noise_value = self
            .get_structure_metadata()
            .noise
            .get_noise_2d(structure_position.x as f32, structure_position.y as f32)
            * 0.5
            + 0.5;
        let mut rng = StdRng::seed_from_u64((noise_value.abs() * 10000.) as u64);

        let voxel_grid = Self::grow(&self, &mut rng);

        Rc::new(voxel_grid)
    }
}
