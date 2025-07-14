use bevy_inspector_egui::egui;
use egui_node_editor::{Graph, NodeDataTrait, NodeId, NodeResponse};
use serde::{Deserialize, Serialize};

use crate::debug_tools::terrain_node_editor::{
    terrain_data_type::TerrainDataType, terrain_node_template::TerrainNodeTemplate,
    terrain_response::TerrainResponse, terrain_value_type::TerrainValueType,
};

#[derive(Serialize, Deserialize)]
pub struct TerrainNodeData {
    pub template: TerrainNodeTemplate,
}

impl NodeDataTrait for TerrainNodeData {
    type Response = TerrainResponse;

    type UserState = ();

    type DataType = TerrainDataType;

    type ValueType = TerrainValueType;

    fn bottom_ui(
        &self,
        _ui: &mut egui::Ui,
        _node_id: NodeId,
        _graph: &Graph<Self, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
    ) -> Vec<NodeResponse<Self::Response, Self>>
    where
        Self::Response: egui_node_editor::UserResponseTrait,
    {
        vec![]
    }
}
