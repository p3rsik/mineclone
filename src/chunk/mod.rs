use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    block::{BlockId, BLOCK_HALF_SIZE},
    camera::LookingAt,
    chunk::debug::{show_chunk_border, toggle_show_chunks, ShowChunks},
    config::GameConfig,
    player::Player,
    world::GameWorld,
};

pub mod debug;
pub mod mesh;

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChunkDimensions::default())
            .insert_resource(ShowChunks::DontShow)
            .add_systems(
                Update,
                (
                    load_chunks,
                    create_object,
                    toggle_show_chunks,
                    show_chunk_border,
                ),
            )
            .add_systems(First, mark_chunks)
            .add_systems(Last, (unload_chunks, destroy_object));
    }
}

#[derive(Component, Debug)]
pub struct Chunk {
    pub block_data: Vec<BlockId>,
    pub translation: ChunkTranslation,
    pub dimensions: ChunkDimensions,
}

impl Chunk {
    // translation must be relative to chunk and x, y or z must not be < 0
    pub fn delete_block_at(&mut self, translation: &Vec3) {
        let index = (translation.x as usize) * self.dimensions.width * self.dimensions.height
            + (translation.y as usize) * self.dimensions.width
            + (translation.z as usize);
        self.block_data[index] = BlockId(0);
    }
    // translation must be relative to chunk and x, y or z must not be < 0
    pub fn create_block_at(&mut self, translation: &Vec3) {
        let index = (translation.x as usize) * self.dimensions.width * self.dimensions.height
            + (translation.y as usize) * self.dimensions.width
            + (translation.z as usize);
        self.block_data[index] = BlockId(1);
    }
    // translation must be relative to chunk and x, y or z must not be < 0
    pub fn get_block_at(&self, translation: &Vec3) -> BlockId {
        let index = (translation.x as usize) * self.dimensions.width * self.dimensions.height
            + (translation.y as usize) * self.dimensions.width
            + (translation.z as usize);
        self.block_data[index].clone()
    }
    pub fn get_chunk_translation(
        global_point: &Vec3,
        dimensions: &ChunkDimensions,
    ) -> ChunkTranslation {
        ChunkTranslation {
            x: global_point.x.div_euclid(dimensions.width as f32) as isize,
            y: global_point.y.div_euclid(dimensions.height as f32) as isize,
            z: global_point.z.div_euclid(dimensions.depth as f32) as isize,
        }
    }
}

#[derive(Component)]
// Marker for chunks that needs to be loaded
pub struct LoadedChunk;

#[derive(Component)]
pub struct NotLoadedChunk;

#[derive(Resource, Component, Clone, Debug)]
// Dimensions must be even
pub struct ChunkDimensions {
    pub width: usize,
    pub height: usize,
    pub depth: usize,
}

impl Default for ChunkDimensions {
    fn default() -> Self {
        ChunkDimensions {
            width: 16,
            height: 16,
            depth: 16,
        }
    }
}

#[derive(Component, Clone, PartialEq, Debug, Eq, Hash)]
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

fn mark_chunks(
    mut commands: Commands,
    config: Res<GameConfig>,
    chunk_dimensions: Res<ChunkDimensions>,
    player_query: Query<&Transform, (With<Player>, Changed<Transform>)>,
    chunk_query: Query<(Entity, &Chunk)>,
) {
    let render_dist = config.chunk_config.render_distance;
    let chunk_translation =
        Chunk::get_chunk_translation(&player_query.single().translation, &chunk_dimensions);

    // these are coordinates of chunks(i.e. not real global coordinates)
    let start_x = chunk_translation.x - render_dist as isize;
    let end_x = chunk_translation.x + render_dist as isize;
    let start_y = chunk_translation.y - render_dist as isize;
    let end_y = chunk_translation.y + render_dist as isize;
    let start_z = chunk_translation.z - render_dist as isize;
    let end_z = chunk_translation.z + render_dist as isize;

    let mut chunks =
        vec![vec![vec![false; 2 * render_dist + 1]; 2 * render_dist + 1]; 2 * render_dist + 1];
    for (chunk_entity, chunk) in chunk_query.iter() {
        if start_x <= chunk.translation.x
            && chunk.translation.x <= end_x
            && start_y <= chunk.translation.y
            && chunk.translation.y <= end_y
            && start_z <= chunk.translation.z
            && chunk.translation.z <= end_z
        {
            // these are "normalized" chunk coordinates
            let x = if start_x >= 0 {
                chunk.translation.x - start_x.abs()
            } else {
                chunk.translation.x + start_x.abs()
            };
            let y = if start_y >= 0 {
                chunk.translation.y - start_y.abs()
            } else {
                chunk.translation.y + start_y.abs()
            };
            let z = if start_z >= 0 {
                chunk.translation.z - start_z.abs()
            } else {
                chunk.translation.z + start_z.abs()
            };
            chunks[x as usize][y as usize][z as usize] = true;
        } else if let Some(mut chunk) = commands.get_entity(chunk_entity) {
            chunk.remove::<(LoadedChunk, NotLoadedChunk)>();
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
                    // global coordinates would equal to chunk translation coord * dimension
                    let x_global = {
                        let x = x as f32;
                        let width = chunk_dimensions.width as f32;
                        x * width + BLOCK_HALF_SIZE
                    };
                    let y_global = {
                        let y = y as f32;
                        let height = chunk_dimensions.height as f32;
                        y * height - BLOCK_HALF_SIZE
                    };
                    let z_global = {
                        let z = z as f32;
                        let depth = chunk_dimensions.depth as f32;
                        z * depth + BLOCK_HALF_SIZE
                    };
                    commands
                        .spawn((ChunkTranslation { x, y, z }, chunk_dimensions.clone()))
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
    game_world: Res<GameWorld>,
    // all chunks marked for loading
    to_load_query: Query<(Entity, &ChunkTranslation, &ChunkDimensions), With<NotLoadedChunk>>,
) {
    let material_h = materials.add(Color::WHITE);

    for (chunk, translation, dimensions) in to_load_query.iter() {
        if let Some(mut chunk_entity) = commands.get_entity(chunk) {
            chunk_entity.remove::<NotLoadedChunk>();
            let chunk = game_world.get_chunk(translation.clone(), dimensions.clone());
            let mesh = meshes.add(chunk.mesh());

            chunk_entity.with_children(|parent| {
                parent.spawn((PbrBundle {
                    mesh,
                    material: material_h.clone(),
                    transform: Transform::from_translation(Vec3::splat(-BLOCK_HALF_SIZE)),
                    ..default()
                },));
            });
            chunk_entity
                .remove::<(ChunkTranslation, ChunkDimensions)>()
                .insert(LoadedChunk)
                .insert(chunk);
        }
    }
}

fn unload_chunks(
    mut commands: Commands,
    // all chunks that are not marked for loading or are already loaded(preserved)
    mut game_world: ResMut<GameWorld>,
    to_unload_query: Query<(Entity, &Chunk), (Without<LoadedChunk>, Without<NotLoadedChunk>)>,
) {
    for (chunk_entity, chunk) in to_unload_query.iter() {
        if let Some(chunk_commands) = commands.get_entity(chunk_entity) {
            game_world.save_chunk(chunk.translation.clone(), chunk.dimensions.clone(), chunk);
            chunk_commands.despawn_recursive();
        }
    }
}

fn destroy_object(
    mut commands: Commands,
    looking_at_query: Query<(&Parent, &LookingAt, &Transform)>,
    mut chunk_query: Query<&mut Chunk>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        for (chunk_entity, looking_at, transform) in looking_at_query.iter() {
            // TODO Check if this solution is okay
            // block always has one parent and it's its chunk
            let mut chunk = chunk_query.get_mut(chunk_entity.get()).unwrap();
            chunk.delete_block_at(&transform.translation);
            commands.entity(looking_at.entity).despawn_recursive();
        }
    }
}

fn create_object(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    chunk_dimension: Res<ChunkDimensions>,
    buttons: Res<ButtonInput<MouseButton>>,
    // blocks are now children of Chunk, so Transform is local
    looking_at_query: Query<(&GlobalTransform, &LookingAt)>,
    mut chunk_query: Query<(Entity, &mut Chunk, &Transform)>,
) {
    let material_h = materials.add(Color::RED);
    let cube = Cuboid {
        half_size: Vec3::splat(0.5),
    };
    let mesh_h = meshes.add(cube.mesh());
    if buttons.just_pressed(MouseButton::Right) {
        for (transform, looking_at) in looking_at_query.iter() {
            // where to place new block in global coords
            let translation = transform.translation() + looking_at.normal;
            let to_chunk_translation = Chunk::get_chunk_translation(&translation, &chunk_dimension);
            let (chunk_entity, mut chunk, chunk_transform) = chunk_query
                .iter_mut()
                .filter(|(_e, chunk, _tr)| chunk.translation == to_chunk_translation)
                .last()
                .unwrap();

            let x = { translation.x - chunk_transform.translation.x };
            let y = { translation.y - chunk_transform.translation.y };
            let z = { translation.z - chunk_transform.translation.z };

            chunk.create_block_at(&Vec3::new(x, y, z));
            commands.entity(chunk_entity).with_children(|parent| {
                parent.spawn((
                    PbrBundle {
                        mesh: mesh_h.clone(),
                        material: material_h.clone(),
                        transform: Transform::from_xyz(x, y, z),
                        ..default()
                    },
                    RigidBody::Fixed,
                    Collider::cuboid(0.5, 0.5, 0.5),
                ));
            });
        }
    }
}
