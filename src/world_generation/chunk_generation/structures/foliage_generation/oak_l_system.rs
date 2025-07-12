use std::ops::Range;

use bevy::math::Vec3;
use rand::{rngs::StdRng, Rng};

use crate::world_generation::{
    chunk_generation::{
        BlockType, 
        structures::foliage_generation::tree_l_system::{LSystem, LSystemEntry},
        VOXEL_SIZE
    },
};

pub struct OakLSystem;

#[derive(Clone, Copy)]
pub enum OakEntryType {
    Stem,
    Branch { angle_x: f32, angle_z: f32 },
    Leaf,
}

impl LSystem<OakEntryType> for OakLSystem {
    fn get_start_state(
        position: Vec3, 
        rng: &mut StdRng
    ) -> Vec<LSystemEntry<OakEntryType>> {
        Self::create_straight_piece(
            &position,
            0.,
            0.,
            2.0,
            (rng.random_range(3.5..5.5) / VOXEL_SIZE) as usize,
            OakEntryType::Stem,
            OakEntryType::Branch {
                angle_x: 0.,
                angle_z: 0.,
            },
        )
    }

    fn process_tree(
        mut start_state: &mut Vec<LSystemEntry<OakEntryType>>, 
        rng: &mut StdRng
    ) {
        for _ in 0..3 {
            Self::recurse_l_system(&mut start_state, rng);
        }
        Self::add_leafs(&mut start_state);
    }

    fn get_block_from_entry(entry: &LSystemEntry<OakEntryType>) -> BlockType {
        match entry.entry_type {
            OakEntryType::Leaf => BlockType::Grass,
            _ => BlockType::Path,
        }
    }

    fn recurse_entry(
        entry: &LSystemEntry<OakEntryType>,
        rng: &mut StdRng,
        branches: &mut Vec<LSystemEntry<OakEntryType>>,
    ) {
        if let OakEntryType::Branch { angle_x, angle_z } = entry.entry_type {
            let random_range: Range<f32> = -45.0..45.0;
            let new_thickness = (entry.thickness - 0.5).max(0.75);

            for _ in 0..6 {
                let new_length = (rng.random_range(3.5..5.5) / VOXEL_SIZE) as usize;

                branches.extend(Self::create_straight_piece(
                    &entry.pos,
                    angle_x + rng.random_range(random_range.clone()),
                    angle_z + rng.random_range(random_range.clone()),
                    new_thickness,
                    new_length,
                    OakEntryType::Stem,
                    OakEntryType::Branch { angle_x, angle_z },
                ));
            }
        }
    }
}

impl OakLSystem {
    fn add_leafs(data: &mut Vec<LSystemEntry<OakEntryType>>) {
        let mut i = 0usize;
        while i < data.len() {
            let entry = &data[i];
            if let OakEntryType::Branch {..} = entry.entry_type {
                let branches = vec![LSystemEntry {
                    pos: entry.pos,
                    entry_type: OakEntryType::Leaf,
                    thickness: 2.0,
                }];

                let length = branches.len();

                data.splice(i..i + 1, branches);
                i += length;
            }
            i += 1;
        }
    }
}
