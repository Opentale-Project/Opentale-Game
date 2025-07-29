use std::{
    env,
    f32::consts::PI,
    fs::File,
    io::{Read, Write},
};

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
use bevy_inspector_egui::{
    bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass},
    egui,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use itertools::Itertools;
use opentale::{
    debug_tools::debug_plugin::OpentaleDebugPlugin,
    utils::cartesian_product::cube_cartesian_product,
    world_generation::{
        array_texture::ArrayTextureMaterial,
        chunk_generation::{
            block_type::BlockType, chunk_lod::ChunkLod,
            mesh_generation::generate_mesh,
            structures::structure_model::StructureModel, voxel_data::VoxelData,
        },
    },
};
use ron::ser::PrettyConfig;

const PLACEABLE_BLOCKS: [BlockType; 4] = [
    BlockType::Stone,
    BlockType::Grass,
    BlockType::Log,
    BlockType::Snow,
];

fn main() {
    let args = env::args().collect_vec();
    let open_structure = args.get(1);
    let mut voxel_data = VoxelData::default();
    let mut save_data = SaveData::default();
    if let Some(open_structure) = open_structure {
        let mut file = File::open(format!("assets/{}.ron", open_structure))
            .expect("Could not open file.");

        let mut ron_string = String::new();
        file.read_to_string(&mut ron_string)
            .expect("Could not read file.");

        let structure_model: StructureModel =
            ron::from_str(&ron_string).expect("Could not parse file.");

        for (x, y, z) in
            cube_cartesian_product(0..structure_model.model_size.max_element())
        {
            if structure_model.blocks.len() <= x as usize
                || structure_model.blocks[x as usize].len() <= y as usize
                || structure_model.blocks[x as usize][y as usize].len()
                    <= z as usize
            {
                continue;
            }

            let block =
                structure_model.blocks[x as usize][y as usize][z as usize];

            let block_pos = IVec3::new(x, y, z) + IVec3::ONE;
            voxel_data.set_block(block_pos, block);
        }

        save_data.file_name = open_structure.clone();
    }

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
                remesh
                    .run_if(resource_added::<TextureResource>)
                    .after(setup),
                update_blocks,
            ),
        )
        .add_systems(EguiPrimaryContextPass, render_gui)
        .insert_resource(WireframeConfig {
            global: false,
            default_color: Color::srgb(1., 0., 0.),
        })
        .insert_resource(VoxelDataResource {
            voxel_data,
            selected_block: BlockType::Stone,
        })
        .insert_resource(save_data)
        .run();
}

#[derive(Resource)]
struct VoxelDataResource {
    voxel_data: VoxelData,
    selected_block: BlockType,
}

#[derive(Resource)]
struct TextureResource {
    texture_handle: Handle<Image>,
}

#[derive(Component)]
struct MeshEntity;

#[derive(Resource, Default)]
struct SaveData {
    file_name: String,
}

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

            let block = voxel_data.selected_block;
            voxel_data.voxel_data.set_block(next_block_pos, block);
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

fn render_gui(
    mut contexts: EguiContexts,
    mut save_data: ResMut<SaveData>,
    mut voxel_data_resource: ResMut<VoxelDataResource>,
) -> Result {
    egui::TopBottomPanel::top("top").show(contexts.ctx_mut()?, |ui| {
        ui.horizontal(|ui| {
            for block in PLACEABLE_BLOCKS {
                let mut button = ui.button(format!("{:?}", block));

                if voxel_data_resource.selected_block == block {
                    button = button.highlight();
                }

                if button.clicked() {
                    voxel_data_resource.selected_block = block;
                }
            }
        });
    });

    egui::TopBottomPanel::bottom("structure_builder").show(
        contexts.ctx_mut()?,
        |ui| {
            let filename_label = ui.label("Filename:");
            ui.text_edit_singleline(&mut save_data.file_name)
                .labelled_by(filename_label.id);

            if ui.button("Save").clicked() {
                let filename = save_data.file_name.clone();

                let mut min = IVec3::MAX;
                let mut max = IVec3::MIN;

                for (x, y, z) in cube_cartesian_product(0..66) {
                    let pos = IVec3::new(x, y, z);

                    let block = voxel_data_resource.voxel_data.get_block(pos);

                    if let BlockType::Air = block {
                        continue;
                    }

                    min = min.min(pos);
                    max = max.max(pos);
                }

                let mut structure_model = StructureModel {
                    model_size: max - min + IVec3::ONE,
                    blocks: Vec::with_capacity((max.x - min.x + 1) as usize),
                };

                for x in min.x..=max.x {
                    structure_model
                        .blocks
                        .push(Vec::with_capacity((max.y - min.y + 1) as usize));
                    for y in min.y..=max.y {
                        structure_model.blocks[(x - min.x) as usize].push(
                            Vec::with_capacity((max.z - min.z + 1) as usize),
                        );
                        for z in min.z..=max.z {
                            let block = voxel_data_resource
                                .voxel_data
                                .get_block([x, y, z]);
                            structure_model.blocks[(x - min.x) as usize]
                                [(y - min.y) as usize]
                                .push(block);
                        }
                    }
                }

                let mut file = File::create(format!("assets/{}.ron", filename))
                    .expect("Could not open file.");
                let ron_string = ron::ser::to_string_pretty(
                    &structure_model,
                    PrettyConfig::default(),
                )
                .expect("Could not parse model.");
                file.write_all(ron_string.as_bytes())
                    .expect("Could not write to file.");
                file.flush().expect("Could not flush file.")
            }
        },
    );

    Ok(())
}
