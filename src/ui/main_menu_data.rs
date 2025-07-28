use bevy::prelude::*;

#[derive(Resource)]
pub struct MainMenuData {
    pub seed: String,
}

impl Default for MainMenuData {
    fn default() -> Self {
        Self {
            seed: "Seed".into(),
        }
    }
}
