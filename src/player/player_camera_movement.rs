use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;

use crate::{
    debug_tools::debug_resource::OpentaleDebugResource,
    player::player_component::{Player, PlayerCamera},
};

pub(super) fn move_camera(
    player: Query<&Transform, (With<Player>, Without<PlayerCamera>)>,
    mut camera: Query<&mut PanOrbitCamera, (With<PlayerCamera>, Without<Player>)>,
    options: Res<OpentaleDebugResource>,
) {
    if options.unlock_camera {
        return;
    }

    let (Ok(player), Ok(mut camera)) = (player.single(), camera.single_mut()) else {
        return;
    };

    let camera_position = camera.target_focus;
    let difference = (player.translation + Vec3::Y) - camera_position;
    camera.target_focus += difference * 0.25;
}
