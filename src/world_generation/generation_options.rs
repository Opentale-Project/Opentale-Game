use crate::world_generation::chunk_generation::{
    block_type::BlockType,
    noise::terrain_noise::{TERRAIN_NOISE_FILE_PATH, TerrainNoise},
    structures::{
        oak_structure_generator::OakStructureGenerator,
        structure_generator::{StructureGenerator, VoxelStructureMetadata},
        tree_structure_generator::TreeStructureGenerator,
    },
};
use bevy::prelude::*;
use fastnoise_lite::FastNoiseLite;
use noise::NoiseFn;
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use std::sync::Arc;
use std::{fs::File, io::Read};

#[derive(Resource)]
pub struct GenerationOptionsResource(pub Arc<GenerationOptions>);

impl GenerationOptionsResource {
    pub fn from_seed(seed: u64) -> Self {
        // let tree_house = vox_data_to_structure_data(
        //     &from_file("assets/tree_house.vox").unwrap(),
        // );
        // let box_structure =
        //     vox_data_to_structure_data(&from_file("assets/box.vox").unwrap());
        let mut terrain_noise_file = File::open(TERRAIN_NOISE_FILE_PATH)
            .expect("Terrain noise config not found!");
        let mut terrain_noise_text = String::new();
        terrain_noise_file
            .read_to_string(&mut terrain_noise_text)
            .expect("Failed reading terrain config file!");
        let terrain_noise: TerrainNoise = ron::from_str(&terrain_noise_text)
            .expect("Failed deserializing terrain noise config!");

        let mut rng = StdRng::seed_from_u64(seed);

        Self(Arc::new(GenerationOptions {
            seed,
            terrain_noise,
            generate_paths: false,
            structure_generators: vec![
                Arc::new(Box::new(OakStructureGenerator::new(
                    VoxelStructureMetadata {
                        model_size: [27, 27, 27],
                        generation_size: [64, 64],
                        grid_offset: [24, 16],
                        generate_debug_blocks: false,
                        debug_rgb_multiplier: [1., 1., 1.],
                        noise: get_seeded_white_noise(rng.random()),
                    },
                ))),
                Arc::new(Box::new(OakStructureGenerator::new(
                    VoxelStructureMetadata {
                        model_size: [27, 27, 27],
                        generation_size: [64, 64],
                        grid_offset: [43, 52],
                        generate_debug_blocks: false,
                        debug_rgb_multiplier: [1., 1., 1.],
                        noise: get_seeded_white_noise(rng.random()),
                    },
                ))),
                Arc::new(Box::new(OakStructureGenerator::new(
                    VoxelStructureMetadata {
                        model_size: [27, 27, 27],
                        generation_size: [64, 64],
                        grid_offset: [10, 4],
                        generate_debug_blocks: false,
                        debug_rgb_multiplier: [1., 1., 1.],
                        noise: get_seeded_white_noise(rng.random()),
                    },
                ))),
                // Arc::new(Box::new(FixedStructureGenerator {
                //     fixed_structure_model: tree_house.0.clone(),
                //     fixed_structure_metadata: VoxelStructureMetadata {
                //         model_size: tree_house.1,
                //         generation_size: [1000, 1000],
                //         grid_offset: [7, 11],
                //         generate_debug_blocks: false,
                //         debug_rgb_multiplier: [1., 1., 1.],
                //         noise: get_seeded_white_noise(rng.random()),
                //     },
                // })),
            ],
            structure_assets: vec![
                // StructureAsset {
                //     _blocks: (*box_structure.0).clone(),
                // }
            ],
        }))
    }
}

fn get_seeded_white_noise(seed: u64) -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(seed as i32);
    noise.set_noise_type(Some(fastnoise_lite::NoiseType::Value));
    noise.set_frequency(Some(100.));
    noise
}

pub struct GenerationOptions {
    pub seed: u64,
    pub structure_generators:
        Vec<Arc<Box<dyn StructureGenerator + Send + Sync>>>,
    pub structure_assets: Vec<StructureAsset>,
    pub generate_paths: bool,
    pub terrain_noise: TerrainNoise,
}

impl GenerationOptions {
    pub fn get_terrain_noise(&self) -> impl NoiseFn<f64, 2> {
        self.terrain_noise
            .get_noise_fn(&mut StdRng::seed_from_u64(self.seed + 1))
    }
}

pub struct StructureAsset {
    pub _blocks: Vec<Vec<Vec<BlockType>>>,
}

// fn vox_data_to_blocks(vox_data: &VoxData) -> Vec<Vec<Vec<BlockType>>> {
//     let model = vox_data.models.first().unwrap();
//     let mut result: Vec<Vec<Vec<BlockType>>> =
//         Vec::with_capacity(model.size.x as usize);
//     for x in 0..model.size.x {
//         result.push(Vec::with_capacity(model.size.z as usize));
//         for y in 0..model.size.z {
//             result[x as usize].push(Vec::with_capacity(model.size.y as usize));
//             for _ in 0..model.size.y {
//                 result[x as usize][y as usize].push(BlockType::Air);
//             }
//         }
//     }

//     for voxel in model.voxels.iter() {
//         //let color = vox_data.palette.colors[voxel.color_index.0 as usize];
//         let [x, y, z] = [
//             voxel.point.x as usize,
//             voxel.point.y as usize,
//             voxel.point.z as usize,
//         ];
//         result[x][z][y] = BlockType::Stone;
//     }

//     result
// }

// fn vox_data_model_size(vox_data: &VoxData) -> [i32; 3] {
//     let model_size = vox_data.models.first().unwrap().size;
//     [
//         model_size.x as i32,
//         model_size.z as i32,
//         model_size.y as i32,
//     ]
// }

// fn vox_data_to_structure_data(
//     vox_data: &VoxData,
// ) -> (Arc<Vec<Vec<Vec<BlockType>>>>, [i32; 3]) {
//     (
//         Arc::new(vox_data_to_blocks(vox_data)),
//         vox_data_model_size(vox_data),
//     )
// }
