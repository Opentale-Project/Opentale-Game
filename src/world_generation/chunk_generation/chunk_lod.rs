use crate::world_generation::chunk_generation::mesh_generation::generate_mesh;
use crate::world_generation::chunk_generation::voxel_generation::generate_voxels;
use crate::world_generation::chunk_generation::{
    CHUNK_SIZE, ChunkTaskData, VOXEL_SIZE,
};
use crate::world_generation::chunk_loading::country_cache::CountryCache;
use crate::world_generation::generation_options::GenerationOptions;
use bevy::math::{IVec3, Vec3};
use bevy::prelude::{IVec2, Resource, Transform};
use bevy_rapier3d::prelude::Collider;
use std::sync::Arc;

use super::chunk_generation::voxel_data::VoxelData;

pub const MAX_LOD: ChunkLod = ChunkLod::OneTwentyEight;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum ChunkLod {
    #[default]
    Full = 1,
    Half = 2,
    Quarter = 3,
    Eighth = 4,
    Sixteenth = 5,
    Thirtytwoth = 6,
    Sixtyfourth = 7,
    OneTwentyEight = 8,
    TwoFiftySix = 9,
}

impl From<ChunkLod> for i32 {
    fn from(value: ChunkLod) -> Self {
        value as Self
    }
}

impl ChunkLod {
    pub const fn usize(self) -> usize {
        self as usize
    }
    pub const fn u32(self) -> u32 {
        self as u32
    }
    pub const fn i32(self) -> i32 {
        self as i32
    }
    pub const fn f32(self) -> f32 {
        self as u8 as f32
    }
    pub const fn f64(self) -> f64 {
        self as u8 as f64
    }
    pub const fn multiplier_i32(self) -> i32 {
        2i32.pow(self as u32 - 1)
    }
    pub const fn multiplier_f32(self) -> f32 {
        self.multiplier_i32() as f32
    }
    pub const fn inverse_multiplier_i32(self) -> i32 {
        2i32.pow(MAX_LOD as u32 - self as u32)
    }
    pub fn previous(self) -> Self {
        ChunkLod::from_u8(self as u8 - 1).expect("Mapping doesn't exist!")
    }

    pub fn from_u8(number: u8) -> Option<Self> {
        match number {
            1 => Some(Self::Full),
            2 => Some(Self::Half),
            3 => Some(Self::Quarter),
            4 => Some(Self::Eighth),
            5 => Some(Self::Sixteenth),
            6 => Some(Self::Thirtytwoth),
            7 => Some(Self::Sixtyfourth),
            8 => Some(Self::OneTwentyEight),
            9 => Some(Self::TwoFiftySix),
            _ => None,
        }
    }
}
