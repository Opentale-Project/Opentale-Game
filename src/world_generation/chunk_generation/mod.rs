pub mod block_type;
pub mod chunk;
pub mod chunk_generation_plugin;
pub mod chunk_generation_result;
pub mod chunk_lod;
pub mod chunk_start;
pub mod chunk_task;
pub mod chunk_triangles;
pub mod country;
pub mod mesh_generation;
pub mod noise;
pub mod structures;
pub mod voxel_data;
pub mod voxel_generation;

//pub const LEVEL_OF_DETAIL: i32 = 1;
pub const CHUNK_SIZE: usize = 64;
pub const VOXEL_SIZE: f32 = 1.0;
