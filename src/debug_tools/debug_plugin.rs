use bevy::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;

use crate::debug_tools::{
    chunk_gizmos::{draw_path_gizmos, setup_gizmo_settings},
    debug_resource::OpentaleDebugResource,
};

pub struct OpentaleDebugPlugin;

impl Plugin for OpentaleDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OpentaleDebugResource>()
            .register_type::<OpentaleDebugResource>()
            .add_plugins(
                ResourceInspectorPlugin::<OpentaleDebugResource>::default(),
            )
            .add_systems(Startup, setup_gizmo_settings)
            .add_systems(Update, draw_path_gizmos);
    }
}
