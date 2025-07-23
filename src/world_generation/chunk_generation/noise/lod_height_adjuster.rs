use noise::{NoiseFn, Seedable};

use crate::world_generation::chunk_lod::ChunkLod;

pub struct LodHeightAdjuster<T> {
    noise: T,
    lod: ChunkLod,
}

impl<T> LodHeightAdjuster<T> {
    pub const DEFAULT_LOD: ChunkLod = ChunkLod::Full;

    pub fn new(source: T, lod: ChunkLod) -> Self {
        Self {
            noise: source,
            lod: lod,
        }
    }

    pub fn set_lod(self, lod: ChunkLod) -> Self {
        Self { lod, ..self }
    }
}

impl<T> Default for LodHeightAdjuster<T>
where
    T: Default + Seedable,
{
    fn default() -> Self {
        Self {
            noise: Default::default(),
            lod: Self::DEFAULT_LOD,
        }
    }
}

impl<T> Seedable for LodHeightAdjuster<T>
where
    T: Default + Seedable,
{
    fn set_seed(mut self, seed: u32) -> Self {
        self.noise = T::default().set_seed(seed);
        self
    }

    fn seed(&self) -> u32 {
        self.noise.seed()
    }
}

impl<T> NoiseFn<f64, 2usize> for LodHeightAdjuster<T>
where
    T: NoiseFn<f64, 2usize>,
{
    fn get(&self, point: [f64; 2usize]) -> f64 {
        self.noise.get(point) * (1. / self.lod.multiplier_i32() as f64)
            + 1.
            + 10. / self.lod.multiplier_i32() as f64
    }
}
