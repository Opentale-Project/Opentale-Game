use bevy::math::{IVec3, Vec3};

use crate::physics::{aabb_collider::AabbCollider, collider::Collider};

pub trait ColliderTrait {
    fn is_colliding_with(
        &self,
        self_position: Vec3,
        other_collider: &impl ColliderTrait,
        other_position: Vec3,
    ) -> bool;

    fn restrict_movement(
        &self,
        end_position: Vec3,
        other_collider: &impl ColliderTrait,
        other_position: Vec3,
        other_colliders: &Vec<(&Collider, Vec3)>,
        step_height: f32,
        touching_sides: &mut IVec3,
    ) -> Vec3;

    fn get_aabbs<'a>(&'a self) -> Vec<&'a AabbCollider>;
}
