use bevy::{
    prelude::*,
    tasks::{Task, TaskPool, TaskPoolBuilder},
};
use futures_lite::future;

use crate::world_generation::chunk_generation::country::{
    country_cache::{CountryCache, GenerationState},
    country_data::CountryData,
};

#[derive(Component)]
pub struct CacheGenerationTask(pub Task<CountryData>);

#[derive(Resource)]
pub struct CacheTaskPool {
    pub task_pool: TaskPool,
}

impl Default for CacheTaskPool {
    fn default() -> Self {
        Self {
            task_pool: TaskPoolBuilder::new()
                .num_threads(4)
                .stack_size(1_000_000)
                .build(),
        }
    }
}

pub fn set_generated_caches(
    mut commands: Commands,
    mut chunks: Query<(Entity, &mut CacheGenerationTask)>,
    mut country_cache: ResMut<CountryCache>,
) {
    for (entity, mut task) in &mut chunks {
        if let Some(chunk_task_data_option) =
            future::block_on(future::poll_once(&mut task.0))
        {
            country_cache.country_cache.insert(
                chunk_task_data_option.country_pos,
                GenerationState::Some(chunk_task_data_option),
            );
            commands.entity(entity).despawn();
        }
    }
}
