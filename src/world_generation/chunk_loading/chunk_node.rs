use std::sync::{Arc, atomic::AtomicI8};

use bevy::prelude::*;
use itertools::Itertools;

use crate::world_generation::{
    chunk_generation::{chunk::Chunk, chunk_start::ChunkStart},
    chunk_loading::{
        chunk_loader::ChunkLoader, chunk_node_children::ChunkNodeChildren,
        chunk_tree::ChunkTreePos, lod_position::LodPosition,
        node_state::NodeState,
    },
};

/// The Chunk Node component represents a branch in the Quad-Tree.
#[derive(Component, Clone)]
pub struct ChunkNode {
    tree_pos: ChunkTreePos,
    position: LodPosition,
    parent: Option<Entity>,
    state: NodeState,
    is_dead: bool,
    chunk_children: Vec<Entity>,
}

impl Default for ChunkNode {
    fn default() -> Self {
        Self {
            tree_pos: Default::default(),
            position: Default::default(),
            parent: Default::default(),
            state: NodeState::Leaf {
                spawned_task: false,
            },
            is_dead: false,
            chunk_children: Vec::new(),
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
        self.state = NodeState::LeafToBranch {
            children: node_children,
            child_count: Arc::new(AtomicI8::new(0)),
        }
    }

    pub fn to_branch_done(&mut self) {
        let NodeState::LeafToBranch { children, .. } = &self.state else {
            return;
        };

        self.state = NodeState::Branch {
            children: children.clone(),
        }
    }

    pub fn to_leaf(&mut self) {
        let NodeState::Branch { children } = &self.state else {
            return;
        };

        self.state = NodeState::BranchToLeaf {
            spawned_task: false,
            children: children.clone(),
        }
    }

    pub fn to_leaf_done(&mut self) {
        let NodeState::BranchToLeaf { spawned_task, .. } = &self.state else {
            return;
        };

        self.state = NodeState::Leaf {
            spawned_task: *spawned_task,
        }
    }
}

pub fn check_for_task_spawning(
    mut commands: Commands,
    chunk_nodes: Query<(&mut ChunkNode, Entity)>,
) {
    for (mut chunk_node, chunk_node_entity) in
        chunk_nodes.into_iter().filter(|node| {
            !node.0.is_added()
                && match node.0.state {
                    NodeState::Leaf { spawned_task } => !spawned_task,
                    NodeState::BranchToLeaf { spawned_task, .. } => {
                        !spawned_task
                    }
                    _ => false,
                }
        })
    {
        if let NodeState::Leaf {
            ref mut spawned_task,
        } = chunk_node.state
        {
            *spawned_task = true;
        }

        if let NodeState::BranchToLeaf {
            ref mut spawned_task,
            ..
        } = chunk_node.state
        {
            *spawned_task = true;
        }

        let chunk_child_entity = commands.spawn_empty().id();
        commands.entity(chunk_child_entity).insert((
            ChunkStart {
                chunk_lod_pos: chunk_node.position,
                chunk_tree_pos: chunk_node.tree_pos,
                chunk_stack_offset: 0,
            },
            Visibility::Visible,
        ));

        commands
            .entity(chunk_node_entity)
            .add_child(chunk_child_entity);

        chunk_node.chunk_children.push(chunk_child_entity);
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
    }
}

pub fn check_for_merging(
    mut chunk_nodes: Query<(&mut ChunkNode, Entity)>,
    chunk_loaders: Query<(&ChunkLoader, &Transform)>,
) {
    let mut children_to_die = Vec::new();

    for (mut chunk_node, _) in chunk_nodes
        .iter_mut()
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
            children_to_die.extend(children.get_all());

            chunk_node.to_leaf();
        }
    }

    while children_to_die.len() > 0 {
        let current_children_to_die = children_to_die.clone();
        children_to_die.clear();

        for child_entity in current_children_to_die {
            let (mut child_node, ..) = chunk_nodes
                .iter_mut()
                .find(|node| node.1 == child_entity)
                .expect("Child not found!");

            child_node.is_dead = true;

            if let NodeState::Branch { children } = &child_node.state {
                children_to_die.extend(children.get_all());
            }

            if let NodeState::BranchToLeaf { children, .. } = &child_node.state
            {
                children_to_die.extend(children.get_all());
            }

            if let NodeState::LeafToBranch { children, .. } = &child_node.state
            {
                children_to_die.extend(children.get_all());
            }
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
                Transform::default(),
                Visibility::Visible,
            ))
            .id()
    };

    let top_right = spawn_child(chunk_node.position.to_top_right());
    let top_left = spawn_child(chunk_node.position.to_top_left());
    let bottom_right = spawn_child(chunk_node.position.to_bottom_right());
    let bottom_left = spawn_child(chunk_node.position.to_bottom_left());

    chunk_node.to_branch(ChunkNodeChildren {
        top_right: top_right,
        top_left: top_left,
        bottom_right: bottom_right,
        bottom_left: bottom_left,
    });

    commands.entity(chunk_node_entity).add_children(&[
        top_right,
        top_left,
        bottom_right,
        bottom_left,
    ]);
}

pub fn stack_chunks(
    mut commands: Commands,
    added_stacked_chunks: Query<(&Chunk, &ChildOf), Added<Chunk>>,
    mut chunk_nodes: Query<(&mut ChunkNode, Entity)>,
) {
    for (chunk, ChildOf(chunk_node_parent)) in added_stacked_chunks {
        if !chunk.generate_above {
            continue;
        }

        let (mut parent_node, _) = chunk_nodes
            .iter_mut()
            .find(|node| node.1 == *chunk_node_parent)
            .expect("Parent not found!");

        let chunk_child_entity = commands.spawn_empty().id();
        commands.entity(chunk_child_entity).insert((
            ChunkStart {
                chunk_lod_pos: chunk.lod_position,
                chunk_tree_pos: chunk.tree_position,
                chunk_stack_offset: chunk.chunk_height + 1,
            },
            Visibility::Visible,
        ));

        commands
            .entity(*chunk_node_parent)
            .add_child(chunk_child_entity);

        parent_node.chunk_children.push(chunk_child_entity);
    }
}

pub fn update_added_chunks(
    mut commands: Commands,
    added_chunks: Query<(&Chunk, &ChildOf), Added<Chunk>>,
    chunk_nodes: Query<(&mut ChunkNode, Entity)>,
) {
    let mut all_nodes = chunk_nodes
        .into_iter()
        .filter(|node| match node.0.state {
            NodeState::LeafToBranch { .. } => true,
            NodeState::Leaf { .. } => true,
            NodeState::BranchToLeaf { .. } => true,
            _ => false,
        })
        .collect_vec();

    for (_, ChildOf(added_chunk_parent)) in added_chunks
        .iter()
        .filter(|chunk| chunk.0.generate_above == false)
    {
        let (chunk_node, ..) = all_nodes
            .iter_mut()
            .find(|node| node.1 == *added_chunk_parent)
            .expect("No Node found for Chunk!");

        if let NodeState::BranchToLeaf { children, .. } = &chunk_node.state {
            for child in children.get_all() {
                commands.entity(child).despawn();
            }

            chunk_node.to_leaf_done();
        }

        if let Some(chunk_node_parent) = chunk_node.parent {
            update_parent_count(
                chunk_node_parent,
                &mut all_nodes,
                &mut commands,
            );
        }
    }
}

fn update_parent_count(
    parent: Entity,
    chunk_nodes: &mut Vec<(Mut<ChunkNode>, Entity)>,
    commands: &mut Commands,
) {
    let parent_node = chunk_nodes
        .iter_mut()
        .find(|parent_node| parent_node.1 == parent);

    let Some((parent_node, _)) = parent_node else {
        return;
    };

    match &parent_node.state {
        NodeState::BranchToLeaf { children, .. } => {
            for child in children.get_all() {
                commands.entity(child).despawn();
            }

            parent_node.to_leaf_done();

            if let Some(parent_parent) = parent_node.parent {
                update_parent_count(parent_parent, chunk_nodes, commands);
            }
        }
        NodeState::LeafToBranch { child_count, .. } => {
            let child_count = child_count
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                + 1;

            if child_count >= 4 {
                parent_node.to_branch_done();

                for chunk_child in &parent_node.chunk_children {
                    // We use a try-despawn here, because the chunks can despawn themselves when they end up not having a mesh at all
                    commands.entity(*chunk_child).try_despawn();
                }

                parent_node.chunk_children.clear();

                let Some(parent_parent) = parent_node.parent else {
                    return;
                };

                update_parent_count(parent_parent, chunk_nodes, commands);
            }
        }
        _ => {}
    }
}
