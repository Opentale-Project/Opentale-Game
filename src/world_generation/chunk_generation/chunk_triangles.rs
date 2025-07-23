use bevy::prelude::*;

use crate::world_generation::voxel_world::MAX_LOD;

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct ChunkTriangles(pub [u64; MAX_LOD.usize()]);
