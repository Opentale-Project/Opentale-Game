use bevy::prelude::*;

use crate::world_generation::chunk_loading::{
    chunk_tree::ChunkTreePos, lod_position::LodPosition,
};

#[derive(Component)]
pub struct Chunk {
    pub tree_position: ChunkTreePos,
    pub lod_position: LodPosition,
    pub generate_above: bool,
    pub chunk_height: i32,
}
