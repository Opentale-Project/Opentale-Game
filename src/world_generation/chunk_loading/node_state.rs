use std::sync::{Arc, atomic::AtomicI8};

use bevy::ecs::entity::Entity;

use crate::world_generation::chunk_loading::chunk_node_children::ChunkNodeChildren;

#[derive(Clone)]
pub enum NodeState {
    Branch {
        children: ChunkNodeChildren,
        child_count: Arc<AtomicI8>,
    },
    Leaf {
        spawned_task: bool,
        children: Option<ChunkNodeChildren>,
        stacking: Vec<Entity>,
    },
}

impl NodeState {
    pub fn is_leaf(&self) -> bool {
        match self {
            Self::Leaf { .. } => true,
            _ => false,
        }
    }

    pub fn is_branch(&self) -> bool {
        match self {
            Self::Branch { .. } => true,
            _ => false,
        }
    }
}
