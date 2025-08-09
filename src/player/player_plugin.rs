use bevy::prelude::*;

use crate::{
    physics::physics_set::PhysicsSet,
    player::{
        player_camera_movement::move_camera,
        player_component::spawn_player,
        player_movement::{move_body, movement},
        player_state::PlayerState,
    },
    ui::main_menu_state::MainMenuState,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<PlayerState>()
            .add_systems(Update, (move_camera, move_body))
            .add_systems(OnEnter(MainMenuState::Hidden), spawn_player)
            .add_systems(Update, movement.after(PhysicsSet));
    }
}
