use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{config::GameConfig, player::Player};

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Dimensions::default())
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
pub struct Dimensions {
    pub width: usize,
    pub height: usize,
    pub depth: usize,
}

impl Default for Dimensions {
    fn default() -> Self {
        Dimensions {
            width: 2,
            height: 2,
            depth: 2,
        }
    }
}

#[derive(Component, Clone, PartialEq)]
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
    chunk_dimensions: Res<Dimensions>,
) {
    let (player_transofrm, mut chunk_translation) = player_query.single_mut();
    let translation = player_transofrm.translation;
    let x = {
        let x = translation.x.round() as isize;
        if x.signum() == 1 || x.signum() == 0 {
            x - (x % chunk_dimensions.width as isize)
        } else {
            x + (x.abs() % chunk_dimensions.width as isize)
        }
    };
    let y = {
        let y = translation.y.round() as isize;
        if y.signum() == 1 || y.signum() == 0 {
            y - (y % chunk_dimensions.height as isize)
        } else {
            y + (y.abs() % chunk_dimensions.width as isize)
        }
    };
    let z = {
        let z = translation.z.round() as isize;
        if z.signum() == 1 || z.signum() == 0 {
            z - (z % chunk_dimensions.depth as isize)
        } else {
            z + (z.abs() % chunk_dimensions.depth as isize)
        }
    };
    // println!("Player chunk: {}, {}, {}", x, y, z);
    *chunk_translation = ChunkTranslation { x, y, z };
}

fn mark_chunks(
    mut commands: Commands,
    config: Res<GameConfig>,
    chunk_dimensions: Res<Dimensions>,
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
    // println!(
    //     "Checking chunks in range:\n[{}, {}, {}]\n[{}, {}, {}]",
    //     start_x, start_y, start_z, end_x, end_y, end_z
    // );

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
            // println!(
            //     "translation: {}, {}, {}, normalized: {}, {}, {}",
            //     translation.x, translation.y, translation.z, x, y, z
            // );
            chunks[x as usize][y as usize][z as usize] = true;
        } else if let Some(chunk) = commands.get_entity(chunk) {
            chunk.despawn_recursive();
        }
    }

    // println!("marked chunks: {:?}", chunks);

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
                // println!(
                //     "Chunk {}, {}, {} indices are {}, {}, {}",
                //     x, y, z, xi, yi, zi
                // );
                if !chunks[xi as usize][yi as usize][zi as usize] {
                    // println!("Marking chunk {}, {}, {} to spawn", x, y, z);
                    commands
                        .spawn(ChunkBundle {
                            translation: ChunkTranslation { x, y, z },
                            chunk: Chunk,
                        })
                        // here we want to insert left bottom far edge
                        .insert(TransformBundle::from_transform(Transform::from_xyz(
                            (x - (chunk_dimensions.width / 2) as isize) as f32,
                            (y - (chunk_dimensions.height / 2) as isize) as f32,
                            (z - (chunk_dimensions.depth / 2) as isize) as f32,
                        )))
                        .insert(VisibilityBundle::default())
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
    chunk_dimensions: Res<Dimensions>,
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
                                parent
                                    .spawn(PbrBundle {
                                        mesh: mesh_h.clone(),
                                        material: material_h.clone(),
                                        transform: Transform::from_xyz(
                                            x as f32, y as f32, z as f32,
                                        ),
                                        ..default()
                                    })
                                    .insert(RigidBody::Fixed)
                                    .insert(TransformBundle::from_transform(Transform::from_xyz(
                                        x as f32, y as f32, z as f32,
                                    )))
                                    .insert(Collider::cuboid(0.5, 0.5, 0.5));
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
