use bevy::{
    asset::{AssetServer, Assets, Handle},
    color::Color,
    ecs::{
        resource::Resource,
        system::{Commands, Res, ResMut},
    },
    image::{
        Image, 
        ImageAddressMode, 
        ImageLoaderSettings, 
        ImageSampler, 
        ImageSamplerDescriptor
    },
    pbr::{ExtendedMaterial, StandardMaterial},
    state::state::{NextState, States},
    utils::default,
};

use crate::world_generation::array_texture::ArrayTextureMaterial;

#[derive(Resource)]
pub struct GenerationAssets {
    pub material: Handle<ExtendedMaterial<
        StandardMaterial, 
        ArrayTextureMaterial
    >>,
    pub texture_handle: Handle<Image>,
}

pub fn load_generation_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ExtendedMaterial<
        StandardMaterial, 
        ArrayTextureMaterial
    >>>,
    mut generation_asset_state: ResMut<NextState<GenerationAssetState>>,
) {
    let texture_handle = asset_server.load_with_settings(
        "array_texture.png", 
        |s: &mut _| {
            *s = ImageLoaderSettings {
                sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
                    // rewriting mode to repeat image,
                    address_mode_u: ImageAddressMode::Repeat,
                    address_mode_v: ImageAddressMode::Repeat,
                    ..default()
                }),
                ..default()
            }
        }
    );

    commands.insert_resource(GenerationAssets {
        material: materials.add(ExtendedMaterial {
            base: StandardMaterial::from_color(Color::WHITE),
            extension: ArrayTextureMaterial {
                array_texture: texture_handle.clone(),
            },
        }),
        texture_handle,
    });

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
    generation_assets: Res<GenerationAssets>,
    mut images: ResMut<Assets<Image>>,
    mut generation_asset_state: ResMut<NextState<GenerationAssetState>>,
    asset_server: Res<AssetServer>,
) {
    if !asset_server
        .load_state(generation_assets.texture_handle.id())
        .is_loaded()
    {
        return;
    }

    let image = images.get_mut(&generation_assets.texture_handle).unwrap();

    let array_layers = 4;
    image.reinterpret_stacked_2d_as_array(array_layers);

    generation_asset_state.set(GenerationAssetState::Loaded);
}
