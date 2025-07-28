use std::f32::consts::PI;

use bevy::prelude::*;

pub enum RotationDirection {
    X,
    Y,
    Z,
}

pub fn rotate_around(pos: &Vec3, pivot: &Vec3, angle: f32, direction: &RotationDirection) -> Vec3 {
    let radiens = angle * (PI / 180.);
    let quat = match direction {
        RotationDirection::X => Quat::from_rotation_x(radiens),
        RotationDirection::Y => Quat::from_rotation_y(radiens),
        RotationDirection::Z => Quat::from_rotation_z(radiens),
    };
    quat.mul_vec3(*pos - *pivot) + *pivot
}
