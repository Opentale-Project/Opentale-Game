use crate::world_generation::chunk_loading::quad_tree_data::QuadTreeNode::Node;
use bevy::prelude::{Commands, Entity};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub enum QuadTreeNode<T> {
    Data(T, Vec<Entity>),
    Node {
        bottom_left: Box<QuadTreeNode<T>>,
        bottom_right: Box<QuadTreeNode<T>>,
        top_left: Box<QuadTreeNode<T>>,
        top_right: Box<QuadTreeNode<T>>,
        child_count: Arc<Mutex<i32>>,
        chunk_entities: Vec<Entity>,
    },
}

pub enum QuadTreeDistinction {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Into<i32> for QuadTreeDistinction {
    fn into(self) -> i32 {
        match self {
            QuadTreeDistinction::TopLeft => 0,
            QuadTreeDistinction::TopRight => 1,
            QuadTreeDistinction::BottomLeft => 2,
            QuadTreeDistinction::BottomRight => 3,
        }
    }
}

impl<T> QuadTreeNode<T> {
    pub fn add_to_parent(
        &mut self,
        depth: i32,
        position: [i32; 2],
        commands: &mut Commands,
    ) {
        let mut further = false;
        if let Some(Node {
            child_count: child_progress,
            chunk_entities: entities,
            ..
        }) = self.get_parent_node(depth, position)
        {
            let mut child_progress_lock = child_progress.lock().unwrap();
            *child_progress_lock += 1;

            if *child_progress_lock == 4 {
                for entity in entities {
                    if let Ok(mut entity) = commands.get_entity(entity.clone())
                    {
                        entity.despawn();
                    }
                }

                if depth != 1 {
                    further = true;
                }
            }
        }

        if further {
            self.add_to_parent(
                depth - 1,
                [position[0] / 2, position[1] / 2],
                commands,
            );
        }
    }

    pub fn get_parent_node(
        &mut self,
        depth: i32,
        position: [i32; 2],
    ) -> Option<&QuadTreeNode<T>> {
        if depth <= 1 {
            return Some(self);
        }

        return match self {
            QuadTreeNode::Data(_, _) => None,
            QuadTreeNode::Node {
                bottom_left,
                bottom_right,
                top_left,
                top_right,
                ..
            } => {
                let divider = 2_i32.pow(depth as u32 - 1);

                return if position[0] / divider == 0 {
                    if position[1] / divider == 0 {
                        bottom_left.get_parent_node(depth - 1, position)
                    } else {
                        top_left.get_parent_node(
                            depth - 1,
                            [position[0], position[1] - divider],
                        )
                    }
                } else {
                    if position[1] / divider == 0 {
                        bottom_right.get_parent_node(
                            depth - 1,
                            [position[0] - divider, position[1]],
                        )
                    } else {
                        top_right.get_parent_node(
                            depth - 1,
                            [position[0] - divider, position[1] - divider],
                        )
                    }
                };
            }
        };
    }

    pub fn get_node(
        &mut self,
        depth: i32,
        position: [i32; 2],
    ) -> Option<&mut Self> {
        if depth == 0 {
            return Some(self);
        }

        return match self {
            QuadTreeNode::Data(_, _) => None,
            QuadTreeNode::Node {
                bottom_left,
                bottom_right,
                top_left,
                top_right,
                ..
            } => {
                let divider = 2_i32.pow(depth as u32 - 1);

                return if position[0] / divider == 0 {
                    if position[1] / divider == 0 {
                        bottom_left.get_node(depth - 1, position)
                    } else {
                        top_left.get_node(
                            depth - 1,
                            [position[0], position[1] - divider],
                        )
                    }
                } else {
                    if position[1] / divider == 0 {
                        bottom_right.get_node(
                            depth - 1,
                            [position[0] - divider, position[1]],
                        )
                    } else {
                        top_right.get_node(
                            depth - 1,
                            [position[0] - divider, position[1] - divider],
                        )
                    }
                };
            }
        };
    }
}
