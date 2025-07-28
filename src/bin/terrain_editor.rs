use bevy::prelude::*;
use bevy_inspector_egui::{
    bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass},
    egui::{self},
};
use opentale::debug_tools::terrain_node_editor::terrain_graph_resource::TerrainGraphResource;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Opentale - Terrain Editor".into(),
                    ..default()
                }),
                ..default()
            }),
            EguiPlugin::default(),
        ))
        .add_systems(Startup, setup_camera)
        .add_systems(EguiPrimaryContextPass, render_terrain_editor)
        .init_resource::<TerrainGraphResource>()
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn render_terrain_editor(
    mut contexts: EguiContexts,
    mut terrain_graph: ResMut<TerrainGraphResource>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    let _graph_response = egui::CentralPanel::default()
        .show(ctx, |ui| terrain_graph.draw(ui))
        .inner;

    egui::TopBottomPanel::bottom("bottom").show(ctx, |ui| {
        if ui.button("Save").clicked() {
            terrain_graph.save().expect("Could not save!");
        }
    });

    Ok(())
}
