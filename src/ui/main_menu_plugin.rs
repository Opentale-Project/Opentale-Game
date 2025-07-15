use std::hash::{DefaultHasher, Hash, Hasher};

use bevy::prelude::*;
use bevy_inspector_egui::{
    bevy_egui::{EguiContexts, EguiPrimaryContextPass},
    egui,
};

use crate::{
    ui::{main_menu_data::MainMenuData, main_menu_state::MainMenuState},
    world_generation::generation_options::GenerationOptionsResource,
};

#[derive(Default)]
pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<MainMenuState>()
            .init_resource::<MainMenuData>()
            .add_systems(OnEnter(MainMenuState::Shown), add_menu_cam)
            .add_systems(OnEnter(MainMenuState::Hidden), remove_menu_cam)
            .add_systems(
                EguiPrimaryContextPass,
                render_main_menu.run_if(in_state(MainMenuState::Shown)),
            );
    }
}

#[derive(Component)]
struct MenuCamera;

fn add_menu_cam(mut commands: Commands) {
    commands.spawn((Camera2d, MenuCamera));
}

fn remove_menu_cam(mut commands: Commands, cameras: Query<Entity, With<MenuCamera>>) {
    for camera in cameras {
        commands.entity(camera).despawn();
    }
}

fn render_main_menu(
    mut menu_data: ResMut<MainMenuData>,
    mut menu_state: ResMut<NextState<MainMenuState>>,
    mut gen_options: ResMut<GenerationOptionsResource>,
    mut contexts: EguiContexts,
) -> Result {
    egui::CentralPanel::default().show(contexts.ctx_mut()?, |ui| {
        ui.vertical_centered(|ui| {
            ui.heading("SpellHaven");

            ui.text_edit_singleline(&mut menu_data.seed);
            if ui.button("Start").clicked() {
                let mut hasher = DefaultHasher::new();
                menu_data.seed.hash(&mut hasher);
                let seed = hasher.finish();

                info!("Seed to use: {}", seed);
                *gen_options = GenerationOptionsResource::from_seed(seed);

                menu_state.set(MainMenuState::Hidden);
            }
        });
    });

    Ok(())
}
