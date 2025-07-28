use bevy::prelude::*;

#[derive(Component, Default)]
pub struct DespawnAnimation {
    progress: f32,
    origin: Option<Vec3>,
}

pub(super) fn animate_despawn_animation(
    mut commands: Commands,
    time: Res<Time>,
    mut despawn_animations: Query<(Entity, &mut Transform, &mut DespawnAnimation)>,
    mut despawn_animations_no_transform: Query<
        Entity,
        (With<DespawnAnimation>, Without<Transform>),
    >,
) {
    for entity in &mut despawn_animations_no_transform {
        commands.entity(entity).despawn();
    }

    for (entity, mut transform, mut despawn_animation) in &mut despawn_animations {
        if despawn_animation.progress == 0. && despawn_animation.origin.is_none() {
            despawn_animation.origin = Some(transform.translation);
        }

        transform.translation.y =
            despawn_animation.origin.unwrap().y - 40. * despawn_animation.progress.min(1.).powi(2);

        if despawn_animation.progress >= 1. {
            commands.entity(entity).despawn();
            continue;
        }

        despawn_animation.progress += time.delta_secs();
    }
}
