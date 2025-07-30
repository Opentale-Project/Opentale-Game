use bevy::prelude::*;

use crate::world_generation::chunk_loading::chunk_loader::ChunkLoader;

#[derive(Component)]
pub struct InitialChunkLoader;

pub fn spawn_initial_chunk_loader(mut commands: Commands) {
    commands.spawn((
        InitialChunkLoader,
        ChunkLoader {
            load_range: 4,
            ..Default::default()
        },
        Transform::default(),
    ));
}

pub fn remove_initial_chunk_loader(
    mut commands: Commands,
    loaders: Query<Entity, With<InitialChunkLoader>>,
) {
    for entity in loaders {
        commands.entity(entity).despawn();
    }
}
