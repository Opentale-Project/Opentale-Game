use bevy::{
    asset::{AssetServer, Assets, Handle, RenderAssetUsages},
    color::Color,
    ecs::{
        resource::Resource,
        system::{Commands, Res, ResMut},
    },
    image::{Image, ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    pbr::{ExtendedMaterial, StandardMaterial},
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    state::state::{NextState, States},
    utils::default,
};
use itertools::Itertools;

use crate::world_generation::array_texture::ArrayTextureMaterial;

#[derive(Resource)]
pub struct BlockTextureAssets {
    pub block_textures: Vec<Handle<Image>>,
}

#[derive(Resource)]
pub struct GenerationAssets {
    pub material:
        Handle<ExtendedMaterial<StandardMaterial, ArrayTextureMaterial>>,
    pub texture_handle: Handle<Image>,
}

impl BlockTextureAssets {
    pub fn is_all_loaded(&self, asset_server: &AssetServer) -> bool {
        self.block_textures
            .iter()
            .all(|t| asset_server.is_loaded(t.id()))
    }
}

pub fn load_block_texture_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut generation_asset_state: ResMut<NextState<GenerationAssetState>>,
) {
    let mut block_textures = Vec::new();

    block_textures.push(asset_server.load("grass_top.png"));
    block_textures.push(asset_server.load("stone.png"));
    block_textures.push(asset_server.load("snow.png"));
    block_textures.push(asset_server.load("sassafras_log.png"));
    block_textures.push(asset_server.load("sassafras_log_top.png"));

    commands.insert_resource(BlockTextureAssets { block_textures });

    generation_asset_state.set(GenerationAssetState::Loading);
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, States, Default)]
pub enum GenerationAssetState {
    #[default]
    Unloaded,
    Loading,
    Loaded,
}

pub fn setup_array_texture(
    mut commands: Commands,
    block_texture_assets: Res<BlockTextureAssets>,
    mut images: ResMut<Assets<Image>>,
    mut generation_asset_state: ResMut<NextState<GenerationAssetState>>,
    mut materials: ResMut<
        Assets<ExtendedMaterial<StandardMaterial, ArrayTextureMaterial>>,
    >,
    asset_server: Res<AssetServer>,
) {
    if !block_texture_assets.is_all_loaded(&asset_server) {
        return;
    }

    let textures = block_texture_assets
        .block_textures
        .iter()
        .map(|bt| images.get(bt).expect("Failed to get image."))
        .collect_vec();

    let mut array_image = Image::new_fill(
        Extent3d {
            width: 32,
            height: 32 * textures.len() as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    for x in 0..32 {
        for y in 0..32 * textures.len() as u32 {
            let image = textures[y as usize / 32];

            let inner_y = y % 32;

            let color = image
                .get_color_at(x, inner_y)
                .expect("Failed to get color.");
            array_image
                .set_color_at(x, y, color)
                .expect("Failed to set color.");
        }
    }

    array_image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..default()
    });

    array_image.reinterpret_stacked_2d_as_array(textures.len() as u32);

    let texture_handle = images.add(array_image);

    commands.insert_resource(GenerationAssets {
        material: materials.add(ExtendedMaterial {
            base: StandardMaterial::from_color(Color::WHITE),
            extension: ArrayTextureMaterial {
                array_texture: texture_handle.clone(),
            },
        }),
        texture_handle,
    });

    generation_asset_state.set(GenerationAssetState::Loaded);
}
