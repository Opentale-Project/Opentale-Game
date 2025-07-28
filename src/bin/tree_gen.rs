use bevy::core_pipeline::experimental::taa::TemporalAntiAliasing;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::pbr::ExtendedMaterial;
use bevy::prelude::*;
use bevy::render::camera::Exposure;
use bevy::window::PresentMode;
use bevy_atmosphere::plugin::AtmosphereCamera;
use bevy_atmosphere::prelude::AtmospherePlugin;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_rapier3d::prelude::{NoUserData, RapierPhysicsPlugin};
use fastnoise_lite::FastNoiseLite;
use rand::{rng, RngCore};
use spellhaven::animations::AnimationPlugin;
use spellhaven::debug_tools::debug_resource::OpentaleDebugPlugin;
use spellhaven::terrain_material::TerrainMaterial;
use spellhaven::world_generation::chunk_generation::mesh_generation::generate_mesh;
use spellhaven::world_generation::chunk_generation::pine_structure_generator::PineStructureGenerator;
use spellhaven::world_generation::chunk_generation::structure_generator::{
    StructureGenerator, VoxelStructureMetadata,
};
use spellhaven::world_generation::chunk_generation::voxel_types::VoxelData;
use spellhaven::world_generation::chunk_generation::{BlockType, CHUNK_SIZE, VOXEL_SIZE};
use spellhaven::world_generation::voxel_world::ChunkLod;
use std::f32::consts::PI;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Spellhaven".into(),
                    present_mode: PresentMode::Immediate,
                    ..default()
                }),
                ..default()
            }),
            PanOrbitCameraPlugin,
            AtmospherePlugin,
            RapierPhysicsPlugin::<NoUserData>::default(),
            //RapierDebugRenderPlugin::default(),
            WireframePlugin { ..default() },
            AnimationPlugin,
            //BirdCameraPlugin,
            EguiPlugin {
                enable_multipass_for_primary_context: false,
            },
            WorldInspectorPlugin::new(),
            OpentaleDebugPlugin,
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, TerrainMaterial>>::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, rebuild_tree_system)
        .insert_resource(WireframeConfig {
            global: false,
            default_color: Color::srgb(1., 0., 0.),
        })
        .run();
}

#[derive(Component)]
struct TreeGen;

fn setup(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, TerrainMaterial>>>,
) {
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 1000.,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 3.),
            ..default()
        },
        Name::new("Light"),
    ));

    commands.spawn((
        Camera3d::default(),
        Msaa::Off,
        TemporalAntiAliasing::default(),
        Transform::from_xyz(-4.0, 6.5, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        Projection::Perspective(PerspectiveProjection {
            far: 2f32.powi(20),
            ..default()
        }),
        Exposure { ev100: 10f32 },
        PanOrbitCamera::default(),
        AtmosphereCamera::default(),
        Name::new("CAMMIE"),
    ));

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 50f32,
        ..default()
    });

    spawn_mesh(commands, meshes, materials);
}

fn spawn_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, TerrainMaterial>>>,
) {
    let chunks = get_tree_voxel_data();

    for (chunk, chunk_pos) in chunks {
        let mesh = generate_mesh(&chunk, 0, ChunkLod::Full);

        let Some(mesh) = mesh else {
            continue;
        };

        commands.spawn((
            Transform::from_translation(chunk_pos.as_vec3() * CHUNK_SIZE as f32 * VOXEL_SIZE),
            Name::new("Chunk"),
            Mesh3d(meshes.add(mesh.0)),
            MeshMaterial3d(materials.add(ExtendedMaterial {
                base: Color::WHITE.into(),
                extension: TerrainMaterial {
                    chunk_blocks: chunk.array,
                    palette: chunk.palette,
                    chunk_pos: chunk_pos,
                    chunk_lod: ChunkLod::Full.multiplier_i32(),
                    min_chunk_height: chunk_pos.y * CHUNK_SIZE as i32,
                },
            })),
            TreeGen,
        ));
    }
}

fn rebuild_tree_system(
    mut tree_entities: Query<Entity, With<TreeGen>>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, TerrainMaterial>>>,
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
) {
    if !input.pressed(KeyCode::Space) {
        return;
    }

    for entity in &mut tree_entities {
        commands.entity(entity).despawn();
    }

    spawn_mesh(commands, meshes, materials);
}

fn get_tree_voxel_data() -> Vec<(Box<VoxelData>, IVec3)> {
    let mut chunks = vec![
        (Box::new(VoxelData::default()), IVec3::new(0, 0, 0)),
        (Box::new(VoxelData::default()), IVec3::new(0, 0, 1)),
        (Box::new(VoxelData::default()), IVec3::new(0, 1, 0)),
        (Box::new(VoxelData::default()), IVec3::new(0, 1, 1)),
        (Box::new(VoxelData::default()), IVec3::new(1, 0, 0)),
        (Box::new(VoxelData::default()), IVec3::new(1, 0, 1)),
        (Box::new(VoxelData::default()), IVec3::new(1, 1, 0)),
        (Box::new(VoxelData::default()), IVec3::new(1, 1, 1)),
        (Box::new(VoxelData::default()), IVec3::new(0, 2, 0)),
        (Box::new(VoxelData::default()), IVec3::new(0, 2, 1)),
        (Box::new(VoxelData::default()), IVec3::new(1, 2, 0)),
        (Box::new(VoxelData::default()), IVec3::new(1, 2, 1)),
    ];

    let seed = rng().next_u32();

    let mut noise = FastNoiseLite::with_seed(seed as i32);
    noise.set_noise_type(Some(fastnoise_lite::NoiseType::Value));
    noise.set_frequency(Some(100.));

    let tree_generator = PineStructureGenerator {
        fixed_structure_metadata: VoxelStructureMetadata {
            debug_rgb_multiplier: [0., 0., 0.],
            generate_debug_blocks: false,
            generation_size: [0, 0],
            grid_offset: [0, 0],
            model_size: [0, 0, 0],
            noise,
        },
    };

    let tree_model = tree_generator.get_structure_model(IVec2::new(0, 0), ChunkLod::Full);

    for (chunk, chunk_pos) in &mut chunks {
        apply_trees(chunk, *chunk_pos, &tree_model);
    }

    chunks
}

fn apply_trees(
    blocks: &mut VoxelData,
    chunk_position: IVec3,
    tree_model: &Vec<Vec<Vec<BlockType>>>,
) {
    let chunk_x = chunk_position.x * CHUNK_SIZE as i32;
    let chunk_y = chunk_position.y * CHUNK_SIZE as i32;
    let chunk_z = chunk_position.z * CHUNK_SIZE as i32;

    for x in chunk_x - 1..chunk_x + CHUNK_SIZE as i32 + 1 {
        if x < 0 {
            continue;
        }

        let x = x as usize;

        if x >= tree_model.len() {
            break;
        }

        let tree_model_x = &tree_model[x];
        for y in chunk_y - 1..chunk_y + CHUNK_SIZE as i32 + 1 {
            if y < 0 {
                continue;
            }

            let y = y as usize;

            if y >= tree_model_x.len() {
                break;
            }

            let tree_model_y = &tree_model_x[y];
            for z in chunk_z - 1..chunk_z + CHUNK_SIZE as i32 + 1 {
                if z < 0 {
                    continue;
                }

                let z = z as usize;

                if z >= tree_model_y.len() {
                    break;
                }

                let tree_model_block = tree_model_y[z];

                let chunk_x = x as i32 - chunk_x;
                let chunk_y = y as i32 - chunk_y;
                let chunk_z = z as i32 - chunk_z;

                blocks.set_block(
                    IVec3::new(chunk_x + 1, chunk_y + 1, chunk_z + 1),
                    tree_model_block,
                );
            }
        }
    }
}
