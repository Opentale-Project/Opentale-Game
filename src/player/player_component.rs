use bevy::{
    core_pipeline::{bloom::Bloom, tonemapping::Tonemapping},
    pbr::Atmosphere,
    prelude::*,
    render::{
        camera::Exposure,
        view::{ColorGrading, ColorGradingGlobal},
    },
};
use bevy_panorbit_camera::PanOrbitCamera;

use crate::{
    physics::{
        collider::Collider, physics_object::DynamicPhysicsObject,
        physics_position::PhysicsPosition,
    },
    player::player_state::PlayerState,
    world_generation::chunk_loading::chunk_loader::ChunkLoader,
};

#[derive(Component)]
pub struct Player {
    pub fly: bool,
}

#[derive(Component)]
pub(super) struct PlayerBody;

#[derive(Component)]
pub(super) struct PlayerCamera;

pub(super) fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut player_state: ResMut<NextState<PlayerState>>,
    asset_server: Res<AssetServer>,
) {
    player_state.set(PlayerState::Spawend);

    // Player
    commands.spawn((
        DynamicPhysicsObject {
            step_height: 1.1,
            ..Default::default()
        },
        PhysicsPosition {
            position: Vec3::new(0., 2200., 0.),
            ..Default::default()
        },
        Transform::from_xyz(0., 2200., 0.),
        Collider::aabb(Vec3::new(0.8, 1.8, 0.8), Vec3::ZERO),
        Player { fly: false },
        ChunkLoader::default(),
        Name::new("Player"),
    ));

    commands.spawn((
        Camera3d::default(),
        Camera {
            hdr: true,
            ..Default::default()
        },
        ColorGrading {
            global: ColorGradingGlobal {
                post_saturation: 1.15,
                ..Default::default()
            },
            ..Default::default()
        },
        Msaa::Sample4,
        Transform::from_xyz(-4.0, 6.5, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        Projection::Perspective(PerspectiveProjection {
            far: 2f32.powi(20),
            ..default()
        }),
        Exposure::SUNLIGHT,
        Tonemapping::TonyMcMapface,
        Bloom::NATURAL,
        PanOrbitCamera::default(),
        Atmosphere::EARTH,
        PlayerCamera,
        Name::new("PlayerCamera"),
    ));

    commands
        .spawn((PlayerBody, Name::new("PlayerBody"), Mesh3d::default()))
        .with_children(|commands| {
            commands.spawn((
                SceneRoot(asset_server.load("player.gltf#Scene0")),
                Transform::from_xyz(0., 0.15, 0.),
                Name::new("PlayerHead"),
            ));
            commands.spawn((
                Mesh3d(meshes.add(Mesh::from(Capsule3d {
                    radius: 0.4,
                    half_length: 0.3,
                    ..default()
                }))),
                MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
                Transform::from_xyz(0., -0.35, 0.),
                Name::new("PlayerTorso"),
            ));
        });
}
