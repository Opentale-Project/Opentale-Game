use crate::world_generation::{
    chunk_generation::{
        BlockType,
        structures::{
            oak_structure_generator::OakStructureGenerator,
            structure_generator::{
                FixedStructureGenerator, 
                StructureGenerator, 
                VoxelStructureMetadata,
            },
            tree_structure_generator::TreeStructureGenerator,
        }
    },
    chunk_loading::country_cache::{
        CountryCache, 
        PathCache, 
        StructureCache,
    }
};
use bevy::prelude::{IVec2, Resource};
use fastnoise_lite::FastNoiseLite;
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use vox_format::{from_file, VoxData};

#[derive(Resource)]
pub struct GenerationOptionsResource(
    pub Arc<GenerationOptions>,
    pub HashMap<IVec2, GenerationState<CountryCache>>,
);

impl GenerationOptionsResource {
    pub fn from_seed(seed: u64) -> Self {
        let tree_house = vox_data_to_structure_data(
            &from_file("assets/tree_house.vox").unwrap()
        );
        let box_structure = vox_data_to_structure_data(
            &from_file("assets/box.vox").unwrap()
        );

        let mut rng = StdRng::seed_from_u64(seed);

        Self {
            0: Arc::new(GenerationOptions {
                seed,
                generate_paths: false,
                path_cache: GenerationCache::new(),
                structure_cache: GenerationCache::new(),
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
                    Arc::new(Box::new(FixedStructureGenerator {
                        fixed_structure_model: tree_house.0.clone(),
                        fixed_structure_metadata: VoxelStructureMetadata {
                            model_size: tree_house.1,
                            generation_size: [1000, 1000],
                            grid_offset: [7, 11],
                            generate_debug_blocks: false,
                            debug_rgb_multiplier: [1., 1., 1.],
                            noise: get_seeded_white_noise(rng.random()),
                        },
                    })),
                ],
                structure_assets: vec![StructureAsset {
                    _blocks: (*box_structure.0).clone(),
                }],
            }),
            1: HashMap::new(),
        }
    }
}

impl Default for GenerationOptionsResource {
    fn default() -> Self {
        Self::from_seed(3)
    }
}

fn get_seeded_white_noise(seed: u64) -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(seed as i32);
    noise.set_noise_type(Some(fastnoise_lite::NoiseType::Value));
    noise.set_frequency(Some(100.));
    noise
}

pub enum GenerationState<T> {
    Generating,
    Some(T),
}

pub struct GenerationOptions {
    pub seed: u64,
    pub structure_generators: Vec<Arc<Box<
        dyn StructureGenerator + Send + Sync
    >>>,
    pub structure_assets: Vec<StructureAsset>,
    pub path_cache: GenerationCache<IVec2, PathCache>,
    pub structure_cache: GenerationCache<IVec2, StructureCache>,
    pub generate_paths: bool,
}

pub trait GenerationCacheItem<K: Copy + Eq + Hash> {
    fn generate(key: K, generation_options: &GenerationOptions) -> Self;
}

pub struct GenerationCache<K: Copy + Eq + Hash, T: GenerationCacheItem<K>> {
    cache_lock: RwLock<HashMap<K, Arc<RwLock<Option<Arc<T>>>>>>,
}

impl<K: Copy + Eq + Hash, T: GenerationCacheItem<K>> GenerationCache<K, T> {
    pub fn new() -> Self {
        Self {
            cache_lock: RwLock::new(HashMap::new()),
        }
    }

    pub fn get_cache_entry(
        &self, 
        key: K, 
        generation_options: &GenerationOptions
    ) -> Arc<T> {
        self.get_generated_cache_entry(
            self.get_hash_lock_entry(key), 
            key, 
            generation_options
        )
    }

    pub fn try_get_entry_no_lock(&self, key: K) -> Option<Arc<T>> {
        match self.cache_lock.try_read() {
            Ok(read) => {
                let entry = read.get(&key)?;
                match entry.try_read() {
                    Ok(read) => match read.deref() {
                        None => None,
                        Some(t) => Some(t.clone()),
                    },
                    Err(_) => None,
                }
            }
            Err(_) => None,
        }
    }

    fn get_hash_lock_entry(&self, key: K) -> Arc<RwLock<Option<Arc<T>>>> {
        let read = self.cache_lock.read().unwrap();
        match read.get(&key) {
            None => {
                drop(read);
                let mut write = self.cache_lock.write().unwrap();
                let result = match write.get(&key) {
                    None => {
                        let lock = Arc::new(RwLock::new(None));
                        write.insert(key, lock);
                        write.get(&key).unwrap().clone()
                    }
                    Some(cache) => cache.clone(),
                };
                drop(write);
                result
            }
            Some(cache) => cache.clone(),
        }
    }

    fn get_generated_cache_entry(
        &self,
        hash_lock_entry: Arc<RwLock<Option<Arc<T>>>>,
        key: K,
        generation_options: &GenerationOptions,
    ) -> Arc<T> {
        let read = hash_lock_entry.read().unwrap();
        match read.deref() {
            None => {
                drop(read);
                let mut write = hash_lock_entry.write().unwrap();
                match write.deref() {
                    None => write
                        .insert(Arc::new(T::generate(key, generation_options)))
                        .clone(),
                    Some(country_cache) => country_cache.clone(),
                }
            }
            Some(country_cache) => country_cache.clone(),
        }
    }
}

pub struct StructureAsset {
    pub _blocks: Vec<Vec<Vec<BlockType>>>,
}

fn vox_data_to_blocks(vox_data: &VoxData) -> Vec<Vec<Vec<BlockType>>> {
    let model = vox_data.models.first().unwrap();
    let mut result: Vec<Vec<Vec<BlockType>>> = Vec::with_capacity(
        model.size.x as usize
    );
    for x in 0..model.size.x {
        result.push(Vec::with_capacity(model.size.z as usize));
        for y in 0..model.size.z {
            result[x as usize].push(Vec::with_capacity(model.size.y as usize));
            for _ in 0..model.size.y {
                result[x as usize][y as usize].push(BlockType::Air);
            }
        }
    }

    for voxel in model.voxels.iter() {
        //let color = vox_data.palette.colors[voxel.color_index.0 as usize];
        let [x, y, z] = [
            voxel.point.x as usize,
            voxel.point.y as usize,
            voxel.point.z as usize,
        ];
        result[x][z][y] = BlockType::Stone;
    }

    result
}

fn vox_data_model_size(vox_data: &VoxData) -> [i32; 3] {
    let model_size = vox_data.models.first().unwrap().size;
    [
        model_size.x as i32,
        model_size.z as i32,
        model_size.y as i32,
    ]
}

fn vox_data_to_structure_data(
    vox_data: &VoxData
) -> (Arc<Vec<Vec<Vec<BlockType>>>>, [i32; 3]) {
    (
        Arc::new(vox_data_to_blocks(vox_data)),
        vox_data_model_size(vox_data),
    )
}
