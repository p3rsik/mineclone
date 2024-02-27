use bevy::prelude::*;

use crate::chunk::Chunk;
use crate::chunk::ChunkDimensions;
use crate::config::GameConfig;

#[derive(Resource)]
pub enum ShowChunks {
    Show,
    DontShow,
}

pub fn toggle_show_chunks(
    mut show_chunks: ResMut<ShowChunks>,
    k_input: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
) {
    if k_input.just_pressed(config.key_config.debug_controls.toggle_chunk_border) {
        match *show_chunks {
            ShowChunks::Show => *show_chunks = ShowChunks::DontShow,
            ShowChunks::DontShow => *show_chunks = ShowChunks::Show,
        }
    }
}

pub fn show_chunk_border(
    mut gizmos: Gizmos,
    show_chunks: Res<ShowChunks>,
    chunk_dimensions: Res<ChunkDimensions>,
    chunk_query: Query<&Transform, With<Chunk>>,
) {
    match *show_chunks {
        ShowChunks::DontShow => {}
        ShowChunks::Show => {
            for transform in chunk_query.iter() {
                let x = transform.translation.x + chunk_dimensions.width as f32 * 0.5 / 2.0;
                let y = transform.translation.y + chunk_dimensions.height as f32 * 0.5 / 2.0;
                let z = transform.translation.z + chunk_dimensions.depth as f32 * 0.5 / 2.0;
                let origin = Vec3::new(x, y, z);
                gizmos.primitive_3d(
                    Cuboid {
                        half_size: Vec3::new(
                            chunk_dimensions.width as f32 * 0.5,
                            chunk_dimensions.height as f32 * 0.5,
                            chunk_dimensions.depth as f32 * 0.5,
                        ),
                    },
                    origin,
                    Quat::IDENTITY,
                    Color::WHITE,
                )
            }
        }
    }
}
