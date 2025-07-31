use crate::player::player_camera_movement::move_camera;
use crate::player::player_component::spawn_player;
use crate::player::player_movement::{move_body, movement};
use crate::player::player_state::PlayerState;
use crate::ui::main_menu_state::MainMenuState;
use avian3d::prelude::PhysicsSchedule;
use bevy::prelude::*;
use bevy_tnua::TnuaUserControlsSystemSet;
use bevy_tnua::prelude::TnuaControllerPlugin;
use bevy_tnua_avian3d::TnuaAvian3dPlugin;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<PlayerState>()
            .add_plugins((
                TnuaControllerPlugin::new(PhysicsSchedule),
                TnuaAvian3dPlugin::new(PhysicsSchedule),
            ))
            .add_systems(Update, (move_camera, move_body))
            .add_systems(
                PhysicsSchedule,
                movement.in_set(TnuaUserControlsSystemSet),
            )
            .add_systems(OnEnter(MainMenuState::Hidden), spawn_player);
    }
}
