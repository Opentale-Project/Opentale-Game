use std::borrow::Cow;

use bevy_inspector_egui::egui;
use egui_node_editor::DataTypeTrait;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainDataType {
    NoiseF64x2,
    F64,
    I64,
}

impl DataTypeTrait<()> for TerrainDataType {
    fn data_type_color(&self, _: &mut ()) -> egui::Color32 {
        match self {
            TerrainDataType::NoiseF64x2 => egui::Color32::YELLOW,
            TerrainDataType::F64 => egui::Color32::BLUE,
            TerrainDataType::I64 => egui::Color32::DARK_GREEN,
        }
    }

    fn name(&self) -> std::borrow::Cow<str> {
        match self {
            TerrainDataType::NoiseF64x2 => Cow::Borrowed("Noise F64 x 2"),
            TerrainDataType::F64 => Cow::Borrowed("F64"),
            TerrainDataType::I64 => Cow::Borrowed("I64"),
        }
    }
}
