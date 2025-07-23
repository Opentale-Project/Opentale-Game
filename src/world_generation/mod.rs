pub mod array_texture;
pub mod chunk_generation;
pub mod chunk_loading;
pub mod generation_assets;
pub mod generation_options;
pub mod texture_loading;

use crate::world_generation::chunk_generation::chunk_generation_plugin::ChunkGenerationPlugin;
use crate::world_generation::generation_assets::{
    GenerationAssetState, load_generation_assets, setup_array_texture,
};
use crate::world_generation::texture_loading::texture_loading;
use bevy::app::{App, Startup, Update};
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::prelude::Plugin;
use bevy::state::app::AppExtStates;
use bevy::state::condition::in_state;
use bevy::state::state::OnEnter;

pub struct WorldGenerationPlugin;

impl Plugin for WorldGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GenerationAssetState>()
            .add_systems(
                OnEnter(GenerationAssetState::Unloaded),
                load_generation_assets,
            )
            .add_systems(
                Update,
                setup_array_texture
                    .run_if(in_state(GenerationAssetState::Loading)),
            )
            .add_systems(Startup, texture_loading)
            .add_plugins(ChunkGenerationPlugin);
    }
}
