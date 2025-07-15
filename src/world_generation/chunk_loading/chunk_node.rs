use std::{
    ops::DerefMut,
    sync::{Arc, atomic::AtomicI8},
};

use bevy::prelude::*;
use itertools::Itertools;

use crate::world_generation::{
    chunk_generation::ChunkTaskGenerator,
    chunk_loading::{
        chunk_loader::ChunkLoader, chunk_tree::ChunkTreePos,
        lod_position::LodPosition,
    },
};

/// The Chunk Node component represents a branch in the Quad-Tree.
#[derive(Component, Default, Clone)]
pub struct ChunkNode {
    tree_pos: ChunkTreePos,
    position: LodPosition,
    parent: Option<Entity>,
    children_completion: Arc<AtomicI8>,
    tree_children: Option<ChunkNodeChildren>,
    is_leaf: bool,
}

impl ChunkNode {
    pub fn new(position: LodPosition, tree_pos: ChunkTreePos) -> Self {
        Self {
            position,
            parent: None,
            tree_children: None,
            tree_pos,
            ..Default::default()
        }
    }

    pub fn new_with_parent(
        position: LodPosition,
        parent: Entity,
        tree_pos: ChunkTreePos,
    ) -> Self {
        Self {
            position,
            parent: Some(parent),
            tree_children: None,
            tree_pos,
            ..Default::default()
        }
    }

    pub fn set_children(&mut self, children: ChunkNodeChildren) {
        self.tree_children = Some(children)
    }
}

/// The children of a Chunk Node.
/// These should also be spawned as the children of the Chunk Node,
/// so despawning the Chunk Node also despawns them.
///
/// top_right: +x, +y
///
/// top_left: -x, +y
///
/// bottom_right: +x, -y
///
/// bottom_left: -x, -y
#[derive(Clone, Copy)]
pub struct ChunkNodeChildren {
    top_right: Entity,
    top_left: Entity,
    bottom_right: Entity,
    bottom_left: Entity,
}

pub fn recurse_chunk_nodes(
    mut commands: Commands,
    chunk_nodes: Query<(&mut ChunkNode, Entity)>,
    chunk_loaders: Query<(&ChunkLoader, &Transform)>,
) {
    for (mut chunk_node, chunk_node_entity) in chunk_nodes
        .into_iter()
        .sort_by::<(&ChunkNode, Entity)>(|a, b| {
            a.0.position.lod.cmp(&b.0.position.lod).reverse()
        })
    {
        let tree_pos = chunk_node.tree_pos;
        let min_lod = chunk_node
            .position
            .get_containing_chunk_pos()
            .flat_map(|chunk_pos| {
                let pos_clone = chunk_pos.clone();
                chunk_loaders.iter().map(move |chunk_loader| {
                    chunk_loader.0.get_min_lod_for_chunk(
                        pos_clone.to_absolute(tree_pos),
                        chunk_loader.1.translation,
                    )
                })
            })
            .min();

        let Some(min_lod) = min_lod else {
            continue;
        };

        if min_lod < chunk_node.position.lod
            && chunk_node.tree_children.is_none()
        {
            devide_chunk_node(
                &mut chunk_node,
                chunk_node_entity,
                &mut commands,
            );
        }

        if min_lod == chunk_node.position.lod && !chunk_node.is_leaf {
            chunk_node.is_leaf = true;
            commands
                .entity(chunk_node_entity)
                .insert(ChunkTaskGenerator(
                    *chunk_node.tree_pos,
                    chunk_node.position.lod,
                    chunk_node.position.relative_position,
                    0,
                    chunk_node_entity,
                ));
        }
    }
}

/// Divide the Chunk Node into 4 more chunk nodes.
/// We don't have to handle the despawning of the existing meshes since we handle that later with the counter.
fn devide_chunk_node(
    chunk_node: &mut ChunkNode,
    chunk_node_entity: Entity,
    commands: &mut Commands,
) {
    let mut spawn_child = |new_pos| {
        commands
            .spawn(ChunkNode::new_with_parent(
                new_pos,
                chunk_node_entity,
                chunk_node.tree_pos,
            ))
            .id()
    };

    let top_right = spawn_child(chunk_node.position.to_top_right());
    let top_left = spawn_child(chunk_node.position.to_top_left());
    let bottom_right = spawn_child(chunk_node.position.to_bottom_right());
    let bottom_left = spawn_child(chunk_node.position.to_bottom_left());

    chunk_node.set_children(ChunkNodeChildren {
        top_right,
        top_left,
        bottom_right,
        bottom_left,
    });
}
