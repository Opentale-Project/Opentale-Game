use bevy::prelude::*;

use crate::world_generation::chunk_loading::chunk_tree::init_chunk_trees;

pub struct ChunkLoaderPlugin;

impl Plugin for ChunkLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, init_chunk_trees);
    }
}
