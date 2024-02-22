use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::camera::FirstPersonCamera;

pub struct BlockPlugin;

impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, block_selection)
            .add_systems(Update, draw_box_aroud_object)
            .add_systems(Last, (destroy_object, create_object));
    }
}

#[derive(Component)]
pub struct LookingAt {
    // Entity which the player is looking at
    pub entity: Entity,
    // normal of the plane at which player is looking
    pub normal: Vec3,
}

fn block_selection(
    mut commands: Commands,
    camera_query: Query<&Transform, With<FirstPersonCamera>>,
    looked_at_query: Query<Entity, With<LookingAt>>,
    rapier_context: Res<RapierContext>,
) {
    let camera_transform = camera_query.single();
    let ray_pos = camera_transform.translation;
    let ray_dir = camera_transform.forward().xyz();
    let max_toi = 4.0;
    let solid = true;
    let filter = QueryFilter::new();

    for looked_at in looked_at_query.iter() {
        commands.entity(looked_at).remove::<LookingAt>();
    }
    if let Some((entity, intersection)) =
        rapier_context.cast_ray_and_get_normal(ray_pos, ray_dir, max_toi, solid, filter)
    {
        if let Some(mut entity_commands) = commands.get_entity(entity) {
            debug!("Camera is looking at {:?}", entity);
            entity_commands.insert(LookingAt {
                entity,
                normal: intersection.normal,
            });
        }
    };
}

fn draw_box_aroud_object(mut gizmos: Gizmos, looked_at_query: Query<&Transform, With<LookingAt>>) {
    let cube = Cuboid {
        half_size: Vec3::new(0.5 + 0.001, 0.5 + 0.001, 0.5 + 0.001),
    };

    for looked_at in looked_at_query.iter() {
        debug!("Drawing gizmo around {}", looked_at.translation);
        gizmos.primitive_3d(
            cube,
            looked_at.translation,
            looked_at.rotation,
            Color::WHITE,
        );
    }
}

fn destroy_object(
    mut commands: Commands,
    looking_at_query: Query<&LookingAt>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        for looking_at in looking_at_query.iter() {
            debug!("Destroying cube {:?}", looking_at.entity);
            commands.entity(looking_at.entity).despawn();
        }
    }
}

fn create_object(
    mut commands: Commands,
    looking_at_query: Query<(&Transform, &LookingAt)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    let material = materials.add(Color::WHITE);
    let cube_size = 0.5;
    let cube = Cuboid {
        half_size: Vec3::new(cube_size, cube_size, cube_size),
    };
    let mesh_h = meshes.add(cube.mesh());

    if buttons.just_pressed(MouseButton::Right) {
        for (transform, looking_at) in looking_at_query.iter() {
            debug!(
                "Placing cube at {}",
                transform.translation + looking_at.normal
            );
            commands
                .spawn(PbrBundle {
                    mesh: mesh_h.clone(),
                    material: material.clone(),
                    transform: Transform::from_translation(
                        transform.translation + looking_at.normal,
                    ),
                    ..default()
                })
                .insert(RigidBody::Fixed)
                .insert(TransformBundle::from_transform(
                    Transform::from_translation(transform.translation + looking_at.normal),
                ))
                .insert(Collider::cuboid(cube_size, cube_size, cube_size));
        }
    }
}
