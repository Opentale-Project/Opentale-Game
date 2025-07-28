use bevy::prelude::*;
use itertools::Itertools;

use crate::world_generation::{
    chunk_generation::{
        chunk_generation_result::ChunkGenerationResult,
        chunk_task::{ChunkTask, ChunkTaskPool},
        country::{
            cache_generation_task::CacheTaskPool, country_cache::CountryCache,
            country_cache_position::CountryPosition, country_data::CountryData,
        },
        mesh_generation::generate_mesh,
        voxel_generation::generate_voxels,
    },
    chunk_loading::{chunk_tree::ChunkTreePos, lod_position::LodPosition},
    generation_options::{GenerationOptions, GenerationOptionsResource},
};

#[derive(Component)]
pub struct ChunkStart {
    pub chunk_lod_pos: LodPosition,
    pub chunk_tree_pos: ChunkTreePos,
    pub chunk_stack_offset: i32,
}

pub fn queue_chunk_tasks(
    mut commands: Commands,
    mut country_cache: ResMut<CountryCache>,
    generation_options: Res<GenerationOptionsResource>,
    chunk_starts: Query<(&ChunkStart, Entity)>,
    chunk_tasks: Query<(), With<ChunkTask>>,
    chunk_task_pool: Res<ChunkTaskPool>,
    cache_task_pool: Res<CacheTaskPool>,
) {
    let current_task_count = chunk_tasks.iter().count();

    if current_task_count > 20 {
        return;
    }

    let mut currently_added_tasks = 0;

    for (chunk_start, chunk_entity) in chunk_starts
        .iter()
        .sorted_by(|a, b| a.0.chunk_lod_pos.lod.cmp(&b.0.chunk_lod_pos.lod))
    {
        if currently_added_tasks + current_task_count > 20 {
            return;
        }

        let chunk_pos = chunk_start
            .chunk_lod_pos
            .get_absolute_chunk_pos(chunk_start.chunk_tree_pos);
        let country_pos = CountryPosition::from_chunk_pos(chunk_pos);

        let Some(country_data) = country_cache.get_or_queue(
            &mut commands,
            country_pos,
            &cache_task_pool,
            &generation_options,
        ) else {
            continue;
        };

        currently_added_tasks += 1;
        let generation_options = generation_options.0.clone();
        let lod_pos = chunk_start.chunk_lod_pos;
        let tree_pos = chunk_start.chunk_tree_pos;
        let stack_height = chunk_start.chunk_stack_offset;
        let task = chunk_task_pool.task_pool.spawn(async move {
            generate_chunk(
                lod_pos,
                tree_pos,
                stack_height,
                &generation_options,
                &country_data,
            )
        });

        commands
            .entity(chunk_entity)
            .remove::<ChunkStart>()
            .insert(ChunkTask(task));
    }
}

fn generate_chunk(
    chunk_pos: LodPosition,
    tree_pos: ChunkTreePos,
    stack_height: i32,
    generation_options: &GenerationOptions,
    country_data: &CountryData,
) -> ChunkGenerationResult {
    let absolute_chunk_pos = chunk_pos.get_absolute_chunk_pos(tree_pos);
    let (data, min_height, more) = generate_voxels(
        [absolute_chunk_pos.x, stack_height, absolute_chunk_pos.y],
        generation_options,
        chunk_pos.lod,
        country_data,
    );

    let mesh = generate_mesh(&data, min_height, chunk_pos.lod);

    ChunkGenerationResult {
        mesh: mesh.map(|o| o.0),
        generate_above: more,
        chunk_pos,
        chunk_tree_position: tree_pos,
        chunk_stack_offset: stack_height,
    }
}
