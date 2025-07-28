use noise::NoiseFn;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::world_generation::chunk_generation::noise::terrain_noise_type::TerrainNoiseType;

pub const TERRAIN_NOISE_FILE_PATH: &'static str = "assets/terrain_noise.ron";

#[derive(Serialize, Deserialize)]
pub struct TerrainNoise {
    noise_types: Vec<TerrainNoiseType>,
    start_index: usize,
}

impl TerrainNoise {
    pub fn new(start_index: usize, noise_types: Vec<TerrainNoiseType>) -> Self {
        Self {
            noise_types,
            start_index,
        }
    }

    pub fn get_noise_fn(&self, rng: &mut impl Rng) -> Box<dyn NoiseFn<f64, 2>> {
        self.noise_types[self.start_index].to_noise_fn(&self.noise_types, rng)
    }
}
