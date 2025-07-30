use bevy::prelude::*;
use std::{collections::HashMap, sync::Arc};

use crate::world_generation::{
    chunk_generation::country::{
        cache_generation_task::{CacheGenerationTask, CacheTaskPool},
        country_cache_position::CountryPosition,
        country_data::CountryData,
        generation_cache::{GenerationCache, GenerationCacheItem},
        path_data::PathData,
        structure_data::StructureData,
    },
    generation_options::GenerationOptionsResource,
};

pub const COUNTRY_SIZE: usize = 2usize.pow(13);

#[derive(Resource, Default)]
pub struct CountryCache {
    pub country_cache: HashMap<CountryPosition, GenerationState<CountryData>>,
    pub cache_store: Arc<CacheStore>,
}

#[derive(Default)]
pub struct CacheStore {
    pub path_cache: GenerationCache<CountryPosition, PathData>,
    pub structure_cache: GenerationCache<CountryPosition, StructureData>,
}

pub enum GenerationState<T> {
    Generating,
    Some(T),
}

impl CountryCache {
    pub fn get_or_queue(
        &mut self,
        commands: &mut Commands,
        country_pos: CountryPosition,
        cache_task_pool: &CacheTaskPool,
        generation_options: &GenerationOptionsResource,
    ) -> Option<CountryData> {
        let Some(country_data) = self.country_cache.get(&country_pos) else {
            let cache_store = self.cache_store.clone();
            let generation_options = generation_options.0.clone();
            commands.spawn(CacheGenerationTask(
                cache_task_pool.task_pool.spawn(async move {
                    CountryData::generate(
                        country_pos,
                        &generation_options,
                        cache_store,
                    )
                }),
            ));

            self.country_cache
                .insert(country_pos, GenerationState::Generating);

            return None;
        };

        match country_data {
            GenerationState::Generating => None,
            GenerationState::Some(country_data) => Some(country_data.clone()),
        }
    }
}
