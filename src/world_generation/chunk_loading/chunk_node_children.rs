use bevy::prelude::*;

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
#[derive(Clone)]
pub struct ChunkNodeChildren {
    pub top_right: Entity,
    pub top_left: Entity,
    pub bottom_right: Entity,
    pub bottom_left: Entity,
}

impl ChunkNodeChildren {
    pub fn get_all(&self) -> impl Iterator<Item = Entity> {
        [
            self.top_right,
            self.top_left,
            self.bottom_right,
            self.bottom_left,
        ]
        .into_iter()
    }
}
