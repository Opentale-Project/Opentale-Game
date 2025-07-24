use bevy::prelude::*;

use crate::world_generation::{
    chunk_generation::{CHUNK_SIZE, VOXEL_SIZE, chunk_lod::MAX_LOD},
    chunk_loading::{chunk_node::ChunkNode, lod_position::LodPosition},
};

/// Relative position of a Chunk Tree
#[derive(Debug, Clone, Copy, Deref, DerefMut, Default, PartialEq, Eq, Hash)]
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

    pub fn to_global_pos(&self) -> Vec2 {
        self.as_vec2()
            * CHUNK_SIZE as f32
            * VOXEL_SIZE
            * MAX_LOD.multiplier_f32()
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
        let lod_pos = LodPosition::new(MAX_LOD, 0, 0);
        commands.entity(added_chunk_tree_entity).with_child((
            ChunkNode::new(lod_pos, added_chunk_tree.position),
            Transform::default(), // Transform::from_translation(
                                  //     lod_pos
                                  //         .get_absolute(added_chunk_tree.position)
                                  //         .extend(0.)
                                  //         .xzy(),
                                  // ),
        ));
    }
}
