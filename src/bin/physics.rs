use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use opentale::{
    debug_tools::{
        debug_plugin::OpentaleDebugPlugin, physics_debug::PhysicsDebugResource,
    },
    physics::{
        collider::Collider,
        physics_object::{DynamicPhysicsObject, StaticPhysicsObject},
        physics_plugin::PhysicsPlugin,
        physics_position::PhysicsPosition,
        physics_set::PhysicsSet,
    },
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Opentale - Physics".into(),
                        // present_mode: PresentMode::Immediate,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            PanOrbitCameraPlugin,
            PhysicsPlugin,
            EguiPlugin::default(),
            WorldInspectorPlugin::default(),
            OpentaleDebugPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, move_player.after(PhysicsSet))
        .init_resource::<PhysicsDebugResource>()
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Transform::from_xyz(0., 0., 0.),
        StaticPhysicsObject,
        Collider::compund(&[
            (Vec3::new(5., 0.25, 5.), Vec3::ZERO),
            (Vec3::new(5., 0.25, 5.), Vec3::new(1., 0.25, 1.)),
        ]),
    ));

    commands.spawn((
        DynamicPhysicsObject {
            step_height: 0.6,
            ..Default::default()
        },
        PhysicsPosition {
            position: Vec3::NEG_Z * 2.,
            ..Default::default()
        },
        Collider::aabb(Vec3::ONE, Vec3::ZERO),
    ));

    commands.spawn(PanOrbitCamera::default());
}

fn move_player(
    mut player: Query<&mut PhysicsPosition>,
    input: Res<ButtonInput<KeyCode>>,
) -> Result {
    let mut velocity = Vec3::ZERO;

    if input.pressed(KeyCode::KeyW) {
        velocity += Vec3::Z;
    }
    if input.pressed(KeyCode::KeyS) {
        velocity -= Vec3::Z;
    }

    if input.pressed(KeyCode::KeyA) {
        velocity += Vec3::X;
    }
    if input.pressed(KeyCode::KeyD) {
        velocity -= Vec3::X;
    }

    if input.pressed(KeyCode::KeyE) {
        velocity += Vec3::Y;
    }
    if input.pressed(KeyCode::KeyQ) {
        velocity -= Vec3::Y;
    }

    player.single_mut()?.velocity = velocity;

    Ok(())
}
