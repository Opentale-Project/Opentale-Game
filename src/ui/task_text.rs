use bevy::prelude::{Component, Query, Text, With, Without};

use crate::world_generation::chunk_generation::{
    chunk_start::ChunkStart, chunk_task::ChunkTask,
    country::cache_generation_task::CacheGenerationTask,
};

#[derive(Component)]
pub struct CountryTaskText;

#[derive(Component)]
pub struct ChunkTaskText;

pub fn update_task_ui(
    mut country_texts: Query<
        &mut Text,
        (With<CountryTaskText>, Without<ChunkTaskText>),
    >,
    mut chunk_texts: Query<
        &mut Text,
        (With<ChunkTaskText>, Without<CountryTaskText>),
    >,
    chunk_tasks: Query<(), With<ChunkStart>>,
    chunk_task_generators: Query<(), With<ChunkTask>>,
    country_tasks: Query<(), With<CacheGenerationTask>>,
) {
    let country_count = country_tasks.iter().count();
    let chunk_count = chunk_tasks.iter().count();
    let chunk_queue_count = chunk_task_generators.iter().count();

    for mut text in &mut country_texts {
        text.0 = format!("Country Tasks: {:?}", country_count);
    }

    for mut text in &mut chunk_texts {
        text.0 =
            format!("Chunk Tasks: {:?} + {:?}", chunk_count, chunk_queue_count);
    }
}
