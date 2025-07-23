use bevy::prelude::*;
use noise::{Add, Constant, NoiseFn};

use crate::{
    debug_tools::debug_resource::OpentaleDebugResource,
    player::player_component::Player,
    world_generation::{
        chunk_generation::{
            VOXEL_SIZE,
            country::{
                country_cache::{CountryCache, GenerationState},
                country_cache_position::CountryPosition,
            },
            voxel_generation::get_terrain_noise,
        },
        chunk_loading::chunk_pos::AbsoluteChunkPos,
        generation_options::GenerationOptionsResource,
    },
};

pub fn setup_gizmo_settings(mut config: ResMut<GizmoConfigStore>) {
    let (config, ..) = config.config_mut::<DefaultGizmoConfigGroup>();
    config.depth_bias = -1.;
    config.line.width = 4.;
}

pub fn draw_path_gizmos(
    mut gizmos: Gizmos,
    generation_options: Res<GenerationOptionsResource>,
    country_cache: Res<CountryCache>,
    players: Query<&Transform, With<Player>>,
    debug_resource: Res<OpentaleDebugResource>,
) {
    if !debug_resource.show_path_debug {
        return;
    }

    let terrain_noise =
        Add::new(get_terrain_noise(&generation_options.0), Constant::new(5.));

    for player in &players {
        let player_chunk_pos =
            AbsoluteChunkPos::from_absolute(player.translation);
        let player_country_pos =
            CountryPosition::from_chunk_pos(player_chunk_pos);

        let player_voxel_pos =
            (player.translation / VOXEL_SIZE).as_ivec3().xz();

        let Some(country_cache) =
            country_cache.country_cache.get(&player_country_pos)
        else {
            continue;
        };

        let GenerationState::Some(country_cache) = country_cache else {
            continue;
        };
        for path in country_cache
            .this_path_cache
            .paths
            .iter()
            .chain(&country_cache.bottom_path_cache.paths)
            .chain(&country_cache.left_path_cache.paths)
        {
            if !path.is_in_box(
                player_voxel_pos,
                IVec2::ONE * debug_resource.path_show_range,
            ) {
                continue;
            }

            for path_line in &path.lines {
                if !path_line.is_in_box(
                    player_voxel_pos,
                    IVec2::ONE * debug_resource.path_show_range,
                ) {
                    continue;
                }

                let is_in_path =
                    path_line.is_in_box(player_voxel_pos, IVec2::ONE * 5);

                let color = if is_in_path {
                    Color::srgb(229. / 255., 171. / 255., 0.)
                } else {
                    Color::srgb(0., 200. / 255., 0.)
                };

                gizmos.line(
                    Vec3::from((
                        path_line.start.as_vec2(),
                        terrain_noise.get(path_line.start.as_dvec2().to_array())
                            as f32,
                    ))
                    .xzy()
                        * VOXEL_SIZE,
                    Vec3::from((
                        path_line.end.as_vec2(),
                        terrain_noise.get(path_line.end.as_dvec2().to_array())
                            as f32,
                    ))
                    .xzy()
                        * VOXEL_SIZE,
                    color,
                );
                if !is_in_path {
                    continue;
                }
                gizmos.circle(
                    Isometry3d {
                        rotation: Quat::from_rotation_arc(Vec3::Z, Vec3::Y),
                        translation: Vec3A::from((
                            path_line.spline_one,
                            terrain_noise
                                .get(path_line.spline_one.as_dvec2().to_array())
                                as f32,
                        ))
                        .xzy()
                            * VOXEL_SIZE,
                    },
                    debug_resource.path_circle_radius,
                    Color::srgb(0., 200. / 255., 0.),
                );
                gizmos.circle(
                    Isometry3d {
                        rotation: Quat::from_rotation_arc(Vec3::Z, Vec3::Y),
                        translation: Vec3A::from((
                            path_line.spline_two,
                            terrain_noise
                                .get(path_line.spline_two.as_dvec2().to_array())
                                as f32,
                        ))
                        .xzy()
                            * VOXEL_SIZE,
                    },
                    debug_resource.path_circle_radius,
                    Color::srgb(200. / 255., 0., 0.),
                );
                gizmos.circle(
                    Isometry3d {
                        rotation: Quat::from_rotation_arc(Vec3::Z, Vec3::Y),
                        translation: Vec3A::from((
                            path_line.start.as_vec2(),
                            terrain_noise
                                .get(path_line.start.as_dvec2().to_array())
                                as f32,
                        ))
                        .xzy()
                            * VOXEL_SIZE,
                    },
                    debug_resource.path_circle_radius,
                    Color::srgb(0., 200. / 255., 0.),
                );
                gizmos.circle(
                    Isometry3d {
                        rotation: Quat::from_rotation_arc(Vec3::Z, Vec3::Y),
                        translation: Vec3A::from((
                            path_line.end.as_vec2(),
                            terrain_noise
                                .get(path_line.end.as_dvec2().to_array())
                                as f32,
                        ))
                        .xzy()
                            * VOXEL_SIZE,
                    },
                    debug_resource.path_circle_radius,
                    Color::srgb(200. / 255., 0., 0.),
                );

                for i in 1..path_line.sample_points.len() {
                    let start = path_line.sample_points[i - 1];
                    let end = path_line.sample_points[i];
                    gizmos.line(
                        Vec3::from((
                            start.as_vec2(),
                            terrain_noise.get(start.as_dvec2().to_array())
                                as f32,
                        ))
                        .xzy()
                            * VOXEL_SIZE,
                        Vec3::from((
                            end.as_vec2(),
                            terrain_noise.get(end.as_dvec2().to_array()) as f32,
                        ))
                        .xzy()
                            * VOXEL_SIZE,
                        Color::srgb(200. / 255., 0., 0.),
                    );
                }

                let Some((player_pos_on_path, _)) = path_line
                    .closest_point_on_path(player_voxel_pos, IVec2::ONE * 5)
                else {
                    continue;
                };

                gizmos.circle(
                    Isometry3d {
                        rotation: Quat::from_rotation_arc(Vec3::Z, Vec3::Y),
                        translation: Vec3A::from((
                            player_pos_on_path,
                            terrain_noise
                                .get(player_pos_on_path.as_dvec2().to_array())
                                as f32,
                        ))
                        .xzy()
                            * VOXEL_SIZE,
                    },
                    debug_resource.path_circle_radius,
                    Color::srgb(0., 0., 200. / 255.),
                );
                gizmos.circle(
                    Isometry3d {
                        rotation: Quat::from_rotation_arc(Vec3::Z, Vec3::Y),
                        translation: Vec3A::from((
                            player_pos_on_path.as_ivec2().as_vec2()
                                + VOXEL_SIZE,
                            terrain_noise
                                .get(player_pos_on_path.as_dvec2().to_array())
                                as f32,
                        ))
                        .xzy()
                            * VOXEL_SIZE,
                    },
                    debug_resource.path_circle_radius,
                    Color::srgb(0., 200. / 255., 200. / 255.),
                );

                gizmos.circle(
                    Isometry3d {
                        rotation: Quat::from_rotation_arc(Vec3::Z, Vec3::Y),
                        translation: Vec3A::from((
                            player_voxel_pos.as_vec2() + VOXEL_SIZE,
                            terrain_noise
                                .get(player_pos_on_path.as_dvec2().to_array())
                                as f32,
                        ))
                        .xzy()
                            * VOXEL_SIZE,
                    },
                    debug_resource.path_circle_radius,
                    Color::srgb(0., 100. / 255., 200. / 255.),
                );
            }
        }
    }
}
