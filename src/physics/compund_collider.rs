use bevy::prelude::*;
use itertools::Itertools;

use crate::physics::{
    aabb_collider::AabbCollider, collider::Collider,
    collider_trait::ColliderTrait,
};

pub struct CompoundCollider {
    pub colliders: Vec<AabbCollider>,
    shell: AabbCollider,
}

impl CompoundCollider {
    pub fn new(aabbs: Vec<AabbCollider>) -> Self {
        let starts = aabbs.iter().map(|aabb| aabb.offset);
        let ends = aabbs.iter().map(|aabb| aabb.offset + aabb.size);

        let mut min = Vec3::MAX;
        let mut max = Vec3::MIN;

        for start in starts {
            min = min.min(start);
        }

        for end in ends {
            max = max.max(end);
        }

        Self {
            colliders: aabbs,
            shell: AabbCollider {
                offset: min,
                size: max - min,
            },
        }
    }
}

impl ColliderTrait for CompoundCollider {
    fn is_colliding_with(
        &self,
        self_position: Vec3,
        other_collider: &impl ColliderTrait,
        other_position: Vec3,
    ) -> bool {
        if !self.shell.is_colliding_with(
            self_position,
            other_collider,
            other_position,
        ) {
            return false;
        }

        for self_collider in self.get_aabbs() {
            if self_collider.is_colliding_with(
                self_position,
                other_collider,
                other_position,
            ) {
                return true;
            }
        }

        false
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
        let mut result = end_position;

        for self_collider in
            other_collider.get_aabbs().iter().sorted_by(|aabb1, aabb2| {
                aabb1
                    .distance_squared(other_position, end_position)
                    .total_cmp(
                        &aabb2.distance_squared(other_position, end_position),
                    )
            })
        {
            result = self_collider.restrict_movement(
                end_position,
                other_collider,
                other_position,
                other_colliders,
                step_height,
                touching_sides,
            );
        }

        result
    }

    fn get_aabbs<'a>(&'a self) -> Vec<&'a AabbCollider> {
        self.colliders.iter().collect_vec()
    }
}
