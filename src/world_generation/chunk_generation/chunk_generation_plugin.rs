use bevy::prelude::*;

use crate::world_generation::{
    chunk_generation::{
        chunk_start::queue_chunk_tasks,
        chunk_task::{ChunkTaskPool, set_generated_chunks},
        chunk_triangles::ChunkTriangles,
        country::{
            cache_generation_task::{CacheTaskPool, set_generated_caches},
            country_cache::CountryCache,
        },
    },
    chunk_loading::chunk_loader_plugin::ChunkLoaderPlugin,
    generation_options::GenerationOptionsResource,
};

pub struct ChunkGenerationPlugin;

impl Plugin for ChunkGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ChunkLoaderPlugin)
            .init_resource::<ChunkTriangles>()
            .init_resource::<ChunkTaskPool>()
            .init_resource::<CacheTaskPool>()
            .init_resource::<CountryCache>()
            .register_type::<ChunkTriangles>()
            .add_systems(
                Update,
                (
                    queue_chunk_tasks
                        .run_if(resource_exists::<GenerationOptionsResource>),
                    set_generated_chunks,
                    set_generated_caches,
                ),
            );
    }
}
