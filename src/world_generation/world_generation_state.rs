use bevy::prelude::*;

use crate::world_generation::chunk_generation::{
    chunk_start::ChunkStart, chunk_task::ChunkTask,
};

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum WorldGenerationState {
    #[default]
    Waiting,
    InitialGeneration,
    Active,
}

pub fn check_world_gen_started(
    chunk_starts: Query<(), With<ChunkStart>>,
    mut world_gen_state: ResMut<NextState<WorldGenerationState>>,
) {
    let chunk_start_count = chunk_starts.iter().count();
    if chunk_start_count > 0 {
        world_gen_state.set(WorldGenerationState::InitialGeneration);
    }
}

pub fn check_world_done_initializing(
    chunk_starts: Query<(), With<ChunkStart>>,
    chunk_tasks: Query<(), With<ChunkTask>>,
    mut world_gen_state: ResMut<NextState<WorldGenerationState>>,
) {
    let chunk_start_count = chunk_starts.iter().count();
    let chunk_task_count = chunk_tasks.iter().count();

    if chunk_start_count == 0 && chunk_task_count == 0 {
        info!("World Ready!");

        world_gen_state.set(WorldGenerationState::Active);
    }
}
