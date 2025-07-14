use noise::{Add, Constant, Max, MultiFractal, Multiply, NoiseFn, ScalePoint, Simplex};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::world_generation::chunk_generation::noise::{
    gradient_fractal_noise::GFT, smooth_step::SmoothStep,
};

#[derive(Serialize, Deserialize)]
pub enum TerrainNoiseType {
    Simplex,
    Constant {
        value: f64,
    },
    Add {
        a_index: usize,
        b_index: usize,
    },
    Max {
        a_index: usize,
        b_index: usize,
    },
    Multiply {
        a_index: usize,
        b_index: usize,
    },
    SmoothStep {
        noise_index: usize,
        steps: f64,
        smoothness: f64,
    },
    ScalePoint {
        noise_index: usize,
        scale: f64,
    },
    GFT {
        noise_index: usize,
        octaves: usize,
        frequency: f64,
        lacunarity: f64,
        persistence: f64,
        gradient: f64,
        amplitude: f64,
    },
}

impl TerrainNoiseType {
    pub fn to_noise_fn(
        &self,
        noise_types: &Vec<TerrainNoiseType>,
        rng: &mut impl Rng,
    ) -> Box<dyn NoiseFn<f64, 2>> {
        match self {
            TerrainNoiseType::Simplex => Box::new(Simplex::new(rng.random())),
            TerrainNoiseType::Add { a_index, b_index } => Box::new(Add::new(
                noise_types[*a_index].to_noise_fn(noise_types, rng),
                noise_types[*b_index].to_noise_fn(noise_types, rng),
            )),
            TerrainNoiseType::Constant { value } => Box::new(Constant::new(*value)),
            TerrainNoiseType::Max { a_index, b_index } => Box::new(Max::new(
                noise_types[*a_index].to_noise_fn(noise_types, rng),
                noise_types[*b_index].to_noise_fn(noise_types, rng),
            )),
            TerrainNoiseType::Multiply { a_index, b_index } => Box::new(Multiply::new(
                noise_types[*a_index].to_noise_fn(noise_types, rng),
                noise_types[*b_index].to_noise_fn(noise_types, rng),
            )),
            TerrainNoiseType::SmoothStep {
                noise_index,
                steps,
                smoothness,
            } => Box::new(
                SmoothStep::new(noise_types[*noise_index].to_noise_fn(noise_types, rng))
                    .set_smoothness(*smoothness)
                    .set_steps(*steps),
            ),
            TerrainNoiseType::ScalePoint { noise_index, scale } => Box::new(
                ScalePoint::new(noise_types[*noise_index].to_noise_fn(noise_types, rng))
                    .set_scale(*scale),
            ),
            TerrainNoiseType::GFT {
                noise_index,
                octaves,
                frequency,
                lacunarity,
                persistence,
                gradient,
                amplitude,
            } => Box::new(
                GFT::new_with_source(noise_types[*noise_index].to_noise_fn(noise_types, rng))
                    .set_octaves(*octaves)
                    .set_frequency(*frequency)
                    .set_lacunarity(*lacunarity)
                    .set_persistence(*persistence)
                    .set_gradient(*gradient)
                    .set_amplitude(*amplitude),
            ),
        }
    }
}
