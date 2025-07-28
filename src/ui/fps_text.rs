use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::{Component, Query, Res, Text, With};

#[derive(Component)]
pub struct FpsText;

pub fn update_fps_ui(
    mut texts: Query<&mut Text, With<FpsText>>,
    diagnostics: Res<DiagnosticsStore>,
) {
    for mut text in &mut texts {
        if let Some(fps) = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
        {
            text.0 = format!("FPS: {:.0}", fps);
        }
    }
}
