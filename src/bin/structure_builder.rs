use std::f32::consts::PI;

use bevy::{
    color::palettes::tailwind::{PINK_100, RED_500},
    image::{ImageAddressMode, ImageLoaderSettings, ImageSampler},
    pbr::{
        ExtendedMaterial,
        wireframe::{WireframeConfig, WireframePlugin},
    },
    picking::pointer::PointerInteraction,
    prelude::*,
    render::primitives::Aabb,
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
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            MeshPickingPlugin,
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
                update_blocks,
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
    texture_resource: Res<TextureResource>,
    mut materials: ResMut<
        Assets<ExtendedMaterial<StandardMaterial, ArrayTextureMaterial>>,
    >,
) {
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
        MeshMaterial3d(materials.add(ExtendedMaterial {
            base: StandardMaterial::from_color(Color::WHITE),
            extension: ArrayTextureMaterial {
                array_texture: texture_resource.texture_handle.clone(),
            },
        })),
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
    mesh_entities: Query<Entity, With<MeshEntity>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    for entity in mesh_entities {
        let Some((generated_mesh, positions, ..)) =
            generate_mesh(&voxel_data.voxel_data, 0, ChunkLod::Full)
        else {
            return;
        };

        commands.entity(entity).insert((
            Mesh3d(meshes.add(generated_mesh)),
            Aabb::enclosing(positions).unwrap(),
        ));
    }
}

fn update_blocks(
    mut voxel_data: ResMut<VoxelDataResource>,
    pointers: Query<&PointerInteraction>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut gizmos: Gizmos,
) {
    for (point, normal) in pointers
        .iter()
        .filter_map(|interaction| interaction.get_nearest_hit())
        .filter_map(|(_entity, hit)| hit.position.zip(hit.normal))
    {
        if mouse.just_released(MouseButton::Left) {
            let next_block_pos = (point + normal * 0.1).floor().as_ivec3();

            if next_block_pos.min_element() < 1
                || next_block_pos.max_element() > 64
            {
                continue;
            }

            voxel_data
                .voxel_data
                .set_block(next_block_pos, BlockType::Stone);
        }

        if mouse.just_released(MouseButton::Right) {
            let current_block_pos = (point - normal * 0.1).floor().as_ivec3();

            if current_block_pos.min_element() < 1
                || current_block_pos.max_element() > 64
            {
                continue;
            }

            voxel_data
                .voxel_data
                .set_block(current_block_pos, BlockType::Air);
        }

        gizmos.sphere(point, 0.05, RED_500);
        gizmos.arrow(point, point + normal.normalize() * 0.5, PINK_100);
    }
}
