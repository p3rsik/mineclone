use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    block::BlockId,
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
            .add_systems(First, (mark_chunks, reload_chunk_mesh))
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
    pub fn remove_block_at(&mut self, translation: &Vec3) {
        let index = (translation.x as isize + (self.dimensions.width / 2) as isize)
            * (self.dimensions.width as isize)
            * (self.dimensions.height as isize)
            + (translation.y as isize + (self.dimensions.height / 2) as isize)
                * (self.dimensions.width as isize)
            + (translation.z as isize + (self.dimensions.depth / 2) as isize);
        self.block_data[index as usize] = BlockId(0);
        println!(
            "remove_block_at({} = {}) -> {:?}",
            translation, index, self.block_data[index as usize]
        );
    }
    pub fn set_block_at(&mut self, translation: &Vec3) {
        let index = (translation.x as isize + (self.dimensions.width / 2) as isize)
            * (self.dimensions.width as isize)
            * (self.dimensions.height as isize)
            + (translation.y as isize + (self.dimensions.height / 2) as isize)
                * (self.dimensions.width as isize)
            + (translation.z as isize + (self.dimensions.depth / 2) as isize);
        self.block_data[index as usize] = BlockId(1);
    }
    pub fn get_block_at(&self, translation: &Vec3) -> BlockId {
        let index = (translation.x as isize + (self.dimensions.width / 2) as isize)
            * (self.dimensions.width as isize)
            * (self.dimensions.height as isize)
            + (translation.y as isize + (self.dimensions.height / 2) as isize)
                * (self.dimensions.width as isize)
            + (translation.z as isize + (self.dimensions.depth / 2) as isize);
        self.block_data[index as usize].clone()
    }
    pub fn get_local_block_pos(&self, translation: &Vec3) -> Vec3 {
        let width = self.dimensions.width as f32;
        let height = self.dimensions.height as f32;
        let depth = self.dimensions.depth as f32;
        let origin = Vec3::new(
            (self.translation.x as f32 + 0.5) * width,
            (self.translation.y as f32 + 0.5) * height,
            (self.translation.z as f32 + 0.5) * depth,
        );
        println!(
            "get_local_block_pos({}) -> {}",
            translation,
            *translation - origin
        );
        *translation - origin
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

#[derive(Component)]
pub struct ReloadChunk;

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
            width: 2,
            height: 2,
            depth: 2,
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
                        (x + 0.5) * width
                    };
                    let y_global = {
                        let y = y as f32;
                        let height = chunk_dimensions.height as f32;
                        (y + 0.5) * height
                    };
                    let z_global = {
                        let z = z as f32;
                        let depth = chunk_dimensions.depth as f32;
                        (z + 0.5) * depth
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
            let mesh = chunk.mesh();
            // apparently Collider::from_bevy_mesh panics, because of
            // assert!(indices.len() > 0), so I need to check it manually
            // Why can't you rust return Err?
            let collider = mesh.indices().and_then(|inds| {
                if !inds.is_empty() {
                    Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::TriMesh)
                } else {
                    None
                }
            });
            let mesh = meshes.add(mesh);

            if let Some(collider) = collider {
                chunk_entity.with_children(|parent| {
                    parent.spawn((
                        PbrBundle {
                            mesh,
                            material: material_h.clone(),
                            transform: Transform::from_translation(Vec3::splat(0.0)),
                            ..default()
                        },
                        RigidBody::Fixed,
                        collider,
                    ));
                });
            }
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

fn reload_chunk_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    chunks_to_reload_query: Query<(Entity, &Chunk), With<ReloadChunk>>,
) {
    let material_h = materials.add(Color::WHITE);
    for (chunk_entity, chunk) in chunks_to_reload_query.iter() {
        let mesh = chunk.mesh();
        // apparently Collider::from_bevy_mesh panics, because of
        // assert!(indices.len() > 0), so I need to check it manually
        // Why can't you rust return Err?
        let collider = mesh.indices().and_then(|inds| {
            if !inds.is_empty() {
                Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::TriMesh)
            } else {
                None
            }
        });
        let mesh = meshes.add(mesh);

        if let Some(collider) = collider {
            if let Some(mut chunk_commands) = commands.get_entity(chunk_entity) {
                chunk_commands.despawn_descendants();
                chunk_commands.with_children(|parent| {
                    parent.spawn((
                        PbrBundle {
                            mesh,
                            material: material_h.clone(),
                            transform: Transform::from_translation(Vec3::splat(0.0)),
                            ..default()
                        },
                        RigidBody::Fixed,
                        collider,
                    ));
                });
                chunk_commands.remove::<ReloadChunk>();
            }
        }
    }
}

fn destroy_object(
    mut commands: Commands,
    looking_at_query: Query<(&Parent, &LookingAt)>,
    mut chunk_query: Query<(&mut Chunk, &GlobalTransform)>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        for (chunk_entity, looking_at) in looking_at_query.iter() {
            // TODO Check if this solution is okay
            // block always has one parent and it's its chunk
            let (mut chunk, transform) = chunk_query.get_mut(chunk_entity.get()).unwrap();
            let (_scale, _rot, tr) = transform.to_scale_rotation_translation();
            println!(
                "removing block from {:?} with global transform {}",
                chunk.translation, tr
            );
            println!("{:?}", looking_at);
            let local_pos =
                chunk.get_local_block_pos(&(looking_at.block_pos - looking_at.intersection.normal));
            chunk.remove_block_at(&local_pos);
            commands.entity(chunk_entity.get()).insert(ReloadChunk);
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
            let translation = transform.translation() + looking_at.intersection.normal;
            let to_chunk_translation = Chunk::get_chunk_translation(&translation, &chunk_dimension);
            let (chunk_entity, mut chunk, chunk_transform) = chunk_query
                .iter_mut()
                .filter(|(_e, chunk, _tr)| chunk.translation == to_chunk_translation)
                .last()
                .unwrap();

            let x = { translation.x - chunk_transform.translation.x };
            let y = { translation.y - chunk_transform.translation.y };
            let z = { translation.z - chunk_transform.translation.z };

            chunk.set_block_at(&Vec3::new(x, y, z));
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
