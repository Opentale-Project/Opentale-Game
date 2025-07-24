use bevy::render::mesh::Mesh;

use crate::world_generation::chunk_loading::{
    chunk_tree::ChunkTreePos, lod_position::LodPosition,
};

pub struct ChunkGenerationResult {
    pub mesh: Option<Mesh>,
    pub generate_above: bool,
    pub chunk_pos: LodPosition,
    pub chunk_tree_position: ChunkTreePos,
    pub chunk_stack_offset: i32,
}
