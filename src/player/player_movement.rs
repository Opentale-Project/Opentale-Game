use std::{cmp::Ordering, f32::consts::PI};

use avian3d::math::{AdjustPrecision, AsF32, Vector3};
use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;
use bevy_tnua::{
    TnuaObstacleRadar,
    builtins::TnuaBuiltinClimb,
    control_helpers::TnuaBlipReuseAvoidance,
    math::Float,
    prelude::{TnuaBuiltinJump, TnuaBuiltinWalk, TnuaController},
    radar_lens::{TnuaBlipSpatialRelation, TnuaRadarLens},
};
use bevy_tnua_avian3d::TnuaSpatialExtAvian3d;

use crate::player::player_component::{Player, PlayerBody, PlayerCamera};

pub(super) fn movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut players: Query<(
        &mut TnuaController,
        &mut Player,
        &TnuaObstacleRadar,
        &mut TnuaBlipReuseAvoidance,
    )>,
    mut player_body: Query<&mut Transform, With<PlayerBody>>,
    player_camera: Query<&PanOrbitCamera, With<PlayerCamera>>,
    spacial_ext: TnuaSpatialExtAvian3d,
) {
    let Ok((
        mut controller,
        mut player,
        obstacle_radar,
        mut blip_reuse_avoidance,
    )) = players.single_mut()
    else {
        return;
    };

    if keyboard_input.just_pressed(KeyCode::KeyF) {
        player.fly = !player.fly;
    }

    let mut move_direction = Vec3::ZERO;

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
    // if player.fly {
    //     if keyboard_input.pressed(KeyCode::KeyE) {
    //         move_direction.y += 1.;
    //     }
    //     if keyboard_input.pressed(KeyCode::KeyQ) {
    //         move_direction.y -= 1.;
    //     }
    // }

    let mut movement_speed = if keyboard_input.pressed(KeyCode::ShiftLeft) {
        2.
    } else {
        1.
    };

    movement_speed *= 10.;

    let Ok(player_camera) = player_camera.single() else {
        return;
    };

    let rotation = Quat::from_rotation_y(player_camera.yaw.unwrap_or(0.));
    move_direction =
        rotation.mul_vec3(move_direction.normalize_or_zero() * movement_speed);

    controller.basis(TnuaBuiltinWalk {
        // The `desired_velocity` determines how the character will move.
        desired_velocity: move_direction.normalize_or_zero() * movement_speed,
        // The `float_height` must be greater (even if by little) from the distance between the
        // character's center and the lowest point of its collider.
        float_height: 0.91,
        cling_distance: 0.25,

        // `TnuaBuiltinWalk` has many other fields for customizing the movement - but they have
        // sensible defaults. Refer to the `TnuaBuiltinWalk`'s documentation to learn what they do.
        ..Default::default()
    });

    if keyboard_input.pressed(KeyCode::Space) {
        controller.action(TnuaBuiltinJump {
            // The height is the only mandatory field of the jump button.
            height: 1.5,
            // `TnuaBuiltinJump` also has customization fields with sensible defaults.
            ..Default::default()
        });
    }

    blip_reuse_avoidance.update(controller.as_ref(), obstacle_radar);

    let radar_lens = TnuaRadarLens::new(obstacle_radar, &spacial_ext);

    let already_climbing_on = controller
        .concrete_action::<TnuaBuiltinClimb>()
        .and_then(|(action, _)| {
            let entity = action
                .climbable_entity
                .filter(|entity| obstacle_radar.has_blip(*entity))?;
            Some((entity, action.clone()))
        });

    for blip in radar_lens.iter_blips() {
        if !blip_reuse_avoidance.should_avoid(blip.entity()) {
            if let Some((climbable_entity, action)) =
                already_climbing_on.as_ref()
            {
                if *climbable_entity != blip.entity() {
                    continue;
                }
                let dot_initiation = move_direction
                    .normalize_or_zero()
                    .dot(action.initiation_direction);
                let initiation_direction = if 0.5 < dot_initiation {
                    action.initiation_direction
                } else {
                    Vector3::ZERO
                };
                if initiation_direction == Vector3::ZERO {
                    let right_left = move_direction.dot(Vector3::X);
                    if 0.5 <= right_left.abs() {
                        continue;
                    }
                }

                let mut action = TnuaBuiltinClimb {
                    climbable_entity: Some(blip.entity()),
                    anchor: blip.closest_point().get(),
                    desired_climb_velocity: 10.
                        * move_direction.dot(Vector3::NEG_Z)
                        * Vector3::Y,
                    initiation_direction,
                    desired_vec_to_anchor: action.desired_vec_to_anchor,
                    desired_forward: action.desired_forward,
                    ..default()
                };

                const LOOK_ABOVE_OR_BELOW: Float = 5.0;
                match action
                    .desired_climb_velocity
                    .dot(Vector3::Y)
                    .partial_cmp(&0.0)
                    .unwrap()
                {
                    Ordering::Less => {
                        if controller.is_airborne().unwrap() {
                            let extent = blip.probe_extent_from_closest_point(
                                -Dir3::Y,
                                LOOK_ABOVE_OR_BELOW,
                            );
                            if extent < 0.9 * LOOK_ABOVE_OR_BELOW {
                                action.hard_stop_down = Some(
                                    blip.closest_point().get()
                                        - extent * Vector3::Y,
                                );
                            }
                        } else {
                            if initiation_direction == Vector3::ZERO {
                                continue;
                            } else {
                                action.desired_climb_velocity = Vector3::ZERO;
                            }
                        }
                    }
                    Ordering::Equal => {}
                    // Climbing up
                    Ordering::Greater => {
                        let extent = blip.probe_extent_from_closest_point(
                            Dir3::Y,
                            LOOK_ABOVE_OR_BELOW,
                        );
                        if extent < 0.9 * LOOK_ABOVE_OR_BELOW {
                            action.hard_stop_up = Some(
                                blip.closest_point().get()
                                    + extent * Vector3::Y,
                            );
                        }
                    }
                }

                controller.action(action);
            } else if let TnuaBlipSpatialRelation::Aeside(blip_direction) =
                blip.spatial_relation(0.5)
            {
                if 0.5
                    < move_direction
                        .normalize_or_zero()
                        .dot(blip_direction.adjust_precision())
                {
                    let direction_to_anchor = -blip
                        .normal_from_closest_point()
                        .reject_from_normalized(Vector3::Y);
                    controller.action(TnuaBuiltinClimb {
                        climbable_entity: Some(blip.entity()),
                        anchor: blip.closest_point().get(),
                        desired_vec_to_anchor: 0.5 * direction_to_anchor,
                        desired_forward: Dir3::new(direction_to_anchor.f32())
                            .ok(),
                        initiation_direction: move_direction
                            .normalize_or_zero(),
                        ..default()
                    });
                }
            }
        }
    }

    let Ok(mut transform) = player_body.single_mut() else {
        return;
    };

    move_direction.y = 0.0;
    if move_direction.max_element() > 0.0 || move_direction.min_element() < 0.0
    {
        transform.rotation =
            Quat::from_rotation_y(-move_direction.xz().to_angle() - PI / 2.0);
    }
}

pub(super) fn move_body(
    player: Query<&Transform, (With<Player>, Without<PlayerBody>)>,
    mut player_body: Query<&mut Transform, (With<PlayerBody>, Without<Player>)>,
    time: Res<Time>,
) {
    let Ok(player) = player.single() else {
        return;
    };
    let Ok(mut player_body) = player_body.single_mut() else {
        return;
    };

    player_body.translation = player_body
        .translation
        .lerp(player.translation, (0.5 * time.delta_secs() * 50.).min(1.));
}
