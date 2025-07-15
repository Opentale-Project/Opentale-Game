use bevy::prelude::*;

use crate::world_generation::chunk_loading::{
    chunk_loader::{load_chunks, unload_chunks},
    chunk_node::{recurse_chunk_nodes, update_added_chunks},
    chunk_tree::init_chunk_trees,
};

pub struct ChunkLoaderPlugin;

impl Plugin for ChunkLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                init_chunk_trees,
                recurse_chunk_nodes,
                update_added_chunks,
                load_chunks,
                unload_chunks,
            ),
        );
    }
}
