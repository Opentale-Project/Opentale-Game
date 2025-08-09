use bevy::prelude::*;
use itertools::Itertools;

use crate::physics::{
    aabb_collider::AabbCollider, collider_trait::ColliderTrait,
    compund_collider::CompoundCollider,
};

#[derive(Component)]
pub enum Collider {
    Aabb(AabbCollider),
    Compound(CompoundCollider),
}

impl Collider {
    pub fn aabb(size: Vec3, offset: Vec3) -> Self {
        Self::Aabb(AabbCollider { size, offset })
    }

    pub fn compund(bounds: &[(Vec3, Vec3)]) -> Self {
        Self::Compound(CompoundCollider::new(
            bounds
                .iter()
                .map(|b| AabbCollider {
                    size: b.0,
                    offset: b.1,
                })
                .collect_vec(),
        ))
    }
}

impl ColliderTrait for Collider {
    fn is_colliding_with(
        &self,
        self_position: Vec3,
        other_collider: &impl ColliderTrait,
        other_position: Vec3,
    ) -> bool {
        match self {
            Collider::Aabb(aabb_collider) => aabb_collider.is_colliding_with(
                self_position,
                other_collider,
                other_position,
            ),
            Collider::Compound(compound_collider) => compound_collider
                .is_colliding_with(
                    self_position,
                    other_collider,
                    other_position,
                ),
        }
    }

    fn restrict_movement(
        &self,
        end_position: Vec3,
        other_collider: &impl ColliderTrait,
        other_position: Vec3,
        other_colliders: &Vec<(&Collider, Vec3)>,
        step_height: f32,
        touching_sides: &mut IVec3,
    ) -> Vec3 {
        match self {
            Collider::Aabb(aabb_collider) => aabb_collider.restrict_movement(
                end_position,
                other_collider,
                other_position,
                other_colliders,
                step_height,
                touching_sides,
            ),
            Collider::Compound(compound_collider) => compound_collider
                .restrict_movement(
                    end_position,
                    other_collider,
                    other_position,
                    other_colliders,
                    step_height,
                    touching_sides,
                ),
        }
    }

    fn get_aabbs<'a>(&'a self) -> Vec<&'a AabbCollider> {
        match self {
            Collider::Aabb(aabb_collider) => aabb_collider.get_aabbs(),
            Collider::Compound(compound_collider) => {
                compound_collider.get_aabbs()
            }
        }
    }
}
