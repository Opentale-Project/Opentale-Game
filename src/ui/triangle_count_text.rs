use bevy::prelude::*;

use crate::world_generation::chunk_generation::chunk_triangles::ChunkTriangles;

#[derive(Component)]
pub struct TriangleText;

pub fn update_triangle_ui(
    mut texts: Query<&mut Text, With<TriangleText>>,
    triangle_count: Res<ChunkTriangles>,
) {
    for mut text in &mut texts {
        text.0 = format!(
            "Triangles: {}, Total: {}",
            triangle_count
                .0
                .map(|x| x
                    .to_string()
                    .as_bytes()
                    .rchunks(3)
                    .rev()
                    .map(std::str::from_utf8)
                    .collect::<Result<Vec<&str>, _>>()
                    .unwrap()
                    .join("'"))
                .join(", "),
            triangle_count.0.iter().sum::<u64>()
        );
    }
}
