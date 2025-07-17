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
    pub top_right: Vec<Entity>,
    pub top_left: Vec<Entity>,
    pub bottom_right: Vec<Entity>,
    pub bottom_left: Vec<Entity>,
}

impl ChunkNodeChildren {
    pub fn get_all(&self) -> impl Iterator<Item = Entity> {
        self.top_right
            .iter()
            .chain(self.top_left.iter())
            .chain(self.bottom_right.iter())
            .chain(self.bottom_left.iter())
            .cloned()
    }
}
