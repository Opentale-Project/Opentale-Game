use bevy::prelude::*;

#[derive(Component, Default)]
pub struct SpawnAnimation {
    progress: f32,
    origin: Option<Vec3>,
}

pub(super) fn animate_spawn_animation(
    mut commands: Commands,
    time: Res<Time>,
    mut spawn_animations: Query<(Entity, &mut Transform, &mut SpawnAnimation)>,
) {
    for (entity, mut transform, mut spawn_animation) in &mut spawn_animations {
        if spawn_animation.progress == 0. && spawn_animation.origin.is_none() {
            spawn_animation.origin = Some(transform.translation);
        }

        transform.translation.y = spawn_animation.origin.unwrap().y
            + 40. * (1. - (1. - spawn_animation.progress.min(1.)).powi(2));

        if spawn_animation.progress >= 1. {
            commands.entity(entity).remove::<SpawnAnimation>();
            continue;
        }

        spawn_animation.progress += time.delta_secs();
    }
}
