use bevy::prelude::*;

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct OpentaleDebugResource {
    pub unlock_camera: bool,
    pub show_path_debug: bool,
    pub path_circle_radius: f32,
    pub path_show_range: i32,
}

impl Default for OpentaleDebugResource {
    fn default() -> Self {
        Self {
            unlock_camera: false,
            show_path_debug: false,
            path_circle_radius: 1.,
            path_show_range: 500,
        }
    }
}
