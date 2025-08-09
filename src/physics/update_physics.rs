use bevy::prelude::*;
use itertools::Itertools;

use crate::physics::{
    collider::Collider,
    collider_trait::ColliderTrait,
    physics_object::{DynamicPhysicsObject, StaticPhysicsObject},
    physics_position::PhysicsPosition,
};

pub fn update_physics(
    static_objects: Query<(&StaticPhysicsObject, &Collider, &Transform)>,
    dynamic_objects: Query<(
        &Collider,
        &mut PhysicsPosition,
        &mut DynamicPhysicsObject,
    )>,
    time: Res<Time>,
) {
    let all_colliders = static_objects
        .iter()
        .map(|s| (s.1, s.2.translation))
        .collect_vec();

    for (dynamic_collider, mut dynamic_position, mut dynamic_object) in
        dynamic_objects
    {
        dynamic_object.touching_sides = IVec3::ZERO;

        let velocity = dynamic_position.velocity * time.delta_secs();

        let new_position = dynamic_position.position + velocity;

        let colliding_statics = static_objects.iter().filter(|so| {
            dynamic_collider.is_colliding_with(
                new_position,
                so.1,
                so.2.translation,
            )
        });

        let mut new_pos = velocity + dynamic_position.position;

        for (_, static_collider, static_transform) in colliding_statics
            .sorted_by(|a, b| {
                a.2.translation
                    .distance_squared(dynamic_position.position)
                    .total_cmp(
                        &b.2.translation
                            .distance_squared(dynamic_position.position),
                    )
            })
        {
            new_pos = dynamic_collider.restrict_movement(
                new_pos,
                static_collider,
                static_transform.translation,
                &all_colliders,
                dynamic_object.step_height,
                &mut dynamic_object.touching_sides,
            );
        }

        dynamic_position.lerp_progress = 0.;
        dynamic_position.previous_position = dynamic_position.position;
        dynamic_position.position = new_pos;
    }
}
