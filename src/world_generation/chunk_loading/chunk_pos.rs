use bevy::prelude::*;

use crate::world_generation::{
    chunk_generation::{CHUNK_SIZE, VOXEL_SIZE},
    chunk_loading::chunk_tree::ChunkTreePos,
    voxel_world::MAX_LOD,
};

#[derive(Deref, DerefMut, Clone, Copy)]
pub struct AbsoluteChunkPos(IVec2);

impl AbsoluteChunkPos {
    pub fn new(pos: IVec2) -> Self {
        Self(pos)
    }

    pub fn get_pos_center(&self) -> Vec2 {
        (self.0 * CHUNK_SIZE as i32 + (CHUNK_SIZE as i32 / 2)).as_vec2()
            * VOXEL_SIZE
    }
}

impl From<IVec2> for AbsoluteChunkPos {
    fn from(value: IVec2) -> Self {
        Self(value)
    }
}

#[derive(Deref, DerefMut, Clone, Copy)]
pub struct RelativeChunkPos(IVec2);

impl RelativeChunkPos {
    pub fn new(pos: IVec2) -> Self {
        Self(pos)
    }

    pub fn to_absolute(&self, tree_pos: ChunkTreePos) -> AbsoluteChunkPos {
        AbsoluteChunkPos::new((**self) + (*tree_pos * MAX_LOD.multiplier_i32()))
    }
}

impl From<IVec2> for RelativeChunkPos {
    fn from(value: IVec2) -> Self {
        Self(value)
    }
}
