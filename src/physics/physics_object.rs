use bevy::prelude::*;

use crate::physics::physics_position::PhysicsPosition;

#[derive(Component)]
pub struct StaticPhysicsObject;

#[derive(Component, Default)]
#[require(PhysicsPosition)]
pub struct DynamicPhysicsObject {
    pub step_height: f32,
    pub touching_sides: IVec3,
}
