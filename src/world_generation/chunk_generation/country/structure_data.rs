use std::sync::Arc;

use bevy::prelude::*;
use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::world_generation::{
    chunk_generation::country::{
        country_cache::{COUNTRY_SIZE, CacheStore},
        country_cache_position::CountryPosition,
        generation_cache::GenerationCacheItem,
    },
    generation_options::GenerationOptions,
};

#[derive(Default)]
pub struct StructureData {
    pub city_location: IVec2,
}

impl GenerationCacheItem<CountryPosition> for StructureData {
    fn generate(
        key: CountryPosition,
        generation_options: &GenerationOptions,
        _country_cache: Arc<CacheStore>,
    ) -> Self {
        let mut rng = StdRng::seed_from_u64(if key.x < 0 {
            generation_options.seed.wrapping_sub(key.x.abs() as u64)
        } else {
            generation_options.seed.wrapping_add(key.x.abs() as u64)
        });
        let mut rng = StdRng::seed_from_u64(if key.x < 0 {
            rng.random::<u64>().wrapping_sub(key.y.abs() as u64)
        } else {
            rng.random::<u64>().wrapping_add(key.y.abs() as u64)
        });

        let min_offset = 100i32;

        let city_x =
            rng.random_range(min_offset..COUNTRY_SIZE as i32 - min_offset);
        let city_z =
            rng.random_range(min_offset..COUNTRY_SIZE as i32 - min_offset);

        Self {
            city_location: IVec2::new(city_x, city_z)
                + *key * COUNTRY_SIZE as i32,
        }
    }
}
