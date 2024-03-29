use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    block::Block,
    common::Atlas,
    config::GameConfig,
    player::{LookingAt, Player},
    registry::BlockRegistry,
    world::GameWorld,
};

use super::{
    mesh::ChunkMesh, ChunkDimensions, ChunkEvent, ChunkLoadData, ChunkMarker, ChunkTranslation,
};

pub fn mark_chunks(
    config: Res<GameConfig>,
    chunk_dimensions: Res<ChunkDimensions>,
    player_query: Query<&Transform, (With<Player>, Changed<Transform>)>,
    mut chunk_ev: EventWriter<ChunkEvent>,
    chunk_query: Query<(Entity, &ChunkMarker)>,
) {
    let render_dist = config.chunk_config.render_distance;
    let chunk_translation = ChunkTranslation::get_chunk_translation(
        player_query.single().translation,
        *chunk_dimensions,
    );

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
        } else {
            chunk_ev.send(ChunkEvent::Remove(chunk_entity));
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
                    chunk_ev.send(ChunkEvent::Load(ChunkLoadData {
                        translation: ChunkTranslation { x, y, z },
                        global_pos: Vec3::new(x_global, y_global, z_global),
                    }));
                }
            }
        }
    }
}

pub fn load_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut game_world: ResMut<GameWorld>,
    // all chunks marked for loading
    mut chunk_ev: EventReader<ChunkEvent>,
    blocks: Res<Assets<Block>>,
    block_registry: Res<BlockRegistry>,
    block_atlas: Res<Atlas<Block>>,
    layouts: Res<Assets<TextureAtlasLayout>>,
) {
    for chunk_event in chunk_ev.read() {
        let chunk_data = match chunk_event {
            ChunkEvent::Load(chunk_data) => chunk_data,
            _ => continue,
        };

        let chunk_dimensions = game_world.chunk_dimensions;
        let chunk = game_world.get_chunk_at(chunk_data.translation);
        let atlas_layout = layouts.get(&block_atlas.layout).unwrap();
        let chunk_mesh = ChunkMesh::new(
            chunk,
            chunk_dimensions,
            atlas_layout,
            &block_registry,
            &blocks,
        );

        let mesh = chunk_mesh.mesh();
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

        // we want to spawn a chunk even if it doesn't have a mesh, cauze it can be chunk without any blocks
        let chunk_entity = commands
            .spawn(ChunkMarker {
                translation: chunk_data.translation,
            })
            .insert(SpatialBundle::from_transform(Transform::from_translation(
                chunk_data.global_pos,
            )))
            .id();
        let mut chunk_commands = commands.entity(chunk_entity);

        if let Some(collider) = collider {
            chunk_commands.with_children(|parent| {
                parent.spawn((
                    PbrBundle {
                        mesh,
                        // it is safe to unwrap here, since Block atlas will always contain material
                        material: block_atlas.material.clone().unwrap(),
                        transform: Transform::from_translation(Vec3::splat(0.0)),
                        ..default()
                    },
                    RigidBody::Fixed,
                    collider,
                ));
            });
        }
    }
}

pub fn unload_chunks(
    mut commands: Commands,
    mut chunk_ev: EventReader<ChunkEvent>,
    // to_unload_chunk_query: Query<&ChunkMarker>,
) {
    for chunk_event in chunk_ev.read() {
        let chunk_entity = match chunk_event {
            ChunkEvent::Remove(chunk_entity) => chunk_entity,
            _ => continue,
        };
        // if let Ok(_chunk) = to_unload_chunk_query.get(*chunk_entity) {
        commands.entity(*chunk_entity).despawn_recursive();
        // }
    }
}

pub fn reload_chunk(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunk_ev: EventReader<ChunkEvent>,
    mut game_world: ResMut<GameWorld>,
    chunks_to_reload_query: Query<&ChunkMarker>,
    blocks: Res<Assets<Block>>,
    block_registry: Res<BlockRegistry>,
    block_atlas: Res<Atlas<Block>>,
    layouts: Res<Assets<TextureAtlasLayout>>,
) {
    for chunk_event in chunk_ev.read() {
        let chunk_entity = match chunk_event {
            ChunkEvent::Reload(chunk_entity) => chunk_entity,
            _ => continue,
        };
        if let Ok(chunk_marker) = chunks_to_reload_query.get(*chunk_entity) {
            let chunk_dimensions = game_world.chunk_dimensions;
            let chunk = game_world.get_chunk_at(chunk_marker.translation);
            let atlas_layout = layouts.get(&block_atlas.layout).unwrap();
            let chunk_mesh = ChunkMesh::new(
                chunk,
                chunk_dimensions,
                atlas_layout,
                &block_registry,
                &blocks,
            );

            let mesh = chunk_mesh.mesh();
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
                if let Some(mut chunk_commands) = commands.get_entity(*chunk_entity) {
                    chunk_commands.despawn_descendants();
                    chunk_commands.with_children(|parent| {
                        parent.spawn((
                            PbrBundle {
                                mesh,
                                // It is safe to unwrap here, since block_atlas will always contain a material
                                material: block_atlas.material.clone().unwrap(),
                                transform: Transform::from_translation(Vec3::splat(0.0)),
                                ..default()
                            },
                            RigidBody::Fixed,
                            collider,
                        ));
                    });
                }
            }
        }
    }
}

pub fn destroy_object(
    buttons: Res<ButtonInput<MouseButton>>,
    looking_at: Res<LookingAt>,
    chunks_query: Query<(Entity, &ChunkMarker)>,
    mut game_world: ResMut<GameWorld>,
    mut chunk_ev: EventWriter<ChunkEvent>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        if let LookingAt::Something {
            block_pos,
            normal: _normal,
        } = *looking_at
        {
            let chunk_dimensions = game_world.chunk_dimensions;
            game_world.set_block_at(None, block_pos);
            let mut chunk_entity = None;
            // TODO find a better way to do this
            for (entity, ChunkMarker { translation }) in chunks_query.iter() {
                if *translation
                    == ChunkTranslation::get_chunk_translation(block_pos, chunk_dimensions)
                {
                    chunk_entity = Some(entity);
                }
            }
            if let Some(e) = chunk_entity {
                chunk_ev.send(ChunkEvent::Reload(e));
            }
        }
    }
}

// pub fn create_object(
//     mut commands: Commands,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     mut meshes: ResMut<Assets<Mesh>>,
//     chunk_dimension: Res<ChunkDimensions>,
//     buttons: Res<ButtonInput<MouseButton>>,
//     // blocks are now children of Chunk, so Transform is local
//     looking_at_query: Query<(&GlobalTransform, &LookingAt)>,
//     mut chunk_query: Query<(Entity, &mut Chunk, &Transform)>,
// ) {
//     let material_h = materials.add(Color::RED);
//     let cube = Cuboid {
//         half_size: Vec3::splat(0.5),
//     };
//     let mesh_h = meshes.add(cube.mesh());
//     if buttons.just_pressed(MouseButton::Right) {
//         for (transform, looking_at) in looking_at_query.iter() {
//             // where to place new block in global coords
//             let translation = transform.translation() + looking_at.intersection.normal;
//             let to_chunk_translation = Chunk::get_chunk_translation(&translation, &chunk_dimension);
//             let (chunk_entity, mut chunk, chunk_transform) = chunk_query
//                 .iter_mut()
//                 .filter(|(_e, chunk, _tr)| chunk.translation == to_chunk_translation)
//                 .last()
//                 .unwrap();

//             let x = { translation.x - chunk_transform.translation.x };
//             let y = { translation.y - chunk_transform.translation.y };
//             let z = { translation.z - chunk_transform.translation.z };

//             chunk.set_block_at(&Vec3::new(x, y, z), BlockId::air());
//             commands.entity(chunk_entity).with_children(|parent| {
//                 parent.spawn((
//                     PbrBundle {
//                         mesh: mesh_h.clone(),
//                         material: material_h.clone(),
//                         transform: Transform::from_xyz(x, y, z),
//                         ..default()
//                     },
//                     RigidBody::Fixed,
//                     Collider::cuboid(0.5, 0.5, 0.5),
//                 ));
//             });
//         }
//     }
// }
