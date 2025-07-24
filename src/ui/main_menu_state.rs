use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum MainMenuState {
    #[default]
    Shown,
    LoadingWorldGen,
    Hidden,
}

pub fn hide_main_menu(mut main_menu_state: ResMut<NextState<MainMenuState>>) {
    main_menu_state.set(MainMenuState::Hidden);
}
