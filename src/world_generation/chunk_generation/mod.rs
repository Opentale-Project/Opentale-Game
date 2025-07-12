use crate::debug_tools::debug_resource::OpentaleDebugResource;
use crate::player::player_component::Player;
use crate::world_generation::chunk_generation::voxel_generation::get_terrain_noise;
use crate::world_generation::chunk_loading::chunk_loader::{
    get_chunk_position, ChunkLoader, ChunkLoaderPlugin,
};
use crate::world_generation::chunk_loading::country_cache::{CountryCache, COUNTRY_SIZE};
use crate::world_generation::chunk_loading::quad_tree_data::QuadTreeNode;
use crate::world_generation::chunk_loading::quad_tree_data::QuadTreeNode::{Data, Node};
use crate::world_generation::generation_assets::GenerationAssets;
use crate::world_generation::generation_options::{
    GenerationCacheItem, GenerationOptionsResource, GenerationState,
};
use crate::world_generation::voxel_world::{
    ChunkGenerationResult, ChunkLod, QuadTreeVoxelWorld, VoxelWorld, MAX_LOD,
};
use ::noise::{Add, Constant, NoiseFn};
use bevy::math::Vec3A;
use bevy::prelude::*;
use bevy::tasks::{Task, TaskPool, TaskPoolBuilder};
use bevy_rapier3d::prelude::{Collider, RigidBody};
use futures_lite::future;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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
            .add_systems(
                Update,
                start_generating_quadtree_chunks.after(upgrade_quad_trees),
            )
            .add_systems(Update, upgrade_quad_trees.after(set_generated_chunks))
            .add_systems(Startup, setup_gizmo_settings)
            .insert_resource(QuadTreeVoxelWorld::default())
            .insert_resource(ChunkTaskPool(
                TaskPoolBuilder::new()
                    .num_threads(2)
                    .stack_size(3_000_000)
                    .build(),
            ))
            .insert_resource(CacheTaskPool(
                TaskPoolBuilder::new()
                    .num_threads(2)
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
    pub Entity
);

#[derive(Component)]
pub struct Chunk(pub [i32; 3]);

#[derive(Component, Reflect)]
pub struct ChunkParent(pub [i32; 2]);

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

    if chunk_task_count >= 5 {
        return;
    }

    let mut current_added_tasks = 0usize;

    let mut chunk_tasks_vec = chunk_task_generators.iter().collect::<Vec<_>>();
    chunk_tasks_vec.sort_by(|a, b| a.1 .1.usize().cmp(&b.1 .1.usize()));

    for (entity, chunk_task_generator) in chunk_tasks_vec {
        let parent_pos = chunk_task_generator.0;
        let country_pos = (parent_pos.as_vec2()
            / (COUNTRY_SIZE as f32 / (MAX_LOD.multiplier_f32() * CHUNK_SIZE as f32)))
            .floor()
            .as_ivec2();

        match generation_options.1.get(&country_pos) {
            None => {
                let arc_generation_options = generation_options.0.clone();
                commands.spawn(
                    CacheGenerationTask(cache_task_pool.0.spawn(async move {
                        CountryCache::generate(
                            country_pos, 
                            &arc_generation_options
                        )
                    }
                )));

                generation_options.1
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

                        entity
                            .remove::<ChunkTaskGenerator>()
                            .insert(ChunkGenerationTask(
                                task, 
                                chunk_task_generator.4
                            ));
                    }
                }
            },
        }

        if chunk_task_count + current_added_tasks >= 5 {
            return;
        }
    }
}

fn start_generating_quadtree_chunks(
    mut commands: Commands,
    mut voxel_world: ResMut<QuadTreeVoxelWorld>,
    chunk_generators: Query<(Entity, &ChunkGenerator)>,
    chunk_loaders: Query<(&ChunkLoader, &Transform)>,
) {
    for (entity, chunk_generator) in &chunk_generators {
        match voxel_world.get_chunk(chunk_generator.0) {
            None => {}
            Some(chunk_tree) => {
                let tree = generate_quad_tree_chunk(
                    entity,
                    MAX_LOD,
                    [0, 0],
                    chunk_generator.0,
                    &chunk_loaders,
                    &mut commands,
                    vec![],
                );

                **chunk_tree = Some(tree);

                commands.entity(entity).insert(Name::new(
                    "Chunk [".to_owned()
                        + &chunk_generator.0[0].to_string()
                        + ", "
                        + &chunk_generator.0[1].to_string()
                        + "]",
                ));

                commands.entity(entity).remove::<ChunkGenerator>();
            }
        }
    }
}

fn generate_quad_tree_chunk(
    owner: Entity,
    current_lod: ChunkLod,
    current_lod_pos: [i32; 2],
    owner_chunk_pos: [i32; 2],
    chunk_loaders: &Query<(&ChunkLoader, &Transform)>,
    commands: &mut Commands,
    despawn_entities: Vec<Entity>,
) -> QuadTreeNode<HashMap<i32, Entity>> {
    let mut divide = false;

    if current_lod != ChunkLod::Full {
        for (chunk_loader, transform) in chunk_loaders {
            let loader_chunk_position = get_chunk_position(
                transform.translation, 
                current_lod
            );
            let current_chunk_pos = [
                owner_chunk_pos[0] 
                    * current_lod.inverse_multiplier_i32() 
                    + current_lod_pos[0],
                owner_chunk_pos[1] 
                    * current_lod.inverse_multiplier_i32() 
                    + current_lod_pos[1],
            ];
            let position_difference = [
                loader_chunk_position[0] - current_chunk_pos[0],
                loader_chunk_position[1] - current_chunk_pos[1],
            ];
            let current_range = 
                chunk_loader.lod_range[MAX_LOD.usize() - current_lod.usize()];
            if position_difference[0].abs() <= current_range
                && position_difference[1].abs() <= current_range
            {
                divide = true;
            }
        }
    }

    if divide {
        return Node{
            bottom_left: Box::new(generate_quad_tree_chunk(
                owner,
                current_lod.previous(),
                [current_lod_pos[0] * 2, current_lod_pos[1] * 2],
                owner_chunk_pos,
                chunk_loaders,
                commands,
                vec![],
            )),
            bottom_right: Box::new(generate_quad_tree_chunk(
                owner,
                current_lod.previous(),
                [current_lod_pos[0] * 2 + 1, current_lod_pos[1] * 2],
                owner_chunk_pos,
                chunk_loaders,
                commands,
                vec![],
            )),
            top_left: Box::new(generate_quad_tree_chunk(
                owner,
                current_lod.previous(),
                [current_lod_pos[0] * 2, current_lod_pos[1] * 2 + 1],
                owner_chunk_pos,
                chunk_loaders,
                commands,
                vec![],
            )),
            top_right: Box::new(generate_quad_tree_chunk(
                owner,
                current_lod.previous(),
                [current_lod_pos[0] * 2 + 1, current_lod_pos[1] * 2 + 1],
                owner_chunk_pos,
                chunk_loaders,
                commands,
                vec![],
            )),
            child_count: Arc::new(Mutex::new(0)),
            chunk_entities: despawn_entities,
        };
    }

    let child = commands
        .spawn((
            ChunkTaskGenerator(
                IVec2::from_array(owner_chunk_pos),
                current_lod,
                IVec2::from_array(current_lod_pos),
                0,
                owner,
            ),
            Name::new(format!(
                "SubChunk[lod: {current_lod:?}, pos:{current_lod_pos:?}]"
            )),
            Visibility::Visible,
        ))
        .id();

    commands
        .entity(owner)
        .insert((Transform::default(), Visibility::default()));

    commands.entity(owner).add_child(child);

    let mut map = HashMap::new();
    map.insert(0, child);

    Data(map, despawn_entities)
}

pub(crate) fn upgrade_quad_trees(
    mut commands: Commands,
    mut voxel_world: ResMut<QuadTreeVoxelWorld>,
    chunks: Query<(Entity, &ChunkParent)>,
    chunk_loaders: Query<(&ChunkLoader, &Transform)>,
    generated_chunks: Query<Entity, With<Chunk>>,
) {
    for chunk in &chunks {
        let boxed_tree = voxel_world
            .get_chunk(chunk.1.0)
            .expect("Chunk not found!");

        match &**boxed_tree {
            None => {}
            Some(chunk_tree) => {
                let tree = upgrade_tree_recursion(
                    chunk.0,
                    chunk_tree,
                    MAX_LOD,
                    [0, 0],
                    chunk.1 .0,
                    &chunk_loaders,
                    &mut commands,
                    &generated_chunks,
                );

                **boxed_tree = Some(tree);
            }
        }
    }
}

fn upgrade_tree_recursion(
    owner: Entity,
    current_node: &QuadTreeNode<HashMap<i32, Entity>>,
    current_lod: ChunkLod,
    current_lod_pos: [i32; 2],
    owner_chunk_pos: [i32; 2],
    chunk_loaders: &Query<(&ChunkLoader, &Transform)>,
    commands: &mut Commands,
    generated_chunks: &Query<Entity, With<Chunk>>,
) -> QuadTreeNode<HashMap<i32, Entity>> {
    match current_node {
        Data(children, entities) => {
            let mut divide = false;

            if current_lod != ChunkLod::Full {
                for (chunk_loader, transform) in chunk_loaders {
                    let loader_chunk_position =
                        get_chunk_position(transform.translation, current_lod);
                    let current_chunk_pos = [
                        owner_chunk_pos[0] 
                            * current_lod.inverse_multiplier_i32()
                            + current_lod_pos[0],
                        owner_chunk_pos[1] 
                            * current_lod.inverse_multiplier_i32()
                            + current_lod_pos[1],
                    ];
                    let position_difference = [
                        loader_chunk_position[0] - current_chunk_pos[0],
                        loader_chunk_position[1] - current_chunk_pos[1],
                    ];
                    let current_range = 
                        chunk_loader.lod_range[
                            MAX_LOD.usize() - current_lod.usize()
                        ];
                    if position_difference[0].abs() <= current_range
                        && position_difference[1].abs() <= current_range
                    {
                        divide = true;
                    }
                }
            }

            if !divide {
                return Data(children.clone(), entities.clone());
            }

            let entities = check_entities_for_deletion(
                [
                    children.clone().into_values().collect(), 
                    entities.clone()
                ].concat(),
                commands,
                generated_chunks,
            );

            // TODO: extract this code into a constructor, 
            // so we're not just looking at a massive wall of text  
            Node {
                bottom_left: Box::new(generate_quad_tree_chunk(
                    owner,
                    current_lod.previous(),
                    [current_lod_pos[0] * 2, current_lod_pos[1] * 2],
                    owner_chunk_pos,
                    chunk_loaders,
                    commands,
                    vec![],
                )),
                bottom_right: Box::new(generate_quad_tree_chunk(
                    owner,
                    current_lod.previous(),
                    [current_lod_pos[0] * 2 + 1, current_lod_pos[1] * 2],
                    owner_chunk_pos,
                    chunk_loaders,
                    commands,
                    vec![],
                )),
                top_left: Box::new(generate_quad_tree_chunk(
                    owner,
                    current_lod.previous(),
                    [current_lod_pos[0] * 2, current_lod_pos[1] * 2 + 1],
                    owner_chunk_pos,
                    chunk_loaders,
                    commands,
                    vec![],
                )),
                top_right: Box::new(generate_quad_tree_chunk(
                    owner,
                    current_lod.previous(),
                    [current_lod_pos[0] * 2 + 1, current_lod_pos[1] * 2 + 1],
                    owner_chunk_pos,
                    chunk_loaders,
                    commands,
                    vec![],
                )),
                child_count: Arc::new(Mutex::new(0)),
                chunk_entities: entities,
            }
        }
        Node {
            bottom_left, 
            bottom_right, 
            top_left, 
            top_right, 
            child_count: current_mutex, 
            chunk_entities: current_entities
        } => {
            let mut divide = false;

            if current_lod != ChunkLod::Full {
                for (chunk_loader, transform) in chunk_loaders {
                    let loader_chunk_position =
                        get_chunk_position(transform.translation, current_lod);
                    let current_chunk_pos = [
                        owner_chunk_pos[0] * current_lod.inverse_multiplier_i32()
                            + current_lod_pos[0],
                        owner_chunk_pos[1] * current_lod.inverse_multiplier_i32()
                            + current_lod_pos[1],
                    ];
                    let position_difference = [
                        loader_chunk_position[0] - current_chunk_pos[0],
                        loader_chunk_position[1] - current_chunk_pos[1],
                    ];
                    let current_range =
                        chunk_loader.lod_range[MAX_LOD.usize() - current_lod.usize()];
                    if position_difference[0].abs() <= current_range
                        && position_difference[1].abs() <= current_range
                    {
                        divide = true;
                    }
                }
            }

            if divide {
                return Node {
                    bottom_left: Box::new(upgrade_tree_recursion(
                        owner,
                        &**bottom_left,
                        current_lod.previous(),
                        [current_lod_pos[0] * 2, current_lod_pos[1] * 2],
                        owner_chunk_pos,
                        chunk_loaders,
                        commands,
                        generated_chunks,
                    )),
                    bottom_right: Box::new(upgrade_tree_recursion(
                        owner,
                        &**bottom_right,
                        current_lod.previous(),
                        [current_lod_pos[0] * 2 + 1, current_lod_pos[1] * 2],
                        owner_chunk_pos,
                        chunk_loaders,
                        commands,
                        generated_chunks,
                    )),
                    top_left: Box::new(upgrade_tree_recursion(
                        owner,
                        &**top_left,
                        current_lod.previous(),
                        [current_lod_pos[0] * 2, current_lod_pos[1] * 2 + 1],
                        owner_chunk_pos,
                        chunk_loaders,
                        commands,
                        generated_chunks,
                    )),
                    top_right: Box::new(upgrade_tree_recursion(
                        owner,
                        &**top_right,
                        current_lod.previous(),
                        [current_lod_pos[0] * 2 + 1, current_lod_pos[1] * 2 + 1],
                        owner_chunk_pos,
                        chunk_loaders,
                        commands,
                        generated_chunks,
                    )),
                    child_count: current_mutex.clone(),
                    chunk_entities: current_entities.clone(),
                };
            }

            let entities = [
                get_entities_recursive(&**bottom_left),
                get_entities_recursive(&**bottom_right),
                get_entities_recursive(&**top_left),
                get_entities_recursive(&**top_right),
                current_entities.clone(),
            ]
            .concat();

            let entities = check_entities_for_deletion(entities, commands, generated_chunks);

            generate_quad_tree_chunk(
                owner,
                current_lod,
                current_lod_pos,
                owner_chunk_pos,
                chunk_loaders,
                commands,
                entities,
            )
        }
    }
}

fn check_entities_for_deletion(
    entities: Vec<Entity>,
    commands: &mut Commands,
    generated_chunks: &Query<Entity, With<Chunk>>,
) -> Vec<Entity> {
    let mut new_entities = vec![];

    for entity in entities {
        if generated_chunks.get(entity).is_ok() {
            new_entities.push(entity);
        } else {
            if let Ok(mut entity) = commands.get_entity(entity) {
                entity.despawn();
            } else {
                new_entities.push(entity);
            }
        }
    }

    new_entities
}

fn get_entities_recursive(
    current_node: &QuadTreeNode<HashMap<i32, Entity>>
) -> Vec<Entity> {
    match current_node {
        Data(entities, despawn_children) => [
            entities.clone().into_values().collect(),
            despawn_children.clone(),
        ]
        .concat(),
        Node {
            bottom_left, 
            bottom_right, 
            top_left, 
            top_right: d, 
            chunk_entities: despawn_entities,
            ..
         } => [
            get_entities_recursive(&**bottom_left),
            get_entities_recursive(&**bottom_right),
            get_entities_recursive(&**top_left),
            get_entities_recursive(&**d),
            despawn_entities.clone(),
        ]
        .concat(),
    }
}

fn set_generated_chunks(
    mut commands: Commands,
    mut chunks: Query<(Entity, &mut ChunkGenerationTask)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut voxel_world: ResMut<QuadTreeVoxelWorld>,
    mut chunk_triangles: ResMut<ChunkTriangles>,
    generation_assets: Res<GenerationAssets>,
) {
    for (entity, mut task) in &mut chunks {
        if let Some(
            chunk_generation_result
        ) = future::block_on(
            future::poll_once(&mut task.0)
        ) {
            let tree_depth = <ChunkLod as Into<i32>>::into(MAX_LOD)
                - <ChunkLod as Into<i32>>::into(chunk_generation_result.lod);

            let Some(tree) = voxel_world.get_chunk(
                chunk_generation_result.parent_pos.to_array()
            ) else {
                info!("Owner not found!");
                continue;
            };

            let Some(tree) = tree.as_mut() else {
                info!("Owner not found");
                continue;
            };

            let Some(node) = tree.get_node(
                tree_depth,
                chunk_generation_result.lod_position.to_array()
            ) else {
                info!(
                    "Map not found! depth: {0}, pos: [{1}, {2}]",
                    <ChunkLod as Into<i32>>::into(MAX_LOD)
                        - <ChunkLod as Into<i32>>::into(
                            chunk_generation_result.lod
                        ),
                    chunk_generation_result.lod_position[0],
                    chunk_generation_result.lod_position[1]
                );
                continue;
            };

            if chunk_generation_result.generate_above 
                && let Data(map, _) = node 
            {
                let new_height = chunk_generation_result.chunk_height + 1;

                let child = commands.spawn((
                    ChunkTaskGenerator(
                        chunk_generation_result.parent_pos, 
                        chunk_generation_result.lod, 
                        chunk_generation_result.lod_position, 
                        new_height, 
                        task.1
                    ),
                    Name::new(
                        format!(
                            "SubChunk[lod: {:?}, pos: {:?}, height: {}]", 
                            chunk_generation_result.lod, 
                            chunk_generation_result.lod_position,
                            new_height
                        )
                    ),
                    Visibility::Visible
                )).id();

                commands.entity(task.1).add_child(child);

                map.insert(new_height, child);
            } else if let Data(_, despawn_entities) = node {
                for despawn_entity in despawn_entities.clone() {
                    if let Ok(mut entity) =
                        commands.get_entity(despawn_entity.clone())
                    {
                        entity.despawn();
                    }
                }

                despawn_entities.clear();
            } else {
                tree.add_to_parent(
                    tree_depth,
                    chunk_generation_result.lod_position.to_array(),
                    &mut commands,
                );
            }

            if let Ok(
                mut current_entity
            ) = commands.get_entity(entity) {
                if let Some(
                    chunk_task_data
                ) = chunk_generation_result.task_data {
                    let triangle_count = chunk_task_data.mesh
                        .indices()
                        .unwrap()
                        .len() / 3;
                    let result_lod = chunk_generation_result.lod.usize();
                    chunk_triangles.0[result_lod - 1] +=
                        triangle_count as u64;

                    current_entity.remove::<ChunkGenerationTask>().insert((
                        chunk_task_data.transform,
                        Mesh3d(meshes.add(chunk_task_data.mesh)),
                        MeshMaterial3d(generation_assets.material.clone()),
                        Chunk([
                            chunk_generation_result.parent_pos[0],
                            chunk_generation_result.chunk_height,
                            chunk_generation_result.parent_pos[1],
                        ]),
                        //SpawnAnimation::default()
                    ));

                    if chunk_generation_result.lod == ChunkLod::Full {
                        current_entity.insert((
                            RigidBody::Fixed,
                            chunk_task_data.collider.unwrap(),
                            GlobalTransform::from_translation(
                                chunk_task_data.transform.translation,
                            ),
                        ));
                    }
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
        if let Some(
            chunk_task_data_option
        ) = future::block_on(future::poll_once(&mut task.0)) {
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

    let terrain_noise = Add::new(
        get_terrain_noise(&generation_options.0), 
        Constant::new(5.)
    );

    for player in &players {
        let player_country_pos = (
            player.translation 
            / VOXEL_SIZE 
            / COUNTRY_SIZE as f32
        ).floor().as_ivec3();

        let player_voxel_pos = (
            player.translation 
            / VOXEL_SIZE
        ).as_ivec3().xz();

        let Some(
            country_cache
        ) = generation_options.1.get(&player_country_pos.xz()) else {
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

                let is_in_path = path_line.is_in_box(
                    player_voxel_pos, 
                    IVec2::ONE * 5
                );

                let color = if is_in_path {
                    Color::srgb(229. / 255., 171. / 255., 0.)
                } else {
                    Color::srgb(0., 200. / 255., 0.)
                };
                
                gizmos.line(
                    Vec3::from((
                        path_line.start.as_vec2(),
                        terrain_noise.get(
                            path_line.start.as_dvec2().to_array()
                        ) as f32,
                    )).xzy() * VOXEL_SIZE,
                    Vec3::from((
                        path_line.end.as_vec2(),
                        terrain_noise.get(
                            path_line.end.as_dvec2().to_array()
                        ) as f32,
                    )).xzy() * VOXEL_SIZE,
                    color,
                );
                if !is_in_path {
                    continue;
                }
                gizmos.circle(
                    Isometry3d {
                        rotation: Quat::from_rotation_arc(
                            Vec3::Z, 
                            Vec3::Y
                        ),
                        translation: Vec3A::from((
                            path_line.spline_one,
                            terrain_noise.get(
                                path_line
                                    .spline_one
                                    .as_dvec2()
                                    .to_array(),
                            ) as f32,
                        )).xzy() * VOXEL_SIZE,
                    },
                    debug_resource.path_circle_radius,
                    Color::srgb(0., 200. / 255., 0.),
                );
                gizmos.circle(
                    Isometry3d {
                        rotation: Quat::from_rotation_arc(
                            Vec3::Z, 
                            Vec3::Y
                        ),
                        translation: Vec3A::from((
                            path_line.spline_two,
                            terrain_noise.get(
                                path_line
                                    .spline_two
                                    .as_dvec2()
                                    .to_array(),
                            )as f32,
                        )).xzy() * VOXEL_SIZE,
                    },
                    debug_resource.path_circle_radius,
                    Color::srgb(200. / 255., 0., 0.),
                );
                gizmos.circle(
                    Isometry3d {
                        rotation: Quat::from_rotation_arc(
                            Vec3::Z, 
                            Vec3::Y
                        ),
                        translation: Vec3A::from((
                            path_line.start.as_vec2(),
                            terrain_noise.get(
                                path_line
                                    .start
                                    .as_dvec2()
                                    .to_array()
                            )as f32,
                        )).xzy() * VOXEL_SIZE,
                    },
                    debug_resource.path_circle_radius,
                    Color::srgb(0., 200. / 255., 0.),
                );
                gizmos.circle(
                    Isometry3d {
                        rotation: Quat::from_rotation_arc(
                            Vec3::Z, 
                            Vec3::Y
                        ),
                        translation: Vec3A::from((
                            path_line.end.as_vec2(),
                            terrain_noise.get(
                                path_line
                                    .end
                                    .as_dvec2()
                                    .to_array()
                            ) as f32,
                        )).xzy() * VOXEL_SIZE,
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
                            terrain_noise.get(
                                start
                                    .as_dvec2()
                                    .to_array()
                            ) as f32,
                        )).xzy() * VOXEL_SIZE,
                        Vec3::from((
                            end.as_vec2(),
                            terrain_noise.get(
                                end
                                    .as_dvec2()
                                    .to_array()
                            ) as f32,
                        )).xzy() * VOXEL_SIZE,
                        Color::srgb(200. / 255., 0., 0.),
                    );
                }

                let Some(
                    (player_pos_on_path, _)
                ) = path_line.closest_point_on_path(
                    player_voxel_pos, 
                    IVec2::ONE * 5
                ) else {
                    continue;
                };

                gizmos.circle(
                    Isometry3d {
                        rotation: Quat::from_rotation_arc(
                            Vec3::Z,
                            Vec3::Y,
                        ),
                        translation: Vec3A::from((
                            player_pos_on_path,
                            terrain_noise.get(
                                player_pos_on_path
                                    .as_dvec2()
                                    .to_array(),
                            ) as f32,
                        )).xzy() * VOXEL_SIZE,
                    },
                    debug_resource.path_circle_radius,
                    Color::srgb(0., 0., 200. / 255.),
                );
                gizmos.circle(
                    Isometry3d {
                        rotation: Quat::from_rotation_arc(
                            Vec3::Z,
                            Vec3::Y,
                        ),
                        translation: Vec3A::from((
                            player_pos_on_path.as_ivec2().as_vec2()
                                + VOXEL_SIZE,
                            terrain_noise.get(
                                player_pos_on_path
                                    .as_dvec2()
                                    .to_array(),
                            ) as f32,
                        )).xzy() * VOXEL_SIZE,
                    },
                    debug_resource.path_circle_radius,
                    Color::srgb(0., 200. / 255., 200. / 255.),
                );

                gizmos.circle(
                    Isometry3d {
                        rotation: Quat::from_rotation_arc(
                            Vec3::Z,
                            Vec3::Y,
                        ),
                        translation: Vec3A::from((
                            player_voxel_pos.as_vec2() 
                                + VOXEL_SIZE,
                            terrain_noise.get(
                                player_pos_on_path
                                    .as_dvec2()
                                    .to_array(),
                            )as f32,
                        )).xzy() * VOXEL_SIZE,
                    },
                    debug_resource.path_circle_radius,
                    Color::srgb(0., 100. / 255., 200. / 255.),
                );
            }
        }
    }
}
