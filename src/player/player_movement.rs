use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;

use crate::{
    physics::{
        physics_object::DynamicPhysicsObject, physics_position::PhysicsPosition,
    },
    player::player_component::{Player, PlayerBody, PlayerCamera},
};

pub(super) fn movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut players: Query<(
        &mut Player,
        &mut PhysicsPosition,
        &mut DynamicPhysicsObject,
        &mut Transform,
    )>,
    player_camera: Query<&PanOrbitCamera, With<PlayerCamera>>,
    time: Res<Time>,
) {
    for (
        mut player,
        mut physics_position,
        mut physics_object,
        mut player_transform,
    ) in &mut players
    {
        let mut move_direction = Vec3::ZERO;

        let grounded = physics_object.touching_sides.y < 0;

        if keyboard_input.just_pressed(KeyCode::KeyF) {
            player.fly = !player.fly;
        }

        if grounded || player.fly {
            physics_position.velocity.y = 0.;
        }

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

        if player.fly && keyboard_input.pressed(KeyCode::KeyE) {
            move_direction.y += 1.;
        }
        if player.fly && keyboard_input.pressed(KeyCode::KeyQ) {
            move_direction.y -= 1.;
        }

        let mut movement_speed = if keyboard_input.pressed(KeyCode::ShiftLeft) {
            15.
        } else {
            7.5
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

            if move_direction.xz() != Vec2::ZERO {
                player_transform.rotation = Quat::from_rotation_y(
                    -move_direction.xz().to_angle() - PI * 0.5,
                );
            }
        }

        // Jump if space pressed and the player is close enough to the ground
        if !player.fly && grounded && keyboard_input.pressed(KeyCode::Space) {
            physics_position.velocity.y += 10.;
            physics_object.touching_sides.y = 0;
        }

        physics_position.velocity.x = move_direction.x;
        physics_position.velocity.z = move_direction.z;
        if !player.fly {
            if grounded {
                physics_position.velocity.y -= 1. * time.delta_secs();
            } else {
                physics_position.velocity.y -=
                    (20. * time.delta_secs()).max(-60.);
            }
        } else {
            physics_position.velocity.y = move_direction.y;
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

    player_body.translation = player.translation;
    player_body.rotation = player_body.rotation.lerp(player.rotation, 0.25);
}
