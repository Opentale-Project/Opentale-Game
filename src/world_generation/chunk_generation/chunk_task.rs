use bevy::{
    prelude::*,
    tasks::{Task, TaskPool, TaskPoolBuilder},
};
use futures_lite::future;

use crate::world_generation::{
    chunk_generation::{
        chunk::Chunk, chunk_generation_result::ChunkGenerationResult,
        chunk_triangles::ChunkTriangles,
    },
    generation_assets::GenerationAssets,
};

#[derive(Component)]
pub struct ChunkTask(pub Task<ChunkGenerationResult>);

#[derive(Resource)]
pub struct ChunkTaskPool {
    pub task_pool: TaskPool,
}

impl Default for ChunkTaskPool {
    fn default() -> Self {
        Self {
            task_pool: TaskPoolBuilder::new()
                .num_threads(6)
                .stack_size(1_000_000)
                .build(),
        }
    }
}

pub fn set_generated_chunks(
    mut commands: Commands,
    mut chunks: Query<(Entity, &mut ChunkTask)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunk_triangles: ResMut<ChunkTriangles>,
    generation_assets: Res<GenerationAssets>,
) {
    for (entity, mut task) in &mut chunks {
        let Some(chunk_generation_result) =
            future::block_on(future::poll_once(&mut task.0))
        else {
            continue;
        };

        let mut current_entity = commands.entity(entity);
        current_entity.remove::<ChunkTask>().insert(Chunk {
            tree_position: chunk_generation_result.chunk_tree_position,
            chunk_height: chunk_generation_result.chunk_stack_offset,
            generate_above: chunk_generation_result.generate_above,
            lod_position: chunk_generation_result.chunk_pos,
        });

        let Some(mesh) = chunk_generation_result.mesh else {
            continue;
        };

        let triangle_count = mesh.indices().unwrap().len() / 3;
        let result_lod = chunk_generation_result.chunk_pos.lod.usize();
        chunk_triangles.0[result_lod - 1] += triangle_count as u64;

        let chunk_pos =
            chunk_generation_result.chunk_pos.get_absolute_chunk_pos(
                chunk_generation_result.chunk_tree_position,
            );

        current_entity.insert((
            Transform::from_translation(chunk_pos.to_absolute()),
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(generation_assets.material.clone()),
        ));
    }
}
