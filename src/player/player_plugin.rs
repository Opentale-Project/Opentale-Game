use crate::player::player_camera_movement::move_camera;
use crate::player::player_component::spawn_player;
use crate::player::player_movement::{move_body, movement};
use crate::player::player_state::PlayerState;
use crate::ui::main_menu_state::MainMenuState;
use bevy::core_pipeline::experimental::taa::TemporalAntiAliasPlugin;
use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<PlayerState>()
            .add_plugins(TemporalAntiAliasPlugin)
            .add_systems(Update, (movement, move_camera, move_body))
            .add_systems(OnEnter(MainMenuState::Hidden), spawn_player);
    }
}
