use bevy::prelude::*;

use crate::utils::math::lerp_decay;

#[derive(Component, Default)]
pub struct PhysicsPosition {
    pub position: Vec3,
    pub velocity: Vec3,
    pub previous_position: Vec3,
    pub lerp_progress: f32,
}

pub fn update_transform_position(
    positions: Query<(&mut PhysicsPosition, &mut Transform)>,
    time: Res<Time>,
    fixed_time: Res<Time<Fixed>>,
) {
    for (mut physics_position, mut transform) in positions {
        let smooth_pos = physics_position
            .previous_position
            .lerp(physics_position.position, physics_position.lerp_progress);

        transform.translation = lerp_decay(
            transform.translation,
            smooth_pos,
            20.,
            time.delta_secs(),
        );

        let new_lerp_progress = (physics_position.lerp_progress
            + (time.delta_secs() / fixed_time.timestep().as_secs_f32()))
        .min(1.);
        physics_position.lerp_progress = new_lerp_progress;
    }
}
