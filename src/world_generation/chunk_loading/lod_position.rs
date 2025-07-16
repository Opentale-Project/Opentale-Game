use bevy::math::{IVec2, Vec2};

use crate::world_generation::{
    chunk_generation::{CHUNK_SIZE, VOXEL_SIZE},
    chunk_loading::{
        chunk_pos::{AbsoluteChunkPos, RelativeChunkPos},
        chunk_tree::ChunkTreePos,
    },
    voxel_world::ChunkLod,
};

/// This struct represents a position inside a Quad-Tree
/// The position is always relative to the LOD
#[derive(Default, Clone, Copy)]
pub struct LodPosition {
    pub relative_position: IVec2,
    pub lod: ChunkLod,
}

impl LodPosition {
    pub fn new(lod: ChunkLod, x: i32, y: i32) -> Self {
        Self {
            relative_position: IVec2::new(x, y),
            lod,
        }
    }

    pub fn get_absolute(&self, tree_pos: ChunkTreePos) -> Vec2 {
        let chunk_pos = self.get_absolute_chunk_pos(tree_pos);
        (*chunk_pos * CHUNK_SIZE as i32).as_vec2() * VOXEL_SIZE
    }

    pub fn get_relative_chunk_position(&self) -> RelativeChunkPos {
        RelativeChunkPos::new(
            self.relative_position * self.lod.multiplier_i32(),
        )
    }

    pub fn get_absolute_chunk_pos(
        &self,
        tree_pos: ChunkTreePos,
    ) -> AbsoluteChunkPos {
        self.get_relative_chunk_position().to_absolute(tree_pos)
    }

    pub fn to_top_right(&self) -> Self {
        self.to_new_inner(IVec2::ONE)
    }

    pub fn to_top_left(&self) -> Self {
        self.to_new_inner(IVec2::Y)
    }

    pub fn to_bottom_right(&self) -> Self {
        self.to_new_inner(IVec2::X)
    }

    pub fn to_bottom_left(&self) -> Self {
        self.to_new_inner(IVec2::ZERO)
    }

    fn to_new_inner(&self, offset: IVec2) -> Self {
        let new_lod = self.lod.previous();
        let new_pos = self.relative_position * 2 + offset;
        Self {
            relative_position: new_pos,
            lod: new_lod,
        }
    }

    pub fn get_containing_chunk_pos(
        &self,
    ) -> impl Iterator<Item = RelativeChunkPos> + use<> {
        let min = *self.get_relative_chunk_position();
        let max = (*LodPosition::new(
            self.lod,
            self.relative_position.x + 1,
            self.relative_position.y + 1,
        )
        .get_relative_chunk_position())
            - IVec2::ONE;

        (min.x..=max.x).flat_map(move |x| {
            (min.y..=max.y)
                .map(move |y| RelativeChunkPos::new(IVec2::new(x, y)))
        })
    }
}
