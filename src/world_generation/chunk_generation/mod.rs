use crate::debug_tools::debug_resource::OpentaleDebugResource;
use crate::player::player_component::Player;
use crate::world_generation::chunk_generation::voxel_generation::get_terrain_noise;
use crate::world_generation::chunk_loading::chunk_loader_plugin::ChunkLoaderPlugin;
use crate::world_generation::chunk_loading::chunk_tree::ChunkTreePos;
use crate::world_generation::chunk_loading::country_cache::{
    COUNTRY_SIZE, CountryCache,
};
use crate::world_generation::chunk_loading::lod_position::LodPosition;
use crate::world_generation::generation_assets::GenerationAssets;
use crate::world_generation::generation_options::{
    GenerationCacheItem, GenerationOptionsResource, GenerationState,
};
use crate::world_generation::voxel_world::{
    ChunkGenerationResult, ChunkLod, MAX_LOD, QuadTreeVoxelWorld, VoxelWorld,
};
use ::noise::{Add, Constant, NoiseFn};
use bevy::math::Vec3A;
use bevy::prelude::*;
use bevy::tasks::{Task, TaskPool, TaskPoolBuilder};
use bevy_rapier3d::prelude::Collider;
use futures_lite::future;

pub mod mesh_generation;
pub mod noise;
pub mod structures;
pub mod voxel_generation;
pub mod voxel_types;

//pub const LEVEL_OF_DETAIL: i32 = 1;
pub const CHUNK_SIZE: usize = 64;
pub const VOXEL_SIZE: f32 = 1.0;

pub struct ChunkTaskData {
    pub mesh: Mesh,
    pub transform: Transform,
    pub collider: Option<Collider>,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum BlockType {
    Air,
    Stone,
    Grass,
    Path,
    Snow,
}

impl BlockType {
    pub fn get_texture_index(&self) -> IVec2 {
        match self {
            BlockType::Path => IVec2::new(25, 2),
            BlockType::Grass => IVec2::new(29, 18),
            BlockType::Stone => IVec2::new(30, 29),
            BlockType::Snow => IVec2::new(9, 29),
            _ => IVec2::ZERO,
        }
    }

    pub fn get_texture_id(&self) -> u32 {
        match self {
            BlockType::Path => 2,
            BlockType::Grass => 0,
            BlockType::Stone => 1,
            BlockType::Snow => 3,
            _ => 0,
        }
    }
}

pub struct ChunkGenerationPlugin;

pub struct ChunkTaskPool(pub TaskPool);
impl Resource for ChunkTaskPool {}

pub struct CacheTaskPool(pub TaskPool);
impl Resource for CacheTaskPool {}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct ChunkTriangles(pub [u64; MAX_LOD.usize()]);

impl Plugin for ChunkGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ChunkLoaderPlugin)
            //.add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    set_generated_chunks,
                    start_chunk_tasks,
                    set_generated_caches,
                    draw_path_gizmos,
                ),
            )
            .add_systems(Startup, setup_gizmo_settings)
            .insert_resource(QuadTreeVoxelWorld)
            .insert_resource(ChunkTaskPool(
                TaskPoolBuilder::new()
                    .num_threads(16)
                    .stack_size(3_000_000)
                    .build(),
            ))
            .insert_resource(CacheTaskPool(
                TaskPoolBuilder::new()
                    .num_threads(16)
                    .stack_size(3_000_000)
                    .build(),
            ))
            .insert_resource(GenerationOptionsResource::default())
            .insert_resource(ChunkTriangles([0; MAX_LOD.usize()]))
            .register_type::<ChunkTriangles>();
    }
}

#[derive(Component)]
pub struct ChunkGenerationTask(pub Task<ChunkGenerationResult>, pub Entity);

#[derive(Component)]
pub struct CacheGenerationTask(pub Task<CountryCache>);

#[derive(Component)]
pub struct ChunkTaskGenerator(
    pub IVec2,
    pub ChunkLod,
    pub IVec2,
    pub i32,
    pub Entity,
);

#[derive(Component)]
pub struct Chunk {
    pub tree_position: ChunkTreePos,
    pub lod_position: LodPosition,
    pub generate_above: bool,
    pub chunk_height: i32,
}

#[derive(Component)]
pub struct ChunkGenerator(pub [i32; 2]);

fn start_chunk_tasks(
    mut commands: Commands,
    chunk_task_pool: Res<ChunkTaskPool>,
    cache_task_pool: Res<CacheTaskPool>,
    chunk_task_generators: Query<(Entity, &ChunkTaskGenerator)>,
    chunk_tasks: Query<(), With<ChunkGenerationTask>>,
    mut generation_options: ResMut<GenerationOptionsResource>,
) {
    let chunk_task_count = chunk_tasks.iter().count();

    if chunk_task_count >= 20 {
        return;
    }

    let mut current_added_tasks = 0usize;

    let mut chunk_tasks_vec = chunk_task_generators.iter().collect::<Vec<_>>();
    chunk_tasks_vec.sort_by(|a, b| a.1.1.usize().cmp(&b.1.1.usize()));

    for (entity, chunk_task_generator) in chunk_tasks_vec {
        let parent_pos = chunk_task_generator.0;
        let country_pos = (parent_pos.as_vec2()
            / (COUNTRY_SIZE as f32
                / (MAX_LOD.multiplier_f32() * CHUNK_SIZE as f32)))
            .floor()
            .as_ivec2();

        match generation_options.1.get(&country_pos) {
            None => {
                let arc_generation_options = generation_options.0.clone();
                commands.spawn(CacheGenerationTask(cache_task_pool.0.spawn(
                    async move {
                        CountryCache::generate(
                            country_pos,
                            &arc_generation_options,
                        )
                    },
                )));

                generation_options
                    .1
                    .insert(country_pos, GenerationState::Generating);
            }
            Some(country_cache) => match country_cache {
                GenerationState::Generating => {}
                GenerationState::Some(country_cache) => {
                    if let Ok(mut entity) = commands.get_entity(entity) {
                        current_added_tasks += 1;

                        let generation_options = generation_options.0.clone();
                        let chunk_lod = chunk_task_generator.1;
                        let lod_pos = chunk_task_generator.2;
                        let height = chunk_task_generator.3;
                        let country_cache = country_cache.clone();
                        let task = chunk_task_pool.0.spawn(async move {
                            QuadTreeVoxelWorld::generate_chunk(
                                parent_pos,
                                chunk_lod,
                                lod_pos,
                                generation_options,
                                height,
                                &country_cache,
                            )
                        });

                        entity.remove::<ChunkTaskGenerator>().insert(
                            ChunkGenerationTask(task, chunk_task_generator.4),
                        );
                    }
                }
            },
        }

        if chunk_task_count + current_added_tasks >= 5 {
            return;
        }
    }
}

fn set_generated_chunks(
    mut commands: Commands,
    mut chunks: Query<(Entity, &mut ChunkGenerationTask)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunk_triangles: ResMut<ChunkTriangles>,
    generation_assets: Res<GenerationAssets>,
) {
    for (entity, mut task) in &mut chunks {
        if let Some(chunk_generation_result) =
            future::block_on(future::poll_once(&mut task.0))
        {
            if let Ok(mut current_entity) = commands.get_entity(entity) {
                if let Some(chunk_task_data) = chunk_generation_result.task_data
                {
                    let triangle_count =
                        chunk_task_data.mesh.indices().unwrap().len() / 3;
                    let result_lod = chunk_generation_result.lod.usize();
                    chunk_triangles.0[result_lod - 1] += triangle_count as u64;

                    current_entity.remove::<ChunkGenerationTask>().insert((
                        chunk_task_data.transform,
                        Mesh3d(meshes.add(chunk_task_data.mesh)),
                        MeshMaterial3d(generation_assets.material.clone()),
                        Chunk {
                            tree_position: ChunkTreePos::new(
                                chunk_generation_result.parent_pos,
                            ),
                            lod_position: LodPosition {
                                relative_position: chunk_generation_result
                                    .lod_position,
                                lod: chunk_generation_result.lod,
                            },
                            generate_above: chunk_generation_result
                                .generate_above,
                            chunk_height: chunk_generation_result.chunk_height,
                        },
                        //SpawnAnimation::default()
                    ));

                    // TODO: Replace this with the new collision engine, rapier has problems with the Transform not being applied correctly...
                    // if chunk_generation_result.lod == ChunkLod::Full {
                    //     current_entity.insert((
                    //         RigidBody::Fixed,
                    //         chunk_task_data.collider.unwrap(),
                    //     ));
                    // }
                } else {
                    current_entity.despawn();
                }
            }
        }
    }
}

fn set_generated_caches(
    mut commands: Commands,
    mut chunks: Query<(Entity, &mut CacheGenerationTask)>,
    mut generation_options: ResMut<GenerationOptionsResource>,
) {
    for (entity, mut task) in &mut chunks {
        if let Some(chunk_task_data_option) =
            future::block_on(future::poll_once(&mut task.0))
        {
            generation_options.1.insert(
                chunk_task_data_option.country_pos,
                GenerationState::Some(chunk_task_data_option),
            );
            commands.entity(entity).despawn();
        }
    }
}

fn setup_gizmo_settings(mut config: ResMut<GizmoConfigStore>) {
    let (config, ..) = config.config_mut::<DefaultGizmoConfigGroup>();
    config.depth_bias = -1.;
    config.line.width = 4.;
}

fn draw_path_gizmos(
    mut gizmos: Gizmos,
    generation_options: Res<GenerationOptionsResource>,
    players: Query<&Transform, With<Player>>,
    debug_resource: Res<OpentaleDebugResource>,
) {
    if !debug_resource.show_path_debug {
        return;
    }

    let terrain_noise =
        Add::new(get_terrain_noise(&generation_options.0), Constant::new(5.));

    for player in &players {
        let player_country_pos =
            (player.translation / VOXEL_SIZE / COUNTRY_SIZE as f32)
                .floor()
                .as_ivec3();

        let player_voxel_pos =
            (player.translation / VOXEL_SIZE).as_ivec3().xz();

        let Some(country_cache) =
            generation_options.1.get(&player_country_pos.xz())
        else {
            continue;
        };

        let GenerationState::Some(country_cache) = country_cache else {
            continue;
        };
        for path in country_cache
            .this_path_cache
            .paths
            .iter()
            .chain(&country_cache.bottom_path_cache.paths)
            .chain(&country_cache.left_path_cache.paths)
        {
            if !path.is_in_box(
                player_voxel_pos,
                IVec2::ONE * debug_resource.path_show_range,
            ) {
                continue;
            }

            for path_line in &path.lines {
                if !path_line.is_in_box(
                    player_voxel_pos,
                    IVec2::ONE * debug_resource.path_show_range,
                ) {
                    continue;
                }

                let is_in_path =
                    path_line.is_in_box(player_voxel_pos, IVec2::ONE * 5);

                let color = if is_in_path {
                    Color::srgb(229. / 255., 171. / 255., 0.)
                } else {
                    Color::srgb(0., 200. / 255., 0.)
                };

                gizmos.line(
                    Vec3::from((
                        path_line.start.as_vec2(),
                        terrain_noise.get(path_line.start.as_dvec2().to_array())
                            as f32,
                    ))
                    .xzy()
                        * VOXEL_SIZE,
                    Vec3::from((
                        path_line.end.as_vec2(),
                        terrain_noise.get(path_line.end.as_dvec2().to_array())
                            as f32,
                    ))
                    .xzy()
                        * VOXEL_SIZE,
                    color,
                );
                if !is_in_path {
                    continue;
                }
                gizmos.circle(
                    Isometry3d {
                        rotation: Quat::from_rotation_arc(Vec3::Z, Vec3::Y),
                        translation: Vec3A::from((
                            path_line.spline_one,
                            terrain_noise
                                .get(path_line.spline_one.as_dvec2().to_array())
                                as f32,
                        ))
                        .xzy()
                            * VOXEL_SIZE,
                    },
                    debug_resource.path_circle_radius,
                    Color::srgb(0., 200. / 255., 0.),
                );
                gizmos.circle(
                    Isometry3d {
                        rotation: Quat::from_rotation_arc(Vec3::Z, Vec3::Y),
                        translation: Vec3A::from((
                            path_line.spline_two,
                            terrain_noise
                                .get(path_line.spline_two.as_dvec2().to_array())
                                as f32,
                        ))
                        .xzy()
                            * VOXEL_SIZE,
                    },
                    debug_resource.path_circle_radius,
                    Color::srgb(200. / 255., 0., 0.),
                );
                gizmos.circle(
                    Isometry3d {
                        rotation: Quat::from_rotation_arc(Vec3::Z, Vec3::Y),
                        translation: Vec3A::from((
                            path_line.start.as_vec2(),
                            terrain_noise
                                .get(path_line.start.as_dvec2().to_array())
                                as f32,
                        ))
                        .xzy()
                            * VOXEL_SIZE,
                    },
                    debug_resource.path_circle_radius,
                    Color::srgb(0., 200. / 255., 0.),
                );
                gizmos.circle(
                    Isometry3d {
                        rotation: Quat::from_rotation_arc(Vec3::Z, Vec3::Y),
                        translation: Vec3A::from((
                            path_line.end.as_vec2(),
                            terrain_noise
                                .get(path_line.end.as_dvec2().to_array())
                                as f32,
                        ))
                        .xzy()
                            * VOXEL_SIZE,
                    },
                    debug_resource.path_circle_radius,
                    Color::srgb(200. / 255., 0., 0.),
                );

                for i in 1..path_line.sample_points.len() {
                    let start = path_line.sample_points[i - 1];
                    let end = path_line.sample_points[i];
                    gizmos.line(
                        Vec3::from((
                            start.as_vec2(),
                            terrain_noise.get(start.as_dvec2().to_array())
                                as f32,
                        ))
                        .xzy()
                            * VOXEL_SIZE,
                        Vec3::from((
                            end.as_vec2(),
                            terrain_noise.get(end.as_dvec2().to_array()) as f32,
                        ))
                        .xzy()
                            * VOXEL_SIZE,
                        Color::srgb(200. / 255., 0., 0.),
                    );
                }

                let Some((player_pos_on_path, _)) = path_line
                    .closest_point_on_path(player_voxel_pos, IVec2::ONE * 5)
                else {
                    continue;
                };

                gizmos.circle(
                    Isometry3d {
                        rotation: Quat::from_rotation_arc(Vec3::Z, Vec3::Y),
                        translation: Vec3A::from((
                            player_pos_on_path,
                            terrain_noise
                                .get(player_pos_on_path.as_dvec2().to_array())
                                as f32,
                        ))
                        .xzy()
                            * VOXEL_SIZE,
                    },
                    debug_resource.path_circle_radius,
                    Color::srgb(0., 0., 200. / 255.),
                );
                gizmos.circle(
                    Isometry3d {
                        rotation: Quat::from_rotation_arc(Vec3::Z, Vec3::Y),
                        translation: Vec3A::from((
                            player_pos_on_path.as_ivec2().as_vec2()
                                + VOXEL_SIZE,
                            terrain_noise
                                .get(player_pos_on_path.as_dvec2().to_array())
                                as f32,
                        ))
                        .xzy()
                            * VOXEL_SIZE,
                    },
                    debug_resource.path_circle_radius,
                    Color::srgb(0., 200. / 255., 200. / 255.),
                );

                gizmos.circle(
                    Isometry3d {
                        rotation: Quat::from_rotation_arc(Vec3::Z, Vec3::Y),
                        translation: Vec3A::from((
                            player_voxel_pos.as_vec2() + VOXEL_SIZE,
                            terrain_noise
                                .get(player_pos_on_path.as_dvec2().to_array())
                                as f32,
                        ))
                        .xzy()
                            * VOXEL_SIZE,
                    },
                    debug_resource.path_circle_radius,
                    Color::srgb(0., 100. / 255., 200. / 255.),
                );
            }
        }
    }
}
