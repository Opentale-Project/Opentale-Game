use bevy::prelude::*;

use crate::world_generation::chunk_loading::{
    chunk_load_cache::ChunkLoadCache,
    chunk_loader::{load_chunks, unload_chunks},
    chunk_node::{
        check_for_division, check_for_merging, check_for_task_spawning,
        stack_chunks, update_added_chunks,
    },
    chunk_tree::init_chunk_trees,
    query_stepper::ChunkNodeQueryStepper,
};

pub struct ChunkLoaderPlugin;

impl Plugin for ChunkLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkLoadCache>()
            .init_resource::<ChunkNodeQueryStepper>()
            .add_systems(
                Update,
                (
                    init_chunk_trees,
                    check_for_task_spawning,
                    check_for_division,
                    check_for_merging,
                    load_chunks,
                    unload_chunks,
                    stack_chunks,
                    update_added_chunks,
                ),
            );
    }
}
