use bevy::prelude::*;

use crate::world_generation::{
    chunk_generation::{CHUNK_SIZE, country::country_cache::COUNTRY_SIZE},
    chunk_loading::chunk_pos::AbsoluteChunkPos,
};

#[derive(Deref, DerefMut, Clone, Copy, Hash, PartialEq, Eq)]
pub struct CountryPosition(IVec2);

impl CountryPosition {
    pub fn new(vec: IVec2) -> Self {
        Self(vec)
    }

    pub fn from_chunk_pos(chunk_pos: AbsoluteChunkPos) -> Self {
        let country_chunk_size = COUNTRY_SIZE / CHUNK_SIZE;
        Self(
            (chunk_pos.as_vec2() / country_chunk_size as f32)
                .floor()
                .as_ivec2(),
        )
    }
}
