use std::{
    cell::RefCell,
    collections::HashMap,
    ops::Deref,
    sync::{Arc, atomic::AtomicI8},
};

use bevy::{pbr::ExtendedMaterial, prelude::*};
use bevy_rapier3d::prelude::{Collider, RigidBody};
use itertools::Itertools;

use crate::world_generation::{
    array_texture::ArrayTextureMaterial,
    chunk_generation::{
        CHUNK_SIZE, Chunk, ChunkGenerationTask, ChunkTaskGenerator, VOXEL_SIZE,
    },
    chunk_loading::{
        chunk_loader::ChunkLoader, chunk_node_children::ChunkNodeChildren,
        chunk_pos::AbsoluteChunkPos, chunk_tree::ChunkTreePos,
        lod_position::LodPosition, node_state::NodeState,
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
    state: NodeState,
    is_dead: bool,
}

impl Default for ChunkNode {
    fn default() -> Self {
        Self {
            tree_pos: Default::default(),
            position: Default::default(),
            parent: Default::default(),
            state: NodeState::Leaf {
                spawned_task: false,
                children: None,
            },
            is_dead: false,
        }
    }
}

impl ChunkNode {
    pub fn new(position: LodPosition, tree_pos: ChunkTreePos) -> Self {
        Self {
            position,
            parent: None,
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
            tree_pos,
            ..Default::default()
        }
    }

    pub fn to_branch(&mut self, node_children: ChunkNodeChildren) {
        self.state = NodeState::Branch {
            children: node_children,
            child_count: Arc::new(AtomicI8::new(0)),
        }
    }

    pub fn to_leaf(&mut self) {
        self.state = NodeState::Leaf {
            spawned_task: false,
            children: None,
        }
    }
}

pub fn check_for_division(
    mut commands: Commands,
    chunk_nodes: Query<(&mut ChunkNode, Entity)>,
    chunk_loaders: Query<(&ChunkLoader, &Transform)>,
) {
    for (mut chunk_node, chunk_node_entity) in chunk_nodes
        .into_iter()
        .filter(|node| node.0.state.is_leaf() && !node.0.is_dead)
        .sorted_by(|a, b| a.0.position.lod.cmp(&b.0.position.lod))
    {
        if commands.get_entity(chunk_node_entity).is_err() {
            continue;
        }

        let min_lod = chunk_loaders
            .iter()
            .map(|chunk_loader| {
                chunk_loader.0.get_min_lod(
                    chunk_loader.1.translation,
                    chunk_node.position,
                    chunk_node.tree_pos,
                )
            })
            .min();

        let Some(min_lod) = min_lod else {
            continue;
        };

        if min_lod < chunk_node.position.lod {
            devide_chunk_node(
                &mut chunk_node,
                chunk_node_entity,
                &mut commands,
            );

            continue;
        }

        if let NodeState::Leaf {
            ref mut spawned_task,
            children: _,
        } = chunk_node.state
            && !*spawned_task
        {
            *spawned_task = true;
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

pub fn check_for_merging(
    mut commands: Commands,
    chunk_nodes: Query<(&mut ChunkNode, Entity)>,
    chunk_loaders: Query<(&ChunkLoader, &Transform)>,
) {
    for (mut chunk_node, chunk_node_entity) in chunk_nodes
        .into_iter()
        .filter(|node| node.0.state.is_branch() && !node.0.is_dead)
        .sorted_by(|a, b| a.0.position.lod.cmp(&b.0.position.lod).reverse())
    {
        let min_lod = chunk_loaders
            .iter()
            .map(|chunk_loader| {
                chunk_loader.0.get_min_lod(
                    chunk_loader.1.translation,
                    chunk_node.position,
                    chunk_node.tree_pos,
                )
            })
            .min();

        let Some(min_lod) = min_lod else {
            continue;
        };

        if min_lod == chunk_node.position.lod
            && let NodeState::Branch { children, .. } = &chunk_node.state
        {
            for child in children.get_all() {
                commands.entity(child).despawn();
            }

            chunk_node.to_leaf();
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
                // Transform::from_translation(
                //     new_pos.get_absolute(chunk_node.tree_pos).extend(0.).xzy(),
                // ),
                Transform::default(),
                Visibility::Visible,
            ))
            .id()
    };

    let top_right = vec![spawn_child(chunk_node.position.to_top_right())];
    let top_left = vec![spawn_child(chunk_node.position.to_top_left())];
    let bottom_right = vec![spawn_child(chunk_node.position.to_bottom_right())];
    let bottom_left = vec![spawn_child(chunk_node.position.to_bottom_left())];

    chunk_node.to_branch(ChunkNodeChildren {
        top_right: top_right.clone(),
        top_left: top_left.clone(),
        bottom_right: bottom_right.clone(),
        bottom_left: bottom_left.clone(),
    });

    commands
        .entity(chunk_node_entity)
        .add_children(top_right.as_slice())
        .add_children(top_left.as_slice())
        .add_children(bottom_right.as_slice())
        .add_children(bottom_left.as_slice());

    commands
        .entity(chunk_node_entity)
        .remove::<(
            ChunkGenerationTask,
            ChunkTaskGenerator,
            Chunk,
            Mesh3d,
            MeshMaterial3d<
                ExtendedMaterial<StandardMaterial, ArrayTextureMaterial>,
            >,
            Collider,
            RigidBody,
        )>()
        .insert(Transform::default());
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

        // if let NodeState::Leaf { children, .. } = chunk_node.state
        //     && let Some(children) = children
        // {
        //     commands.entity(children.top_right).despawn();
        //     commands.entity(children.top_left).despawn();
        //     commands.entity(children.bottom_right).despawn();
        //     commands.entity(children.bottom_left).despawn();
        // }

        // update_parent_count(parent, &mut all_chunk_nodes, &mut commands);
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

    let NodeState::Branch {
        ref child_count, ..
    } = parent_node.state
    else {
        return;
    };

    let child_count =
        child_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;

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
