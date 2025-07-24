use bevy::prelude::*;

use crate::world_generation::chunk_generation::chunk_lod::MAX_LOD;

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct ChunkTriangles(pub [u64; MAX_LOD.usize()]);
