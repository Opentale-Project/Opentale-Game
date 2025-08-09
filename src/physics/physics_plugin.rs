use bevy::prelude::*;

use crate::physics::{
    physics_position::update_transform_position, physics_set::PhysicsSet,
    update_physics::update_physics,
};

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, update_physics.in_set(PhysicsSet))
            .add_systems(Update, update_transform_position.in_set(PhysicsSet));
    }
}
