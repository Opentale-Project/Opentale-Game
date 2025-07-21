use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct ChunkNodeQueryStepper {
    pub current_steps: usize,
}
