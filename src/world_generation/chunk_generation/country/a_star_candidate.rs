use std::cmp::Ordering;

use bevy::prelude::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AStarCandidate {
    pub estimated_weight: i32,
    pub real_weight: i32,
    pub state: IVec2,
    pub direction: IVec2,
}

impl PartialOrd for AStarCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AStarCandidate {
    fn cmp(&self, other: &Self) -> Ordering {
        other.estimated_weight.cmp(&self.estimated_weight)
    }
}
