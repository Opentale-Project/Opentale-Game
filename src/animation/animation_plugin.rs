use bevy::prelude::*;

use crate::animation::{
    despawn_animation::animate_despawn_animation, spawn_animation::animate_spawn_animation,
};

pub struct OpentaleAnimationPlugin;

impl Plugin for OpentaleAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (animate_spawn_animation, animate_despawn_animation));
    }
}
