use avian3d::prelude::{CollisionMargin, RigidBody};
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
    mut _chunk_triangles: ResMut<ChunkTriangles>,
    generation_assets: Res<GenerationAssets>,
) {
    for (entity, mut task) in &mut chunks {
        let Some(chunk_generation_result) =
            future::block_on(future::poll_once(&mut task.0))
        else {
            continue;
        };

        let mut current_entity = commands.entity(entity);

        let chunk_pos =
            chunk_generation_result.chunk_pos.get_absolute_chunk_pos(
                chunk_generation_result.chunk_tree_position,
            );

        current_entity.remove::<ChunkTask>().insert((
            Chunk {
                tree_position: chunk_generation_result.chunk_tree_position,
                chunk_height: chunk_generation_result.chunk_stack_offset,
                generate_above: chunk_generation_result.generate_above,
                lod_position: chunk_generation_result.chunk_pos,
            },
            Transform::from_translation(chunk_pos.to_absolute()),
        ));

        if let Some(collider) = chunk_generation_result.mesh_result.collider {
            current_entity.insert((
                collider,
                RigidBody::Static,
                CollisionMargin(0.01),
            ));
        }

        // let triangle_count = mesh.indices().unwrap().len() / 3;
        // let result_lod = chunk_generation_result.chunk_pos.lod.usize();
        // chunk_triangles.0[result_lod - 1] += triangle_count as u64;

        let opaque_mesh = chunk_generation_result.mesh_result.opaque_mesh;
        let transparent_mesh =
            chunk_generation_result.mesh_result.transparent_mesh;

        current_entity.with_children(|child_spawner| {
            if let Some(mesh) = opaque_mesh {
                child_spawner.spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(generation_assets.opaque_material.clone()),
                ));
            }

            if let Some(mesh) = transparent_mesh {
                child_spawner.spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(
                        generation_assets.transparent_material.clone(),
                    ),
                ));
            }
        });
    }
}
