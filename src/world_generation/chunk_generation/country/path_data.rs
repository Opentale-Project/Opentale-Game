use std::{
    collections::{BinaryHeap, HashMap},
    sync::Arc,
    time::Instant,
};

use bevy::prelude::*;
use noise::NoiseFn;

use crate::world_generation::{
    chunk_generation::{
        chunk_lod::ChunkLod,
        country::{
            a_star_candidate::AStarCandidate,
            country_cache::{COUNTRY_SIZE, CacheStore},
            country_cache_position::CountryPosition,
            generation_cache::GenerationCacheItem,
        },
        noise::{
            full_cache::FullCache, lod_height_adjuster::LodHeightAdjuster,
        },
    },
    generation_options::GenerationOptions,
};

#[derive(Default)]
pub struct PathData {
    pub paths: Vec<Path>,
}

pub struct Path {
    pub lines: Vec<PathLine>,
    pub box_pos_start: IVec2,
    pub box_pos_end: IVec2,
}

impl Path {
    pub fn is_in_box(&self, point: IVec2, margin: IVec2) -> bool {
        let bb_start = self.box_pos_start - margin;
        let bb_end = self.box_pos_end + margin;
        !(point.x < bb_start.x
            || point.x > bb_end.x
            || point.y < bb_start.y
            || point.y > bb_end.y)
    }
}

pub struct PathLine {
    pub start: IVec2,
    pub end: IVec2,
    pub spline_one: Vec2,
    pub spline_two: Vec2,
    pub box_pos_start: IVec2,
    pub box_pos_end: IVec2,
    pub estimated_length: f32,
    pub sample_points: Vec<IVec2>,
}

impl PathLine {
    fn new(start: IVec2, end: IVec2, before: IVec2, after: IVec2) -> Self {
        let spline_one = start.as_vec2() + (end - before).as_vec2() / 2. / 3.;
        let spline_two = end.as_vec2() - (after - start).as_vec2() / 2. / 3.;

        let estimated_length = start.as_vec2().distance(end.as_vec2());

        let spline_one = start.as_vec2()
            + (spline_one - start.as_vec2()).normalize()
                * (estimated_length / 2.);
        let spline_two = end.as_vec2()
            + (spline_two - end.as_vec2()).normalize()
                * (estimated_length / 2.);

        let box_pos_start = start.min(end);
        let box_pos_end = start.max(end);

        let mut path_line = Self {
            start,
            end,
            spline_one,
            spline_two,
            box_pos_start,
            box_pos_end,
            estimated_length,
            sample_points: vec![start],
        };

        let num_points = (estimated_length / 20.).max(2.);

        let mut last_point = IVec2::ZERO;

        for i in 1..num_points as i32 {
            let current_progress = i as f32 / num_points;

            let current_pos =
                path_line.lerp_on_spline(current_progress).as_ivec2();

            if last_point != current_pos {
                path_line.sample_points.push(current_pos);
                path_line.box_pos_start =
                    path_line.box_pos_start.min(current_pos);
                path_line.box_pos_end = path_line.box_pos_end.max(current_pos);

                last_point = current_pos;
            }
        }

        path_line.sample_points.push(end);

        path_line
    }

    pub fn is_in_box(&self, point: IVec2, margin: IVec2) -> bool {
        let bb_start = self.box_pos_start - margin;
        let bb_end = self.box_pos_end + margin;
        !(point.x < bb_start.x
            || point.x > bb_end.x
            || point.y < bb_start.y
            || point.y > bb_end.y)
    }

    pub fn get_progress_on_line(&self, point: IVec2) -> f32 {
        let distance_to_start = self.start.distance_squared(point) as f32;
        let distance_to_end = self.end.distance_squared(point) as f32;

        distance_to_start / (distance_to_start + distance_to_end)
    }

    pub fn closest_point_on_path(
        &self,
        point: IVec2,
        margin: IVec2,
    ) -> Option<(Vec2, Vec2)> {
        let mut min_squared = i32::MAX;
        let mut closest = Vec2::ZERO;
        let mut closest_index = 0usize;

        for i in 1..self.sample_points.len() {
            let end = self.sample_points[i];
            let start = self.sample_points[i - 1];

            let box_start = start.min(end);
            let box_end = start.max(end);

            if point.cmpge(box_start - margin).all()
                && point.cmplt(box_end + margin).all()
            {
                let closest_point =
                    Self::get_closest_point_to_line(start, end, point);
                let dist_squared =
                    point.distance_squared(closest_point.as_ivec2());
                if dist_squared < min_squared {
                    min_squared = dist_squared;
                    closest = closest_point;
                    closest_index = i;
                }
            }
        }

        if min_squared < i32::MAX {
            let closest_start = self.sample_points[closest_index - 1];
            let closest_end = self.sample_points[closest_index];

            Some((closest, (closest_end - closest_start).as_vec2().normalize()))
        } else {
            None
        }
    }

    fn get_closest_point_to_line(
        line_start: IVec2,
        line_end: IVec2,
        point: IVec2,
    ) -> Vec2 {
        let length_squared = (line_end - line_start).length_squared();
        if length_squared == 0 {
            return line_start.as_vec2();
        }

        let t = ((point - line_start).dot(line_end - line_start) as f32
            / length_squared as f32)
            .clamp(0., 1.);
        let projection =
            line_start.as_vec2() + t * (line_end - line_start).as_vec2();

        projection
    }

    pub fn lerp_on_spline(&self, t: f32) -> Vec2 {
        let a = Self::lerp(self.start.as_vec2(), self.spline_one, t);
        let b = Self::lerp(self.spline_one, self.spline_two, t);
        let c = Self::lerp(self.spline_two, self.end.as_vec2(), t);

        let d = Self::lerp(a, b, t);
        let e = Self::lerp(b, c, t);

        Self::lerp(d, e, t)
    }

    fn lerp(p1: Vec2, p2: Vec2, t: f32) -> Vec2 {
        (1. - t) * p1 + t * p2
    }
}

impl GenerationCacheItem<CountryPosition> for PathData {
    fn generate(
        key: CountryPosition,
        generation_options: &GenerationOptions,
        cache_store: Arc<CacheStore>,
    ) -> Self {
        if !generation_options.generate_paths {
            return Self { paths: vec![] };
        }

        let top_country_pos = CountryPosition::new(*key + IVec2::X);
        let right_country_pos = CountryPosition::new(*key + IVec2::Y);

        let current_structure_cache = cache_store
            .clone()
            .structure_cache
            .get_cache_entry(key, generation_options, cache_store.clone());
        let top_structure_cache =
            cache_store.clone().structure_cache.get_cache_entry(
                top_country_pos,
                generation_options,
                cache_store.clone(),
            );
        let right_structure_cache =
            cache_store.clone().structure_cache.get_cache_entry(
                right_country_pos,
                generation_options,
                cache_store.clone(),
            );

        let path_finding_lod = ChunkLod::Thirtytwoth;

        let y_start_pos = if key.y % 2 == 0 && key.x % 2 == 0 { current_structure_cache.city_location } else { top_structure_cache.city_location };
        let y_end_pos = if key.y % 2 == 0 && key.x % 2 == 0 { top_structure_cache.city_location } else { current_structure_cache.city_location };

        let x_start_pos = if key.y % 2 == 0 && key.x % 2 == 0 { current_structure_cache.city_location } else { right_structure_cache.city_location };
        let x_end_pos = if key.y % 2 == 0 && key.x % 2 == 0 { right_structure_cache.city_location } else { current_structure_cache.city_location };

        Self {
            paths: vec![
                PathData::generate_path(
                    y_start_pos,
                    y_end_pos,
                    [*key, *top_country_pos],
                    path_finding_lod,
                    generation_options,
                ),
                PathData::generate_path(
                    x_start_pos,
                    x_end_pos,
                    [*key, *right_country_pos],
                    path_finding_lod,
                    generation_options,
                ),
            ],
        }
    }
}

impl PathData {
    fn generate_path(
        mut start_pos: IVec2,
        mut end_pos: IVec2,
        country_positions: [IVec2; 2],
        path_finding_lod: ChunkLod,
        generation_options: &GenerationOptions,
    ) -> Path {
        start_pos /= path_finding_lod.multiplier_i32();
        end_pos /= path_finding_lod.multiplier_i32();

        let terrain_noise = FullCache::new(LodHeightAdjuster::new(
            generation_options.get_terrain_noise(),
            path_finding_lod,
        ));

        let get_terrain_height = |pos: IVec2| -> f64 {
            terrain_noise.get(
                (pos * path_finding_lod.multiplier_i32())
                    .as_dvec2()
                    .to_array(),
            ) * path_finding_lod.multiplier_i32() as f64
        };

        let distance_to_end = |pos: IVec2| -> i32 {
            let diff = (end_pos - pos).abs();
            let smaller = diff.min_element();
            let bigger = diff.max_element();
            bigger * 10 + smaller * 4
        };

        let neighbours = |pos: IVec2| -> [(IVec2, i32); 8] {
            [
                (pos + IVec2::new(1, 0), 10),
                (pos + IVec2::new(0, 1), 10),
                (pos + IVec2::new(-1, 0), 10),
                (pos + IVec2::new(0, -1), 10),
                (pos + IVec2::new(1, 1), 14),
                (pos + IVec2::new(-1, 1), 14),
                (pos + IVec2::new(-1, -1), 14),
                (pos + IVec2::new(1, -1), 14),
            ]
        };

        let is_outside_of_countries = |pos: IVec2| -> bool {
            let pos = pos * path_finding_lod.multiplier_i32();
            let is_outside_first_country = pos.x
                < country_positions[0].x * COUNTRY_SIZE as i32
                || pos.x
                    >= (country_positions[0] + IVec2::X).x
                        * COUNTRY_SIZE as i32
                || pos.y < country_positions[0].y * COUNTRY_SIZE as i32
                || pos.y
                    >= (country_positions[0] + IVec2::Y).y
                        * COUNTRY_SIZE as i32;
            let is_outside_second_country = pos.x
                < country_positions[1].x * COUNTRY_SIZE as i32
                || pos.x
                    >= (country_positions[1] + IVec2::X).x
                        * COUNTRY_SIZE as i32
                || pos.y < country_positions[1].y * COUNTRY_SIZE as i32
                || pos.y
                    >= (country_positions[1] + IVec2::Y).y
                        * COUNTRY_SIZE as i32;
            is_outside_first_country && is_outside_second_country
        };

        let mut queue = BinaryHeap::new();
        let mut previous = HashMap::new();
        let mut weights = HashMap::new();

        weights.insert((start_pos, IVec2::ZERO), 0);
        queue.push(AStarCandidate {
            estimated_weight: distance_to_end(start_pos),
            real_weight: 0,
            state: start_pos,
            direction: IVec2::ZERO,
        });

        info!("start_pos: {start_pos}, end_pos: {end_pos}");

        let now = Instant::now();

        while let Some(AStarCandidate {
            estimated_weight: _,
            real_weight,
            state: current,
            direction: current_direction,
        }) = queue.pop()
        {
            if current == end_pos {
                break;
            }

            let current_height = get_terrain_height(current);

            for (next, weight) in neighbours(current) {
                if is_outside_of_countries(next) {
                    continue;
                }

                let direction = next - current;
                let direction_difference =
                    (direction - current_direction).abs();
                let direction_cost =
                    direction_difference.x + direction_difference.y;

                if direction_cost > 1 {
                    continue;
                }

                let next_height = get_terrain_height(next);

                let height_difference = (current_height - next_height).abs()
                    / path_finding_lod.multiplier_i32() as f64;
                if height_difference > 0.55 {
                    continue;
                }

                let direction_turned = direction.perp();
                let side_height = get_terrain_height(next + direction_turned);
                let steepness = (next_height - side_height).abs()
                    / path_finding_lod.multiplier_i32() as f64;

                let real_weight = real_weight
                    + weight
                    + (height_difference * 40.) as i32
                    + (steepness * 20.) as i32; //((total_steepness * 0.6).max(0.) * 10.0) as i32;
                if weights
                    .get(&(next, direction))
                    .map(|&weight| real_weight < weight)
                    .unwrap_or(true)
                {
                    let estimated_weight = real_weight + distance_to_end(next);
                    weights.insert((next, direction), real_weight);
                    queue.push(AStarCandidate {
                        estimated_weight,
                        real_weight,
                        state: next,
                        direction,
                    });
                    previous.insert((next, direction), (current, current_direction));
                }
            }
        }

        let elapsed = now.elapsed();

        info!("DONE: {}s", elapsed.as_secs_f32());

        if let Some((_, (parent, parent_direction))) = previous.iter().find(|((pos, ..), ..)| *pos == end_pos) {
            let mut points = vec![end_pos * path_finding_lod.multiplier_i32(), *parent * path_finding_lod.multiplier_i32()];
            let mut min = end_pos.min(*parent);
            let mut max = end_pos.max(*parent);

            let mut current = *parent;
            let mut current_direction = *parent_direction;

            while let Some((parent, parent_direction)) = previous.get(&(current, current_direction)) {
                points.push(*parent * path_finding_lod.multiplier_i32());
                current = *parent;
                current_direction = *parent_direction;

                min = min.min(*parent);
                max = max.max(*parent);
            }

            let mut smooth_points = vec![points[0]];

            for i in 0..points.len() - 1 {
                let start = points[i];
                let end = points[i + 1];

                smooth_points.push((start + end) / 2);
            }

            smooth_points.push(points[points.len() - 1]);

            let mut path = vec![];

            if smooth_points.len() >= 4 {
                for i in 1..smooth_points.len() - 2 {
                    path.push(PathLine::new(
                        smooth_points[i],
                        smooth_points[i + 1],
                        smooth_points[i - 1],
                        smooth_points[i + 2],
                    ));
                }
            }

            Path {
                lines: path,
                box_pos_start: min * path_finding_lod.multiplier_i32(),
                box_pos_end: max * path_finding_lod.multiplier_i32(),
            }
        } else {
            info!("NO PATH COULD BE CREATED!");
            Path {
                lines: vec![],
                box_pos_start: Default::default(),
                box_pos_end: Default::default(),
            }
        }
    }
}
