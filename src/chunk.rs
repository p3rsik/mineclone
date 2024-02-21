use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_chunk);
    }
}

pub fn spawn_chunk(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let material = materials.add(Color::WHITE);
    let cube_size = 0.4;
    let cube = Cuboid {
        half_size: Vec3::new(cube_size, cube_size, cube_size),
    };
    let mesh_h = meshes.add(cube.mesh());

    for chunk_x in -1..1 {
        for chunk_z in -1..1 {
            for x in 0..16 {
                for y in 0..16 {
                    for z in 0..16 {
                        commands
                            .spawn(PbrBundle {
                                mesh: mesh_h.clone(),
                                material: material.clone(),
                                transform: Transform::from_xyz(
                                    (x + chunk_x * 16) as f32,
                                    (-y) as f32,
                                    (z + chunk_z * 16) as f32,
                                ),
                                ..default()
                            })
                            .insert(RigidBody::Fixed)
                            .insert(TransformBundle::from_transform(Transform::from_xyz(
                                (x + chunk_x * 16) as f32,
                                (-y) as f32,
                                (z + chunk_z * 16) as f32,
                            )))
                            .insert(Collider::cuboid(cube_size, cube_size, cube_size));
                    }
                }
            }
        }
    }
}
