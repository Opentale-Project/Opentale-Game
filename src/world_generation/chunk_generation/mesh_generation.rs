use crate::world_generation::array_texture::ATTRIBUTE_TEXTURE_ID;
use crate::world_generation::chunk_generation::block_type::{
    BlockFace, BlockType,
};
use crate::world_generation::chunk_generation::chunk_lod::ChunkLod;
use crate::world_generation::chunk_generation::voxel_data::VoxelData;
use crate::world_generation::chunk_generation::{CHUNK_SIZE, VOXEL_SIZE};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

pub struct MeshResult {
    pub opaque_mesh: Option<Mesh>,
    pub transparent_mesh: Option<Mesh>,
}

pub fn generate_mesh(
    voxel_data: &VoxelData,
    min_height: i32,
    chunk_lod: ChunkLod,
) -> MeshResult {
    let opaque_mesh = get_mesh_for_blocks(
        &[
            BlockType::Stone,
            BlockType::Grass,
            BlockType::Log,
            BlockType::Snow,
            BlockType::Dirt,
        ],
        voxel_data,
        min_height,
        chunk_lod,
    );

    let transparent_mesh = get_mesh_for_blocks(
        &[BlockType::Leaf],
        voxel_data,
        min_height,
        chunk_lod,
    );

    MeshResult {
        opaque_mesh,
        transparent_mesh,
    }
}

fn get_mesh_for_blocks(
    blocks: &[BlockType],
    voxel_data: &VoxelData,
    min_height: i32,
    chunk_lod: ChunkLod,
) -> Option<Mesh> {
    let mut mesh =
        Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all());

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut triangles: Vec<[u32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut texture_ids: Vec<u32> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();

    let mut generate_sides = |direction: IVec3, block_face: BlockFace| {
        for i in 1..CHUNK_SIZE + 1 {
            let mut done_faces = [[false; CHUNK_SIZE]; CHUNK_SIZE];
            for j in 1..CHUNK_SIZE + 1 {
                for k in 1..CHUNK_SIZE + 1 {
                    let current_pos = rotate_into_direction(
                        IVec3::new(i as i32, j as i32, k as i32),
                        direction,
                    );

                    let height_dir = rotate_into_direction(IVec3::Y, direction);
                    let width_dir = rotate_into_direction(IVec3::Z, direction);

                    let width_pos = (current_pos * width_dir).max_element();
                    let height_pos = (current_pos * height_dir).max_element();

                    let current_block = voxel_data.get_block(current_pos);

                    let [face_x, face_y] =
                        [width_pos as usize - 1, height_pos as usize - 1];
                    if done_faces[face_x][face_y]
                        || !blocks.contains(&current_block)
                        || voxel_data
                            .get_block(current_pos + direction)
                            .is_covering_for(&current_block)
                    {
                        continue;
                    }

                    let ambient_occlusion = voxel_data
                        .get_ambiant_occlusion(current_pos, direction);

                    let mut height = 1;
                    let mut width = 1;

                    while height_pos + height <= CHUNK_SIZE as i32
                        && !done_faces[width_pos as usize - 1]
                            [height_pos as usize + height as usize - 1]
                        && voxel_data
                            .get_block(current_pos + (height_dir * height))
                            == current_block
                        && !voxel_data
                            .get_block(
                                current_pos + (height_dir * height) + direction,
                            )
                            .is_covering_for(&current_block)
                        && voxel_data.get_ambiant_occlusion(
                            current_pos + (height_dir * height),
                            direction,
                        ) == ambient_occlusion
                    {
                        height += 1;
                    }

                    while width_pos + width <= CHUNK_SIZE as i32
                        && (0..height).all(|height| {
                            let [face_x, face_y] = [
                                width_pos as usize + width as usize - 1,
                                height_pos as usize + height as usize - 1,
                            ];
                            !done_faces[face_x][face_y]
                                && voxel_data.get_block(
                                    current_pos
                                        + (width_dir * width as i32)
                                        + (height_dir * height as i32),
                                ) == current_block
                                && !voxel_data
                                    .get_block(
                                        current_pos
                                            + (width_dir * width as i32)
                                            + (height_dir * height as i32)
                                            + direction,
                                    )
                                    .is_covering_for(&current_block)
                                && voxel_data.get_ambiant_occlusion(
                                    current_pos
                                        + (width_dir * width as i32)
                                        + (height_dir * height as i32),
                                    direction,
                                ) == ambient_occlusion
                        })
                    {
                        width += 1;
                    }

                    for x in width_pos..width_pos + width {
                        for y in height_pos..height_pos + height {
                            done_faces[x as usize - 1][y as usize - 1] = true;
                        }
                    }

                    let uv_start = Vec2::ZERO;
                    let uv_end = Vec2::new(width as f32, height as f32)
                        * chunk_lod.multiplier_f32();

                    uvs.extend_from_slice(&[
                        [uv_end.x, uv_end.y],
                        [uv_start.x, uv_end.y],
                        [uv_start.x, uv_start.y],
                        [uv_end.x, uv_start.y],
                    ]);

                    let height = height as f32 - 1.;
                    let width = width as f32 - 1.;

                    let positions_count = positions.len() as u32;

                    let vertex_pos = current_pos.as_vec3();

                    let direction_adder =
                        direction * direction.min_element().abs();

                    let vecs = &[
                        Vec3::new(0.5, -0.5, -0.5),
                        Vec3::new(0.5, -0.5, 0.5 + width),
                        Vec3::new(0.5, 0.5 + height, 0.5 + width),
                        Vec3::new(0.5, 0.5 + height, -0.5),
                    ];

                    positions.extend_from_slice(
                        vecs.into_iter()
                            .map(|e| {
                                (vertex_pos
                                    + rotate_into_direction(*e, direction)
                                    + direction_adder.as_vec3())
                                .to_array()
                            })
                            .collect::<Vec<_>>()
                            .as_slice(),
                    );

                    normals.extend_from_slice(&[
                        direction.as_vec3().to_array(),
                        direction.as_vec3().to_array(),
                        direction.as_vec3().to_array(),
                        direction.as_vec3().to_array(),
                    ]);

                    let texture_id = current_block.get_texture_id(block_face);

                    texture_ids.extend_from_slice(&[
                        texture_id, texture_id, texture_id, texture_id,
                    ]);

                    let mut invert = !direction.min_element() < 0;

                    if matches!(block_face, BlockFace::Right | BlockFace::Left)
                    {
                        invert = !invert;
                    }

                    colors.extend_from_slice(
                        ambient_occlusion.get_colors().as_slice(),
                    );

                    if ambient_occlusion.turn_quad() {
                        triangles.extend_from_slice(&[
                            [
                                positions_count + 0,
                                positions_count + if invert { 1 } else { 2 },
                                positions_count + if invert { 2 } else { 1 },
                            ],
                            [
                                positions_count + 0,
                                positions_count + if invert { 2 } else { 3 },
                                positions_count + if invert { 3 } else { 2 },
                            ],
                        ]);
                    } else {
                        triangles.extend_from_slice(&[
                            [
                                positions_count + 0,
                                positions_count + if invert { 1 } else { 3 },
                                positions_count + if invert { 3 } else { 1 },
                            ],
                            [
                                positions_count + 1,
                                positions_count + if invert { 2 } else { 3 },
                                positions_count + if invert { 3 } else { 2 },
                            ],
                        ]);
                    }
                }
            }
        }
    };

    generate_sides(IVec3::X, BlockFace::Right);
    generate_sides(IVec3::NEG_X, BlockFace::Left);
    generate_sides(IVec3::Z, BlockFace::Front);
    generate_sides(IVec3::NEG_Z, BlockFace::Back);
    generate_sides(IVec3::Y, BlockFace::Top);
    generate_sides(IVec3::NEG_Y, BlockFace::Bottom);

    if triangles.is_empty() {
        return None;
    }

    for position in positions.iter_mut() {
        position[0] =
            (position[0] - 0.5) * VOXEL_SIZE * chunk_lod.multiplier_f32()
                + VOXEL_SIZE;
        position[1] = (position[1] + min_height as f32 - 0.5)
            * VOXEL_SIZE
            * chunk_lod.multiplier_f32()
            + VOXEL_SIZE;
        position[2] =
            (position[2] - 0.5) * VOXEL_SIZE * chunk_lod.multiplier_f32()
                + VOXEL_SIZE;
    }

    let mut mesh_triangles: Vec<u32> = Vec::new();

    for triangle in &triangles {
        mesh_triangles.push(triangle[0]);
        mesh_triangles.push(triangle[1]);
        mesh_triangles.push(triangle[2]);
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_attribute(ATTRIBUTE_TEXTURE_ID, texture_ids);

    mesh.insert_indices(Indices::U32(mesh_triangles));

    Some(mesh)
}

pub fn rotate_into_direction<T: Vec3Swizzles>(
    vector: T,
    direction: IVec3,
) -> T {
    match direction {
        IVec3::X | IVec3::NEG_X => vector.xyz(),
        IVec3::Y | IVec3::NEG_Y => vector.yxz(),
        IVec3::Z | IVec3::NEG_Z => vector.zyx(),
        _ => vector,
    }
}
