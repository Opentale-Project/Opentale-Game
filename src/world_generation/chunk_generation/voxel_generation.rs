use crate::utils::div_floor::div_floor;
use crate::world_generation::chunk_generation::block_type::BlockType;
use crate::world_generation::chunk_generation::chunk_lod::ChunkLod;
use crate::world_generation::chunk_generation::country::country_data::CountryData;
use crate::world_generation::chunk_generation::country::path_data::{Path, PathLine};
use crate::world_generation::chunk_generation::structures::structure_generator::{
    StructureGenerator, StructureGeneratorCache,
};
use crate::world_generation::chunk_generation::voxel_data::VoxelData;
use crate::world_generation::chunk_generation::{CHUNK_SIZE, VOXEL_SIZE};
use crate::world_generation::generation_options::GenerationOptions;
use bevy::math::{DVec2, IVec2};
use bevy::prelude::Vec2;
use noise::NoiseFn;
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use std::usize;

use super::noise::full_cache::FullCache;
use super::noise::lod_height_adjuster::LodHeightAdjuster;
use super::noise::steepness::Steepness;

pub fn generate_voxels(
    position: [i32; 3],
    generation_options: &GenerationOptions,
    chunk_lod: ChunkLod,
    country_data: &CountryData,
) -> (VoxelData, i32, bool) {
    let mut blocks = VoxelData::default();

    let terrain_noise = FullCache::new(LodHeightAdjuster::new(
        generation_options.get_terrain_noise(),
        chunk_lod,
    ));
    let terrain_steepness = FullCache::new(Steepness::new(FullCache::new(
        generation_options.get_terrain_noise(),
    )));

    let chunk_noise_offset =
        DVec2::new(position[0] as f64, position[2] as f64) * CHUNK_SIZE as f64;

    let min_height =
        (get_min_in_noise_map(&terrain_noise, chunk_noise_offset, chunk_lod)
            as i32)
            - 2
            + position[1] * CHUNK_SIZE as i32
            - 10 / chunk_lod.multiplier_i32();

    let mut generate_more: bool = false;

    let all_paths = vec![
        &country_data.this_path_cache.paths,
        &country_data.bottom_path_cache.paths,
        &country_data.left_path_cache.paths,
    ];

    let structure_generators: Vec<StructureGeneratorCache> = generation_options
        .structure_generators
        .iter()
        .map(|structure_generator| {
            StructureGeneratorCache::new(structure_generator)
        })
        .collect();

    for x in 0..CHUNK_SIZE + 2 {
        for z in 0..CHUNK_SIZE + 2 {
            let total_x = position[0] * CHUNK_SIZE as i32
                + x as i32 * chunk_lod.multiplier_i32();
            let total_z = position[2] * CHUNK_SIZE as i32
                + z as i32 * chunk_lod.multiplier_i32();

            let noise_position = [total_x as f64, total_z as f64];

            //let dryness = value_noise.get([total_x as f64, total_z as f64]);
            //let mountain = mountain_noise.get([total_x as f64, total_z as f64]);

            let steepness = terrain_steepness.get(noise_position);

            let mut noise_height = terrain_noise.get(noise_position) as f32;

            let is_snow =
                noise_height * chunk_lod.multiplier_f32() > 3500. / VOXEL_SIZE;
            let is_grass_steep = if is_snow {
                steepness < 1.2
            } else {
                steepness < 0.8
            };

            let (mut path_distance, closest_point_on_path, _, line) =
                get_min_distance_to_path(
                    IVec2::new(total_x, total_z),
                    &all_paths,
                    IVec2::ONE * 15,
                );

            path_distance *= 2.;

            let is_path = path_distance <= 8.75;

            path_distance /= 10.;

            if path_distance <= 1.65 {
                let path_start_height = terrain_noise
                    .get(line.unwrap().start.as_dvec2().to_array())
                    as f32;
                let path_end_height = terrain_noise
                    .get(line.unwrap().end.as_dvec2().to_array())
                    as f32;
                let path_height = lerp(
                    path_start_height,
                    path_end_height,
                    line.unwrap().get_progress_on_line(closest_point_on_path),
                );

                let closest_point_height = terrain_noise
                    .get(closest_point_on_path.as_dvec2().to_array())
                    as f32;
                let closest_point_height =
                    lerp(closest_point_height, noise_height, 0.75);

                let path_height = lerp(closest_point_height, path_height, 0.5);

                noise_height = lerp(
                    noise_height,
                    path_height,
                    (1.65 - path_distance.powi(2)).clamp(0., 1.),
                )
                .max(noise_height - 10.);
            }

            for y in min_height
                ..noise_height.min((CHUNK_SIZE as i32 + 2 + min_height) as f32)
                    as i32
            {
                if y == CHUNK_SIZE as i32 + 1 + min_height {
                    generate_more = true;
                }
                blocks.set_block(
                    [x as i32, y as i32 - min_height, z as i32],
                    // BlockType::Gray((biome_noise.get([total_x as f64, total_z as f64]) * 255.) as u8)
                    if is_path {
                        BlockType::Dirt
                    } else {
                        if is_grass_steep
                            && y + 1 == noise_height.floor() as i32
                        {
                            if is_snow {
                                BlockType::Snow
                            } else {
                                BlockType::Grass
                            }
                        } else {
                            BlockType::Stone
                        }
                    },
                );
            }

            for structure_generator in &structure_generators {
                let structure_metadata =
                    structure_generator.get_structure_metadata();
                let structure_offset_x = div_floor(
                    total_x + structure_metadata.grid_offset[0],
                    structure_metadata.generation_size[0],
                );
                let structure_offset_z = div_floor(
                    total_z + structure_metadata.grid_offset[1],
                    structure_metadata.generation_size[1],
                );
                let structure_value = structure_metadata.noise.get_noise_2d(
                    structure_offset_x as f32,
                    structure_offset_z as f32,
                ) * 0.5
                    + 0.5;
                if structure_metadata.generate_debug_blocks {
                    let top_terrain = (noise_height
                        .min(CHUNK_SIZE as f32 + min_height as f32)
                        as i32
                        - min_height.min(noise_height as i32))
                    .max(1) as usize
                        - 1;
                    blocks.set_block(
                        [x as i32, top_terrain as i32, z as i32],
                        BlockType::Stone,
                    );
                }
                let mut rand = StdRng::seed_from_u64(
                    (structure_value.abs() * 10000.) as u64,
                );

                if structure_value > 0. {
                    let random_x = rand.random_range(
                        0..=structure_metadata.generation_size[0]
                            - structure_metadata.model_size[0],
                    );
                    let random_z = rand.random_range(
                        0..=structure_metadata.generation_size[1]
                            - structure_metadata.model_size[2],
                    );

                    let structure_x: i32 = (total_x
                        + structure_metadata.grid_offset[0]
                        - structure_offset_x
                            * structure_metadata.generation_size[0])
                        .abs()
                        - random_x;
                    let structure_z: i32 = (total_z
                        + structure_metadata.grid_offset[1]
                        - structure_offset_z
                            * structure_metadata.generation_size[1])
                        .abs()
                        - random_z;

                    if structure_x < 0
                        || structure_z < 0
                        || structure_x >= structure_metadata.model_size[0]
                        || structure_z >= structure_metadata.model_size[2]
                    {
                        continue;
                    }

                    let structure_noise_height_x = structure_offset_x
                        * structure_metadata.generation_size[0]
                        + (structure_metadata.model_size[0] / 2)
                        - structure_metadata.grid_offset[0]
                        + random_x;
                    let structure_noise_height_z = structure_offset_z
                        * structure_metadata.generation_size[1]
                        + (structure_metadata.model_size[2] / 2)
                        - structure_metadata.grid_offset[1]
                        + random_z;

                    let structure_steepness = terrain_steepness.get([
                        structure_noise_height_x as f64,
                        structure_noise_height_z as f64,
                    ]);

                    if structure_steepness > 0.8 {
                        continue;
                    }

                    let structure_center: IVec2 =
                        [structure_noise_height_x, structure_noise_height_z]
                            .into();

                    let (a, _, _, _) = get_min_distance_to_path(
                        structure_center,
                        &all_paths,
                        IVec2::new(
                            structure_metadata.model_size[0] / 2,
                            structure_metadata.model_size[2] / 2,
                        ) + IVec2::ONE * 10,
                    );

                    if (a as i32)
                        < structure_metadata.model_size[0] / 2
                            + structure_metadata.model_size[1] / 2
                    {
                        continue;
                    }

                    let noise_height = terrain_noise.get([
                        structure_noise_height_x as f64,
                        structure_noise_height_z as f64,
                    ]);

                    for (index, sub_structure) in structure_generator
                        .get_structure_model(
                            IVec2 {
                                x: structure_offset_x,
                                y: structure_offset_z,
                            },
                            chunk_lod,
                        )[structure_x as usize]
                        .iter()
                        .enumerate()
                    {
                        if (index as i32
                            + (noise_height * chunk_lod.multiplier_i32() as f64)
                                as i32)
                            % chunk_lod.multiplier_i32()
                            != 0
                        {
                            continue;
                        }
                        let chunk_index =
                            index / chunk_lod.multiplier_i32() as usize;
                        if (noise_height as i32 - min_height
                            + chunk_index as i32)
                            < 0
                        {
                            continue;
                        }
                        let structure_block =
                            sub_structure[structure_z as usize];
                        if structure_block == BlockType::Air {
                            continue;
                        }
                        if noise_height as i32 + chunk_index as i32 - min_height
                            >= CHUNK_SIZE as i32 + 2
                        {
                            generate_more = true;
                            break;
                        }
                        blocks.set_block(
                            [
                                x as i32,
                                noise_height as i32 + chunk_index as i32
                                    - min_height as i32,
                                z as i32,
                            ],
                            structure_block,
                        );
                    }
                }
            }
        }
    }

    (blocks, min_height, generate_more)
}

fn get_min_in_noise_map(
    noise: &impl NoiseFn<f64, 2usize>,
    chunk_offset: DVec2,
    chunk_lod: ChunkLod,
) -> f64 {
    let mut min = noise.get(chunk_offset.to_array());

    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            let current = noise.get([
                x as f64 * chunk_lod.multiplier_i32() as f64 + chunk_offset.x,
                z as f64 * chunk_lod.multiplier_i32() as f64 + chunk_offset.y,
            ]);
            if current < min {
                min = current;
            }
        }
    }

    min
}

fn get_min_distance_to_path<'a>(
    pos: IVec2,
    paths_list: &'a Vec<&'a Vec<Path>>,
    margin: IVec2,
) -> (f32, IVec2, Vec2, Option<&'a PathLine>) {
    let mut min: Option<f32> = None;
    let mut closest_point_total = IVec2::ZERO;
    let mut path_dir = Vec2::ZERO;
    let mut end_path = None;

    for paths in paths_list {
        for path in *paths {
            if !path.is_in_box(pos, margin) {
                continue;
            }

            for line in &path.lines {
                if !line.is_in_box(pos, margin) {
                    continue;
                }

                if let Some((closest_point, closest_path_dir)) =
                    line.closest_point_on_path(pos, margin)
                {
                    let distance = closest_point.distance(pos.as_vec2());
                    match min {
                        None => {
                            min = Some(distance);
                            closest_point_total = closest_point.as_ivec2();
                            path_dir = closest_path_dir;
                            end_path = Some(line);
                        }
                        Some(current_min) => {
                            if distance < current_min {
                                min = Some(distance);
                                closest_point_total = closest_point.as_ivec2();
                                path_dir = closest_path_dir;
                                end_path = Some(line);
                            }
                        }
                    }
                }
            }
        }
    }

    (
        min.unwrap_or(f32::INFINITY),
        closest_point_total,
        path_dir,
        end_path,
    )
}

fn lerp(a: f32, b: f32, f: f32) -> f32 {
    a + f * (b - a)
}
