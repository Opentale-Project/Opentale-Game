use bevy::prelude::*;

use crate::world_generation::{
    chunk_generation::chunk_generation_plugin::ChunkGenerationPlugin,
    generation_assets::{
        GenerationAssetState, load_block_texture_assets, setup_array_texture,
    },
    world_generation_state::{
        WorldGenerationState, check_world_done_initializing,
        check_world_gen_started,
    },
};

pub struct WorldGenerationPlugin;

impl Plugin for WorldGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GenerationAssetState>()
            .init_state::<WorldGenerationState>()
            .add_systems(
                OnEnter(GenerationAssetState::Unloaded),
                load_block_texture_assets,
            )
            .add_systems(
                Update,
                (
                    setup_array_texture
                        .run_if(in_state(GenerationAssetState::Loading)),
                    check_world_gen_started
                        .run_if(in_state(WorldGenerationState::Waiting)),
                    check_world_done_initializing.run_if(in_state(
                        WorldGenerationState::InitialGeneration,
                    )),
                ),
            )
            .add_plugins(ChunkGenerationPlugin);
    }
}
