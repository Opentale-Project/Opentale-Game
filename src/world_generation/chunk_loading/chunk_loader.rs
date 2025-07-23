use crate::world_generation::chunk_generation::{CHUNK_SIZE, VOXEL_SIZE};
use crate::world_generation::chunk_loading::chunk_load_cache::ChunkLoadCache;
use crate::world_generation::chunk_loading::chunk_pos::AbsoluteChunkPos;
use crate::world_generation::chunk_loading::chunk_tree::{
    ChunkTree, ChunkTreePos,
};
use crate::world_generation::chunk_loading::lod_position::LodPosition;
use crate::world_generation::chunk_lod::{ChunkLod, MAX_LOD};
use bevy::math::FloatPow;
use bevy::prelude::*;

#[derive(Component, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ChunkLoader {
    pub load_range: i32,
    pub unload_range: i32,
    pub lod_range: [i32; MAX_LOD.usize() - 1],
}

impl Default for ChunkLoader {
    fn default() -> Self {
        Self {
            load_range: 8,
            unload_range: 10,
            lod_range: [2, 4, 8, 16, 32, 64, 128],
        }
    }
}

impl ChunkLoader {
    pub fn get_min_lod_for_chunk(
        &self,
        chunk_pos: AbsoluteChunkPos,
        loader_pos: Vec3,
    ) -> ChunkLod {
        for (i, lod_render_distance) in self.lod_range.iter().enumerate() {
            let render_distance =
                (lod_render_distance * CHUNK_SIZE as i32) as f32 * VOXEL_SIZE;

            if chunk_pos.get_pos_center().distance_squared(loader_pos.xz())
                < render_distance.squared()
            {
                return ChunkLod::from_u8(i as u8 + 1).expect("LOD not found!");
            }
        }

        MAX_LOD
    }

    pub fn get_min_lod(
        &self,
        loader_pos: Vec3,
        lod_pos: LodPosition,
        tree_pos: ChunkTreePos,
    ) -> ChunkLod {
        let chunk_pos = AbsoluteChunkPos::from_absolute(loader_pos);
        let closest = lod_pos.get_closest_chunk_pos(chunk_pos, tree_pos);
        self.get_min_lod_for_chunk(closest, loader_pos)
    }
}

pub fn load_chunks(
    mut chunk_load_cache: ResMut<ChunkLoadCache>,
    mut commands: Commands,
    chunk_loaders: Query<(&ChunkLoader, &Transform)>,
) {
    for (chunk_loader, transform) in &chunk_loaders {
        let tree_pos =
            ChunkTreePos::from_global_pos(transform.translation.xz());

        for x in -chunk_loader.load_range..chunk_loader.load_range + 1 {
            for z in -chunk_loader.load_range..chunk_loader.load_range + 1 {
                let tree_pos = ChunkTreePos::new(*tree_pos + IVec2::new(x, z));

                if !chunk_load_cache.tree_map.contains_key(&tree_pos) {
                    let tree_entity = commands.spawn((
                        ChunkTree { position: tree_pos },
                        Transform::default(),
                        Visibility::Visible,
                    ));
                    chunk_load_cache
                        .tree_map
                        .insert(tree_pos, tree_entity.id());
                }
            }
        }
    }
}

pub fn unload_chunks(
    mut commands: Commands,
    chunk_loaders: Query<(&ChunkLoader, &Transform)>,
    chunk_trees: Query<(Entity, &ChunkTree)>,
    mut chunk_load_cache: ResMut<ChunkLoadCache>,
) {
    for (entity, chunk_parent) in chunk_trees {
        let mut should_unload = true;

        let chunk_position = chunk_parent.position;

        for (chunk_loader, chunk_loader_transform) in &chunk_loaders {
            let loader_chunk_pos = ChunkTreePos::from_global_pos(
                chunk_loader_transform.translation.xz(),
            );
            if (chunk_position.x - loader_chunk_pos.x).abs()
                < chunk_loader.unload_range
                && (chunk_position.y - loader_chunk_pos.y).abs()
                    < chunk_loader.unload_range
            {
                should_unload = false;
                break;
            }
        }

        if !should_unload {
            continue;
        }

        commands.entity(entity).despawn();
        chunk_load_cache.tree_map.remove(&chunk_position);
    }
}

pub fn get_chunk_position(global_position: Vec3, lod: ChunkLod) -> [i32; 2] {
    [
        (global_position.x
            / (CHUNK_SIZE as f32 * VOXEL_SIZE * lod.multiplier_f32()))
        .floor() as i32,
        (global_position.z
            / (CHUNK_SIZE as f32 * VOXEL_SIZE * lod.multiplier_f32()))
        .floor() as i32,
    ]
}
