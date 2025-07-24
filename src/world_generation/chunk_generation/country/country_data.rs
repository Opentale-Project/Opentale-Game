use std::sync::Arc;

use bevy::prelude::*;

use crate::world_generation::{
    chunk_generation::country::{
        country_cache::CacheStore, country_cache_position::CountryPosition,
        generation_cache::GenerationCacheItem, path_data::PathData,
        structure_data::StructureData,
    },
    generation_options::GenerationOptions,
};

#[derive(Clone)]
pub struct CountryData {
    pub country_pos: CountryPosition,
    pub structure_cache: Arc<StructureData>,
    pub this_path_cache: Arc<PathData>,
    pub bottom_path_cache: Arc<PathData>,
    pub left_path_cache: Arc<PathData>,
}

impl GenerationCacheItem<CountryPosition> for CountryData {
    fn generate(
        key: CountryPosition,
        generation_options: &GenerationOptions,
        cache_store: Arc<CacheStore>,
    ) -> Self {
        Self {
            country_pos: key,
            structure_cache: cache_store
                .clone()
                .structure_cache
                .get_cache_entry(key, generation_options, cache_store.clone()),
            this_path_cache: cache_store.clone().path_cache.get_cache_entry(
                key,
                generation_options,
                cache_store.clone(),
            ),
            bottom_path_cache: cache_store.clone().path_cache.get_cache_entry(
                CountryPosition::new(*key + IVec2::NEG_X),
                generation_options,
                cache_store.clone(),
            ),
            left_path_cache: cache_store.clone().path_cache.get_cache_entry(
                CountryPosition::new(*key + IVec2::NEG_Y),
                generation_options,
                cache_store.clone(),
            ),
        }
    }
}
