use std::hash::{DefaultHasher, Hash, Hasher};

use bevy::prelude::*;
use bevy_inspector_egui::{
    bevy_egui::{EguiContexts, EguiPrimaryContextPass},
    egui,
};

use crate::{
    ui::{
        initial_chunk_loader::{
            remove_initial_chunk_loader, spawn_initial_chunk_loader,
        },
        main_menu_data::MainMenuData,
        main_menu_state::{MainMenuState, hide_main_menu},
    },
    world_generation::{
        generation_options::GenerationOptionsResource,
        world_generation_state::WorldGenerationState,
    },
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
                OnEnter(MainMenuState::LoadingWorldGen),
                spawn_initial_chunk_loader,
            )
            .add_systems(
                OnExit(MainMenuState::LoadingWorldGen),
                remove_initial_chunk_loader,
            )
            .add_systems(OnEnter(WorldGenerationState::Active), hide_main_menu)
            .add_systems(
                EguiPrimaryContextPass,
                (
                    render_main_menu.run_if(in_state(MainMenuState::Shown)),
                    render_loading_screen
                        .run_if(in_state(MainMenuState::LoadingWorldGen)),
                ),
            );
    }
}

#[derive(Component)]
struct MenuCamera;

fn add_menu_cam(mut commands: Commands) {
    commands.spawn((Camera2d, MenuCamera));
}

fn remove_menu_cam(
    mut commands: Commands,
    cameras: Query<Entity, With<MenuCamera>>,
) {
    for camera in cameras {
        commands.entity(camera).despawn();
    }
}

fn render_main_menu(
    mut menu_data: ResMut<MainMenuData>,
    mut menu_state: ResMut<NextState<MainMenuState>>,
    mut commands: Commands,
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
                commands.insert_resource(GenerationOptionsResource::from_seed(
                    seed,
                ));

                menu_state.set(MainMenuState::LoadingWorldGen);
            }
        });
    });

    Ok(())
}

fn render_loading_screen(mut contexts: EguiContexts) -> Result {
    egui::CentralPanel::default().show(contexts.ctx_mut()?, |ui| {
        ui.vertical_centered(|ui| {
            ui.heading("Loading");
        });
    });

    Ok(())
}
