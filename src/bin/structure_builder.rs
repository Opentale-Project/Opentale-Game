use std::f32::consts::PI;

use bevy::{
    image::{ImageAddressMode, ImageLoaderSettings, ImageSampler},
    pbr::{
        ExtendedMaterial,
        wireframe::{WireframeConfig, WireframePlugin},
    },
    prelude::*,
    window::PresentMode,
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use opentale::{
    debug_tools::debug_plugin::OpentaleDebugPlugin,
    world_generation::{
        array_texture::ArrayTextureMaterial,
        chunk_generation::{
            block_type::BlockType, chunk_lod::ChunkLod,
            mesh_generation::generate_mesh, voxel_data::VoxelData,
        },
    },
};

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
            WireframePlugin::default(),
            EguiPlugin::default(),
            WorldInspectorPlugin::new(),
            OpentaleDebugPlugin,
            MaterialPlugin::<
                ExtendedMaterial<StandardMaterial, ArrayTextureMaterial>,
            >::default(),
        ))
        .add_systems(Startup, setup_texture)
        .add_systems(
            Update,
            (
                remesh.run_if(resource_changed::<VoxelDataResource>),
                setup.run_if(resource_added::<TextureResource>),
            ),
        )
        .insert_resource(WireframeConfig {
            global: false,
            default_color: Color::srgb(1., 0., 0.),
        })
        .init_resource::<VoxelDataResource>()
        .run();
}

#[derive(Resource, Default)]
struct VoxelDataResource {
    voxel_data: VoxelData,
}

#[derive(Resource)]
struct TextureResource {
    texture_handle: Handle<Image>,
}

#[derive(Component)]
struct MeshEntity;

fn setup_texture(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture_handle =
        asset_server.load_with_settings("array_texture.png", |s: &mut _| {
            *s = ImageLoaderSettings {
                sampler: ImageSampler::Descriptor(
                    bevy::image::ImageSamplerDescriptor {
                        // rewriting mode to repeat image,
                        address_mode_u: ImageAddressMode::Repeat,
                        address_mode_v: ImageAddressMode::Repeat,
                        ..default()
                    },
                ),
                ..default()
            }
        });

    commands.insert_resource(TextureResource { texture_handle });
}

fn setup(
    mut commands: Commands,
    mut voxel_data: ResMut<VoxelDataResource>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<
        Assets<ExtendedMaterial<StandardMaterial, ArrayTextureMaterial>>,
    >,
    texture_resource: Res<TextureResource>,
) {
    info!("SETUP");

    let image = images
        .get_mut(texture_resource.texture_handle.id())
        .unwrap();

    let array_layers = 4;
    image.reinterpret_stacked_2d_as_array(array_layers);

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

    commands.spawn((
        MeshEntity,
        Mesh3d::default(),
        MeshMaterial3d(materials.add(ExtendedMaterial {
            base: StandardMaterial::from_color(Color::WHITE),
            extension: ArrayTextureMaterial {
                array_texture: texture_resource.texture_handle.clone(),
            },
        })),
        Transform::default(),
    ));

    commands.spawn((
        Transform::default(),
        PanOrbitCamera {
            radius: Some(10.),
            ..Default::default()
        },
    ));

    voxel_data
        .voxel_data
        .set_block(IVec3::new(1, 1, 1), BlockType::Stone);
}

fn remesh(
    voxel_data: Res<VoxelDataResource>,
    mesh_entities: Query<&mut Mesh3d, With<MeshEntity>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for mut mesh in mesh_entities {
        let Some((generated_mesh, ..)) =
            generate_mesh(&voxel_data.voxel_data, 0, ChunkLod::Full)
        else {
            return;
        };

        mesh.0 = meshes.add(generated_mesh);
    }
}
