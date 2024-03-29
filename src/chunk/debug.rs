use bevy::prelude::*;

use crate::block::BLOCK_HALF_SIZE;

use crate::chunk::ChunkDimensions;
use crate::config::GameConfig;

use super::ChunkMarker;

#[derive(Resource, PartialEq, Eq, Clone)]
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
    chunk_query: Query<&GlobalTransform, With<ChunkMarker>>,
) {
    match *show_chunks {
        ShowChunks::DontShow => {}
        ShowChunks::Show => {
            gizmos.primitive_3d(
                Line3d {
                    direction: Direction3d::Y,
                },
                Vec3::splat(0.0),
                Quat::IDENTITY,
                Color::RED,
            );
            gizmos.primitive_3d(
                Line3d {
                    direction: Direction3d::Z,
                },
                Vec3::splat(0.0),
                Quat::IDENTITY,
                Color::BLUE,
            );
            gizmos.primitive_3d(
                Line3d {
                    direction: Direction3d::X,
                },
                Vec3::splat(0.0),
                Quat::IDENTITY,
                Color::GREEN,
            );
            for transform in chunk_query.iter() {
                gizmos.primitive_3d(
                    Cuboid {
                        half_size: Vec3::new(
                            chunk_dimensions.width as f32 * BLOCK_HALF_SIZE,
                            chunk_dimensions.height as f32 * BLOCK_HALF_SIZE,
                            chunk_dimensions.depth as f32 * BLOCK_HALF_SIZE,
                        ),
                    },
                    transform.translation(),
                    Quat::IDENTITY,
                    Color::WHITE,
                )
            }
        }
    }
}
