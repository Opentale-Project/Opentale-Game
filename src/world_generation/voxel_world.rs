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

use super::chunk_generation::voxel_types::VoxelData;

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

pub struct QuadTreeVoxelWorld;

pub trait VoxelWorld {
    fn generate_chunk(
        chunk_position: IVec2,
        chunk_lod: ChunkLod,
        lod_position: IVec2,
        generation_options: Arc<GenerationOptions>,
        chunk_height: i32,
        country_cache: &CountryCache,
    ) -> ChunkGenerationResult;
}

impl Resource for QuadTreeVoxelWorld {}

pub struct ChunkGenerationResult {
    pub task_data: Option<ChunkTaskData>,
    pub generate_above: bool,
    pub parent_pos: IVec2,
    pub lod: ChunkLod,
    pub lod_position: IVec2,
    pub chunk_height: i32,
    pub voxel_data: VoxelData,
    pub chunk_pos: IVec3,
    pub min_height: i32,
}

impl VoxelWorld for QuadTreeVoxelWorld {
    fn generate_chunk(
        parent_pos: IVec2,
        chunk_lod: ChunkLod,
        lod_position: IVec2,
        generation_options: Arc<GenerationOptions>,
        chunk_height: i32,
        country_cache: &CountryCache,
    ) -> ChunkGenerationResult {
        let new_chunk_pos = [
            parent_pos.x * MAX_LOD.multiplier_i32()
                + lod_position.x * chunk_lod.multiplier_i32(),
            chunk_height,
            parent_pos.y * MAX_LOD.multiplier_i32()
                + lod_position.y * chunk_lod.multiplier_i32(),
        ];

        let (data, min_height, more) = generate_voxels(
            new_chunk_pos,
            &generation_options,
            chunk_lod,
            &country_cache,
        );

        let mesh = generate_mesh(&data, min_height, chunk_lod);

        let chunk_transform_pos = Vec3::new(
            new_chunk_pos[0] as f32 * CHUNK_SIZE as f32 * VOXEL_SIZE,
            0., //chunk_height as f32 * CHUNK_SIZE as f32 * VOXEL_SIZE,
            new_chunk_pos[2] as f32 * CHUNK_SIZE as f32 * VOXEL_SIZE,
        );

        return ChunkGenerationResult {
            task_data: match mesh {
                None => None,
                Some(mesh) => Some(ChunkTaskData {
                    transform: Transform::from_translation(chunk_transform_pos),
                    collider: if chunk_lod == ChunkLod::Full {
                        Some(
                            Collider::trimesh(mesh.1, mesh.2)
                                .expect("Failed to build trimesh"),
                        )
                    } else {
                        None
                    },
                    mesh: mesh.0,
                }),
            },
            generate_above: more,
            parent_pos,
            lod: chunk_lod,
            lod_position,
            chunk_height,
            voxel_data: data,
            chunk_pos: IVec3::from_array(new_chunk_pos),
            min_height,
        };
    }
}
