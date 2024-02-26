use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{config::GameConfig, player::Player};

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChunkDimensions::default())
            .add_systems(Update, load_chunks)
            .add_systems(First, (track_player_chunk, mark_chunks))
            .add_systems(Last, unload_chunks);
    }
}

#[derive(Component)]
pub struct Chunk;

#[derive(Component)]
// Marker for chunks that needs to be loaded
pub struct LoadedChunk;

#[derive(Component)]
pub struct NotLoadedChunk;

#[derive(Resource, Clone)]
// Dimensions must be even
pub struct ChunkDimensions {
    pub width: usize,
    pub height: usize,
    pub depth: usize,
}

impl Default for ChunkDimensions {
    fn default() -> Self {
        ChunkDimensions {
            width: 2,
            height: 2,
            depth: 2,
        }
    }
}

#[derive(Component, Clone, PartialEq, Debug)]
// this is a coordinates of the chunk that do not relate to global transform
pub struct ChunkTranslation {
    pub x: isize,
    pub y: isize,
    pub z: isize,
}

#[derive(Bundle)]
pub struct ChunkBundle {
    pub translation: ChunkTranslation,
    pub chunk: Chunk,
}

fn track_player_chunk(
    mut player_query: Query<(&Transform, &mut ChunkTranslation), With<Player>>,
    chunk_dimensions: Res<ChunkDimensions>,
) {
    let (player_transofrm, mut chunk_translation) = player_query.single_mut();
    let translation = player_transofrm.translation;
    let x = {
        let x = translation.x.round() as isize;
        if x >= 0 {
            x - (x / chunk_dimensions.width as isize)
        } else {
            x + (x.abs() / chunk_dimensions.width as isize)
        }
    };
    let y = {
        let y = translation.y.round() as isize;
        if y >= 0 {
            y - (y / chunk_dimensions.height as isize)
        } else {
            y + (y.abs() / chunk_dimensions.width as isize)
        }
    };
    let z = {
        let z = translation.z.round() as isize;
        if z >= 0 {
            z - (z / chunk_dimensions.depth as isize)
        } else {
            z + (z.abs() / chunk_dimensions.depth as isize)
        }
    };
    *chunk_translation = ChunkTranslation { x, y, z };
}

fn mark_chunks(
    mut commands: Commands,
    config: Res<GameConfig>,
    chunk_dimensions: Res<ChunkDimensions>,
    player_query: Query<&ChunkTranslation, With<Player>>,
    chunk_query: Query<(Entity, &ChunkTranslation, Option<&LoadedChunk>), With<Chunk>>,
) {
    let render_dist = config.chunk_config.render_distance;
    let chunk_translation = player_query.single();

    // these are coordinates of chunks(i.e. not real global coordinates)
    let start_x = chunk_translation.x - render_dist as isize / 2;
    let end_x = chunk_translation.x + render_dist as isize / 2;
    let start_y = chunk_translation.y - render_dist as isize / 2;
    let end_y = chunk_translation.y + render_dist as isize / 2;
    let start_z = chunk_translation.z - render_dist as isize / 2;
    let end_z = chunk_translation.z + render_dist as isize / 2;

    let mut chunks = vec![vec![vec![false; render_dist + 1]; render_dist + 1]; render_dist + 1];
    for (chunk, translation, loaded_tag) in chunk_query.iter() {
        if start_x <= translation.x
            && translation.x <= end_x
            && start_y <= translation.y
            && translation.y <= end_y
            && start_z <= translation.z
            && translation.z <= end_z
        {
            if loaded_tag.is_none() {
                if let Some(mut chunk) = commands.get_entity(chunk) {
                    chunk.insert(NotLoadedChunk);
                }
            }
            // these are "normalized" chunk coordinates
            let x = if start_x >= 0 {
                translation.x - start_x.abs()
            } else {
                translation.x + start_x.abs()
            };
            let y = if start_y >= 0 {
                translation.y - start_y.abs()
            } else {
                translation.y + start_y.abs()
            };
            let z = if start_z >= 0 {
                translation.z - start_z.abs()
            } else {
                translation.z + start_z.abs()
            };
            chunks[x as usize][y as usize][z as usize] = true;
        } else if let Some(chunk) = commands.get_entity(chunk) {
            chunk.despawn_recursive();
        }
    }

    // here we iterate on CHUNK coordinates(i.e. not global transform)
    for x in start_x..=end_x {
        for y in start_y..=end_y {
            for z in start_z..=end_z {
                // these are "normalized" chunk coordinates
                let xi = if start_x >= 0 {
                    x - start_x.abs()
                } else {
                    x + start_x.abs()
                };
                let yi = if start_y >= 0 {
                    y - start_y.abs()
                } else {
                    y + start_y.abs()
                };
                let zi = if start_z >= 0 {
                    z - start_z.abs()
                } else {
                    z + start_z.abs()
                };
                // checking if chunk was marked as non-existant
                if !chunks[xi as usize][yi as usize][zi as usize] {
                    // global coordinates would equal to chunk translation coord * dimension - dimension/2
                    let x_global = {
                        let x = x as f32;
                        let width = chunk_dimensions.width as f32;
                        (x - 0.5) * width
                    };
                    let y_global = {
                        let y = y as f32;
                        let height = chunk_dimensions.height as f32;
                        (y - 0.5) * height
                    };
                    let z_global = {
                        let z = z as f32;
                        let depth = chunk_dimensions.depth as f32;
                        (z - 0.5) * depth
                    };
                    commands
                        .spawn(ChunkBundle {
                            translation: ChunkTranslation { x, y, z },
                            chunk: Chunk,
                        })
                        // here we want to insert left bottom far edge
                        .insert(SpatialBundle::from_transform(Transform::from_xyz(
                            x_global, y_global, z_global,
                        )))
                        .insert(NotLoadedChunk);
                }
            }
        }
    }
}

fn load_chunks(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    chunk_dimensions: Res<ChunkDimensions>,
    // all chunks marked for loading
    to_load_query: Query<(Entity, &ChunkTranslation), With<NotLoadedChunk>>,
) {
    let material_h = materials.add(Color::WHITE);
    let cube = Cuboid {
        half_size: Vec3::splat(0.5),
    };
    let mesh_h = meshes.add(cube.mesh());

    for (chunk, translation) in to_load_query.iter() {
        if let Some(mut chunk) = commands.get_entity(chunk) {
            chunk.remove::<NotLoadedChunk>();
            if translation.y <= 0 {
                // println!(
                //     "Spawning chunk {}, {}, {}",
                //     translation.x, translation.y, translation.z
                // );
                chunk.with_children(|parent| {
                    for x in 0..chunk_dimensions.width {
                        for y in 0..chunk_dimensions.height {
                            for z in 0..chunk_dimensions.depth {
                                parent.spawn((
                                    PbrBundle {
                                        mesh: mesh_h.clone(),
                                        material: material_h.clone(),
                                        transform: Transform::from_xyz(
                                            x as f32, y as f32, z as f32,
                                        ),
                                        ..default()
                                    },
                                    RigidBody::Fixed,
                                    Collider::cuboid(0.5, 0.5, 0.5),
                                ));
                            }
                        }
                    }
                });
            }
            chunk.insert(LoadedChunk);
        }
    }
}

fn unload_chunks(
    mut commands: Commands,
    // all chunks that are not marked for loading or are already loaded(preserved)
    to_unload_query: Query<Entity, (With<Chunk>, Without<LoadedChunk>, Without<NotLoadedChunk>)>,
) {
    for chunk in to_unload_query.iter() {
        if let Some(chunk) = commands.get_entity(chunk) {
            chunk.despawn_recursive();
        }
    }
}
