use std::fmt::Debug;

use bevy_inspector_egui::egui::{self, DragValue};
use egui_node_editor::{NodeId, WidgetValueTrait};
use serde::{Deserialize, Serialize};

use crate::debug_tools::terrain_node_editor::{
    terrain_node_data::TerrainNodeData, terrain_response::TerrainResponse,
};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum TerrainValueType {
    NoiseF64x2 { noise_index: usize },
    F64 { value: f64 },
    I64 { value: i64 },
}

impl TerrainValueType {
    pub fn get_noise_index(&self) -> usize {
        match self {
            TerrainValueType::NoiseF64x2 { noise_index } => *noise_index,
            _ => 0,
        }
    }

    pub fn get_f64_value(&self) -> f64 {
        match self {
            TerrainValueType::F64 { value } => *value,
            _ => 0.0,
        }
    }

    pub fn get_i64_value(&self) -> i64 {
        match self {
            TerrainValueType::I64 { value } => *value,
            _ => 0,
        }
    }
}

impl Default for TerrainValueType {
    fn default() -> Self {
        Self::NoiseF64x2 { noise_index: 0 }
    }
}

impl WidgetValueTrait for TerrainValueType {
    type Response = TerrainResponse;
    type UserState = ();
    type NodeData = TerrainNodeData;
    fn value_widget(
        &mut self,
        param_name: &str,
        _node_id: NodeId,
        ui: &mut egui::Ui,
        _user_state: &mut (),
        _node_data: &TerrainNodeData,
    ) -> Vec<Self::Response> {
        match self {
            TerrainValueType::NoiseF64x2 { noise_index: _ } => {
                ui.label(param_name);
            }
            TerrainValueType::F64 { value } => {
                ui.label(param_name);
                ui.add(DragValue::new(value));
            }
            TerrainValueType::I64 { value } => {
                ui.label(param_name);
                ui.add(DragValue::new(value));
            }
        }
        Vec::new()
    }
}
