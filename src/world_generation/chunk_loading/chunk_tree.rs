use bevy::prelude::*;

use crate::world_generation::{
    chunk_generation::{CHUNK_SIZE, VOXEL_SIZE},
    chunk_loading::{chunk_node::ChunkNode, lod_position::LodPosition},
    voxel_world::MAX_LOD,
};

/// Relative position of a Chunk Tree
#[derive(Debug, Clone, Copy, Deref, DerefMut, Default, PartialEq, Eq)]
pub struct ChunkTreePos(IVec2);

impl ChunkTreePos {
    pub fn new(pos: IVec2) -> Self {
        Self(pos)
    }

    pub fn from_global_pos(pos: Vec2) -> Self {
        let adjusted_pos =
            pos / (CHUNK_SIZE as f32 * VOXEL_SIZE * MAX_LOD.multiplier_f32());
        Self(adjusted_pos.floor().as_ivec2())
    }
}

impl From<IVec2> for ChunkTreePos {
    fn from(value: IVec2) -> Self {
        Self(value)
    }
}

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
