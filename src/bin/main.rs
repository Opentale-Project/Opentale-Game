use bevy::pbr::ExtendedMaterial;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy_atmosphere::prelude::AtmospherePlugin;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_panorbit_camera::PanOrbitCameraPlugin;
use bevy_rapier3d::prelude::{NoUserData, RapierPhysicsPlugin};
use opentale::animation::animation_plugin::OpentaleAnimationPlugin;
use opentale::debug_tools::debug_plugin::OpentaleDebugPlugin;
use opentale::player::player_plugin::PlayerPlugin;
use opentale::ui::game_ui_plugin::GameUiPlugin;
use opentale::world_generation::array_texture::ArrayTextureMaterial;
use opentale::world_generation::world_generation_plugin::WorldGenerationPlugin;
use std::f32::consts::PI;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Opentale".into(),
                        present_mode: PresentMode::Immediate,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            PanOrbitCameraPlugin,
            WorldGenerationPlugin,
            AtmospherePlugin,
            RapierPhysicsPlugin::<NoUserData>::default(),
            //RapierDebugRenderPlugin::default(),
            PlayerPlugin,
            WireframePlugin { ..default() },
            OpentaleAnimationPlugin,
            EguiPlugin::default(),
            WorldInspectorPlugin::new(),
            GameUiPlugin,
            OpentaleDebugPlugin,
            MaterialPlugin::<
                ExtendedMaterial<StandardMaterial, ArrayTextureMaterial>,
            >::default(),
        ))
        .add_systems(Startup, setup)
        .insert_resource(WireframeConfig {
            global: false,
            default_color: Color::srgb(1., 0., 0.),
        })
        .run();
}

fn setup(mut commands: Commands, _asset_server: Res<AssetServer>) {
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

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 75f32,
        ..default()
    });
}
