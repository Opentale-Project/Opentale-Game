use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
};

use anyhow::anyhow;
use bevy::prelude::*;
use bevy_inspector_egui::egui::Ui;
use egui_node_editor::{GraphEditorState, GraphResponse, Node, OutputId};
use ron::ser::PrettyConfig;

use crate::{
    debug_tools::terrain_node_editor::{
        all_terrain_node_templates::AllTerrainNodeTemplates,
        terrain_data_type::TerrainDataType,
        terrain_node_data::TerrainNodeData,
        terrain_node_template::{TerrainGraph, TerrainNodeTemplate},
        terrain_response::TerrainResponse,
        terrain_value_type::TerrainValueType,
    },
    world_generation::chunk_generation::{
        VOXEL_SIZE,
        noise::{
            terrain_noise::{TERRAIN_NOISE_FILE_PATH, TerrainNoise},
            terrain_noise_type::TerrainNoiseType,
        },
    },
};

type TerrainGraphState =
    GraphEditorState<TerrainNodeData, TerrainDataType, TerrainValueType, TerrainNodeTemplate, ()>;

#[derive(Resource, Deref, DerefMut)]
pub struct TerrainGraphResource {
    state: TerrainGraphState,
}

const TERRAIN_NOISE_GRAPH_FILE_PATH: &'static str = "assets/terrain_noise_graph.ron";

impl Default for TerrainGraphResource {
    fn default() -> Self {
        let file = File::open(TERRAIN_NOISE_GRAPH_FILE_PATH);
        let Ok(mut file) = file else {
            return Self {
                state: Default::default(),
            };
        };

        let mut output_string = String::new();
        let read_result = file.read_to_string(&mut output_string);
        if read_result.is_err() {
            return Self {
                state: Default::default(),
            };
        }

        let state: TerrainGraphState =
            ron::from_str(&output_string).expect("cannot read terrain data");

        Self { state }
    }
}

impl TerrainGraphResource {
    pub fn draw(&mut self, ui: &mut Ui) -> GraphResponse<TerrainResponse, TerrainNodeData> {
        self.state
            .draw_graph_editor(ui, AllTerrainNodeTemplates, &mut (), Vec::default())
    }

    pub fn save(&self) -> Result<(), anyhow::Error> {
        let output_nodes = self
            .graph
            .nodes
            .iter()
            .filter(|node| node.1.user_data.template == TerrainNodeTemplate::Output)
            .collect::<Vec<_>>();

        if output_nodes.len() != 1 {
            return Err(anyhow!("Too many or too little number of output nodes!"));
        }

        let output_node = output_nodes.first();
        let Some(output_node) = output_node else {
            return Err(anyhow!("No output node found!"));
        };

        let mut noise_array = Vec::new();
        let mut cache = HashMap::new();

        let start_index =
            get_terrain_noise_index(output_node.1, &mut noise_array, &self.graph, &mut cache);

        let terrain_noise = TerrainNoise::new(start_index.get_noise_index(), noise_array);

        let mut file = File::create(TERRAIN_NOISE_FILE_PATH)?;
        let text = ron::ser::to_string_pretty(&terrain_noise, PrettyConfig::default())?;
        file.write_all(text.as_bytes())?;
        file.flush()?;

        let mut file = File::create(TERRAIN_NOISE_GRAPH_FILE_PATH)?;
        let text = ron::ser::to_string_pretty(&self.state, PrettyConfig::default())?;
        file.write_all(text.as_bytes())?;
        file.flush()?;

        Ok(())
    }
}

fn get_terrain_noise_index(
    node: &Node<TerrainNodeData>,
    noise_array: &mut Vec<TerrainNoiseType>,
    graph: &TerrainGraph,
    value_cache: &mut HashMap<OutputId, TerrainValueType>,
) -> TerrainValueType {
    let mut get_input_value = |input_name: &str| {
        let input_id = node.get_input(input_name).expect("Input not found!");
        let connection = graph.connection(input_id);

        let Some(connection) = connection else {
            return graph[input_id].value;
        };

        if let Some(cached_value) = value_cache.get(&connection) {
            return *cached_value;
        }

        let node = &graph[graph[connection].node];
        let value = get_terrain_noise_index(node, noise_array, graph, value_cache);
        value_cache.insert(connection, value);
        value
    };

    match node.user_data.template {
        TerrainNodeTemplate::Output => get_input_value("A"),
        TerrainNodeTemplate::SimplexNoise => {
            let noise_index = noise_array.len();
            noise_array.push(TerrainNoiseType::Simplex);
            TerrainValueType::NoiseF64x2 { noise_index }
        }
        TerrainNodeTemplate::NoiseAdd => {
            let a_index = get_input_value("A").get_noise_index();
            let b_index = get_input_value("B").get_noise_index();
            let noise_index = noise_array.len();
            noise_array.push(TerrainNoiseType::Add { a_index, b_index });
            TerrainValueType::NoiseF64x2 { noise_index }
        }
        TerrainNodeTemplate::PowF64 => {
            let a_value = get_input_value("A").get_f64_value();
            let b_value = get_input_value("B").get_f64_value();
            TerrainValueType::F64 {
                value: a_value.powf(b_value),
            }
        }
        TerrainNodeTemplate::Constant => {
            let a_value = get_input_value("A").get_f64_value();
            let noise_index = noise_array.len();
            noise_array.push(TerrainNoiseType::Constant { value: a_value });
            TerrainValueType::NoiseF64x2 { noise_index }
        }
        TerrainNodeTemplate::Multiply => {
            let a_index = get_input_value("A").get_noise_index();
            let b_index = get_input_value("B").get_noise_index();
            let noise_index = noise_array.len();
            noise_array.push(TerrainNoiseType::Multiply { a_index, b_index });
            TerrainValueType::NoiseF64x2 { noise_index }
        }
        TerrainNodeTemplate::Max => {
            let a_index = get_input_value("A").get_noise_index();
            let b_index = get_input_value("B").get_noise_index();
            let noise_index = noise_array.len();
            noise_array.push(TerrainNoiseType::Max { a_index, b_index });
            TerrainValueType::NoiseF64x2 { noise_index }
        }
        TerrainNodeTemplate::SmoothStep => {
            let noise_index = get_input_value("noise").get_noise_index();
            let steps = get_input_value("steps").get_f64_value();
            let smoothness = get_input_value("smoothness").get_f64_value();
            let index = noise_array.len();
            noise_array.push(TerrainNoiseType::SmoothStep {
                noise_index,
                steps,
                smoothness,
            });
            TerrainValueType::NoiseF64x2 { noise_index: index }
        }
        TerrainNodeTemplate::ScalePoint => {
            let noise_index = get_input_value("noise").get_noise_index();
            let scale = get_input_value("scale").get_f64_value();
            let index = noise_array.len();
            noise_array.push(TerrainNoiseType::ScalePoint { noise_index, scale });
            TerrainValueType::NoiseF64x2 { noise_index: index }
        }
        TerrainNodeTemplate::GFT => {
            let noise_index = get_input_value("noise").get_noise_index();
            let octaves = get_input_value("octaves").get_i64_value();
            let frequency = get_input_value("frequency").get_f64_value();
            let lacunarity = get_input_value("lacunarity").get_f64_value();
            let persistence = get_input_value("persistence").get_f64_value();
            let gradient = get_input_value("gradient").get_f64_value();
            let amplitude = get_input_value("amplitude").get_f64_value();
            let index = noise_array.len();
            noise_array.push(TerrainNoiseType::GFT {
                noise_index,
                octaves: octaves as usize,
                frequency,
                lacunarity,
                persistence,
                gradient,
                amplitude,
            });
            TerrainValueType::NoiseF64x2 { noise_index: index }
        }
        TerrainNodeTemplate::VoxelSize => TerrainValueType::F64 {
            value: VOXEL_SIZE as f64,
        },
    }
}
