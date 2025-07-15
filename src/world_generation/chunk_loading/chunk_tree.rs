use bevy::prelude::*;

use crate::world_generation::{
    chunk_loading::{chunk_node::ChunkNode, lod_position::LodPosition},
    voxel_world::MAX_LOD,
};

/// Relative position of a Chunk Tree
#[derive(Debug, Clone, Copy, Deref, DerefMut, Default)]
pub struct ChunkTreePos(IVec2);

/// This component is for managing the chunk trees.
/// It as all the Part-Chunks as children, so despawning this will get rid of everything.
#[derive(Component)]
pub struct ChunkTree {
    pub position: ChunkTreePos,
}

pub fn init_chunk_trees(
    mut commands: Commands,
    added_chunk_trees: Query<(&ChunkTree, Entity), Added<ChunkTree>>,
) {
    for (added_chunk_tree, added_chunk_tree_entity) in added_chunk_trees {
        commands
            .entity(added_chunk_tree_entity)
            .with_child(ChunkNode::new(
                LodPosition::new(MAX_LOD, 0, 0),
                added_chunk_tree.position,
            ));
    }
}
