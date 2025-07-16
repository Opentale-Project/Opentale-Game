use std::{
    cell::RefCell,
    collections::HashMap,
    ops::Deref,
    sync::{Arc, atomic::AtomicI8},
};

use bevy::prelude::*;
use itertools::Itertools;

use crate::world_generation::{
    chunk_generation::{
        CHUNK_SIZE, Chunk, ChunkGenerationTask, ChunkTaskGenerator, VOXEL_SIZE,
    },
    chunk_loading::{
        chunk_loader::ChunkLoader, chunk_pos::AbsoluteChunkPos,
        chunk_tree::ChunkTreePos, lod_position::LodPosition,
        query_stepper::ChunkNodeQueryStepper,
    },
    voxel_world::ChunkLod,
};

/// The Chunk Node component represents a branch in the Quad-Tree.
#[derive(Component, Clone)]
pub struct ChunkNode {
    tree_pos: ChunkTreePos,
    position: LodPosition,
    parent: Option<Entity>,
    children_completion: Arc<AtomicI8>,
    tree_children: Option<ChunkNodeChildren>,
    is_leaf: bool,
    has_generated: bool,
}

impl Default for ChunkNode {
    fn default() -> Self {
        Self {
            tree_pos: Default::default(),
            position: Default::default(),
            parent: Default::default(),
            children_completion: Default::default(),
            tree_children: Default::default(),
            is_leaf: true,
            has_generated: false,
        }
    }
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
    pub top_right: Entity,
    pub top_left: Entity,
    pub bottom_right: Entity,
    pub bottom_left: Entity,
}

pub fn recurse_chunk_nodes(
    mut commands: Commands,
    chunk_nodes: Query<(&mut ChunkNode, Entity)>,
    chunk_loaders: Query<(&ChunkLoader, &Transform)>,
) {
    let cache_map: RefCell<HashMap<(AbsoluteChunkPos, ChunkLoader), ChunkLod>> =
        RefCell::new(HashMap::new());

    for (mut chunk_node, chunk_node_entity) in chunk_nodes
        .into_iter()
        .sort_by::<(&ChunkNode, Entity)>(|a, b| {
            a.0.position.lod.cmp(&b.0.position.lod).reverse()
        })
    {
        let tree_pos = chunk_node.tree_pos;
        let min_lod = chunk_loaders
            .iter()
            .map(|chunk_loader| {
                let chunk_pos =
                    AbsoluteChunkPos::from_absolute(chunk_loader.1.translation);
                let closest = chunk_node
                    .position
                    .get_closest_chunk_pos(chunk_pos, tree_pos);
                let lod = chunk_loader
                    .0
                    .get_min_lod_for_chunk(closest, chunk_loader.1.translation);
                cache_map
                    .borrow_mut()
                    .insert((closest, *chunk_loader.0), lod);
                lod
            })
            .min();

        let Some(min_lod) = min_lod else {
            continue;
        };

        if min_lod < chunk_node.position.lod
            && chunk_node.tree_children.is_none()
        {
            chunk_node.is_leaf = false;

            devide_chunk_node(
                &mut chunk_node,
                chunk_node_entity,
                &mut commands,
            );
        }

        if chunk_node.is_leaf
            && !chunk_node.has_generated
            && !chunk_node.is_added()
        {
            chunk_node.has_generated = true;
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
            .spawn((
                ChunkNode::new_with_parent(
                    new_pos,
                    chunk_node_entity,
                    chunk_node.tree_pos,
                ),
                Transform::from_translation(
                    new_pos.get_absolute(chunk_node.tree_pos).extend(0.).xzy(),
                ),
                Visibility::Visible,
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

    commands.entity(chunk_node_entity).add_children(&[
        top_right,
        top_left,
        bottom_right,
        bottom_left,
    ]);

    chunk_node.is_leaf = false;
}

pub fn update_added_chunks(
    mut commands: Commands,
    mut added_chunks: Query<
        (&mut ChunkNode, Option<&Chunk>, Entity),
        Added<Chunk>,
    >,
) {
    let mut all_chunk_nodes =
        added_chunks
            .iter_mut()
            .collect::<Vec<(Mut<ChunkNode>, Option<&Chunk>, Entity)>>();
    let added_chunks_copy = all_chunk_nodes
        .iter()
        .map(|x| (x.0.deref().clone(), x.1.clone(), x.2))
        .collect::<Vec<(ChunkNode, Option<&Chunk>, Entity)>>();

    for (chunk_node, chunk, entity) in added_chunks_copy {
        let Some(chunk) = chunk else {
            continue;
        };

        if chunk.generate_above {
            let mut child_entity = commands.spawn_empty();
            child_entity.insert((
                ChunkTaskGenerator(
                    *chunk_node.tree_pos,
                    chunk_node.position.lod,
                    chunk_node.position.relative_position,
                    chunk.chunk_height + 1,
                    child_entity.id(),
                ),
                Transform::default(),
            ));
            let child_entity = child_entity.id();
            commands.entity(entity).add_child(child_entity);
        }

        let Some(parent) = chunk_node.parent else {
            continue;
        };

        if let Some(tree_children) = chunk_node.tree_children {
            commands.entity(tree_children.top_right).despawn();
            commands.entity(tree_children.top_left).despawn();
            commands.entity(tree_children.bottom_right).despawn();
            commands.entity(tree_children.bottom_left).despawn();
        }

        update_parent_count(parent, &mut all_chunk_nodes, &mut commands);
    }
}

fn update_parent_count(
    parent: Entity,
    chunk_nodes: &mut Vec<(Mut<ChunkNode>, Option<&Chunk>, Entity)>,
    commands: &mut Commands,
) {
    let parent_node = chunk_nodes
        .iter_mut()
        .find(|parent_node| parent_node.2 == parent);

    let Some((parent_node, _, parent_node_entity)) = parent_node else {
        return;
    };

    let child_count = parent_node
        .children_completion
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        + 1;

    if child_count >= 4 {
        remove_chunk_components(commands, *parent_node_entity);

        let Some(parent_parent) = parent_node.parent else {
            return;
        };

        update_parent_count(parent_parent, chunk_nodes, commands);
    }
}

fn remove_chunk_components(commands: &mut Commands, entity: Entity) {
    commands
        .entity(entity)
        .remove::<(ChunkGenerationTask, ChunkTaskGenerator, Chunk)>();
}
