use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;
use bevy_rapier3d::prelude::*;

use crate::player::player_component::{Player, PlayerBody, PlayerCamera};

pub(super) fn movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut players: Query<(
        &mut KinematicCharacterController,
        &mut Player,
        Option<&KinematicCharacterControllerOutput>,
        &mut Transform,
    )>,
    player_camera: Query<&PanOrbitCamera, With<PlayerCamera>>,
) {
    for (mut controller, mut player, controller_output, mut transform) in
        &mut players
    {
        if keyboard_input.just_pressed(KeyCode::KeyF) {
            player.fly = !player.fly;
        }

        let mut move_direction = Vec3::ZERO;
        let mut last_movement = player.velocity;

        if let Some(controller_output) = controller_output {
            if player.jumped && controller_output.grounded {
                player.jumped = false;
            }
        }

        last_movement.x *= 0.8;
        last_movement.y *= if player.fly { 0.8 } else { 0.98 };
        last_movement.z *= 0.8;

        // Directional movement
        if keyboard_input.pressed(KeyCode::KeyW)
            || keyboard_input.pressed(KeyCode::ArrowUp)
        {
            move_direction.z -= 1.;
        }
        if keyboard_input.pressed(KeyCode::KeyA)
            || keyboard_input.pressed(KeyCode::ArrowLeft)
        {
            move_direction.x -= 1.;
        }
        if keyboard_input.pressed(KeyCode::KeyS)
            || keyboard_input.pressed(KeyCode::ArrowDown)
        {
            move_direction.z += 1.;
        }
        if keyboard_input.pressed(KeyCode::KeyD)
            || keyboard_input.pressed(KeyCode::ArrowRight)
        {
            move_direction.x += 1.;
        }
        if player.fly {
            if keyboard_input.pressed(KeyCode::KeyE) {
                move_direction.y += 1.;
            }
            if keyboard_input.pressed(KeyCode::KeyQ) {
                move_direction.y -= 1.;
            }
        }

        let mut movement_speed = if keyboard_input.pressed(KeyCode::ShiftLeft) {
            2.
        } else {
            1.
        };

        if player.fly {
            movement_speed *= 10.;
        }

        if let Ok(player_camera) = player_camera.single() {
            // Rotate vector to camera
            let rotation =
                Quat::from_rotation_y(player_camera.yaw.unwrap_or(0.));
            move_direction = rotation
                .mul_vec3(move_direction.normalize_or_zero() * movement_speed);
        }

        if !player.fly
            && controller_output.is_some()
            && !controller_output.unwrap().grounded
        {
            move_direction.y -= 0.4;
        }

        move_direction *= time.delta_secs();

        // Jump if space pressed and the player is close enough to the ground
        if keyboard_input.pressed(KeyCode::Space)
            && controller_output.is_some()
            && controller_output.unwrap().grounded
            && !player.jumped
        {
            move_direction.y = 0.1;
            player.jumped = true;
        }

        let movement = move_direction + last_movement;
        controller.translation = Some(movement);
        player.velocity = movement;

        move_direction.y = 0.0;
        if move_direction.max_element() > 0.0
            || move_direction.min_element() < 0.0
        {
            transform.rotation = Quat::from_rotation_y(
                -move_direction.xz().to_angle() - PI / 2.0,
            );
        }
    }
}

pub(super) fn move_body(
    player: Query<&Transform, (With<Player>, Without<PlayerBody>)>,
    mut player_body: Query<&mut Transform, (With<PlayerBody>, Without<Player>)>,
) {
    let (Ok(player), Ok(mut player_body)) =
        (player.single(), player_body.single_mut())
    else {
        return;
    };

    let difference = player.translation - player_body.translation;
    player_body.translation += difference * 0.25;
    player_body.rotation = player_body.rotation.lerp(player.rotation, 0.25);
}
