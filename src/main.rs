use bevy::{
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_rapier3d::prelude::*;
use camera::{CameraPlugin, FirstPersonCamera};
use chunk::ChunkPlugin;

mod camera;
mod chunk;

#[derive(Component)]
struct LookingAt {
    // Entity which the player is looking at
    entity: Entity,
    // normal of the plane at which player is looking
    normal: Vec3,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.1, 0.0, 0.15)))
        .insert_resource(AmbientLight {
            brightness: 0.75,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(CameraPlugin)
        .add_plugins(ChunkPlugin)
        // .add_systems(Startup, spawn_objects)
        .add_systems(Startup, setup_lights)
        .add_systems(Update, cursor_grab)
        .add_systems(Update, object_selection)
        .add_systems(Update, draw_box_aroud_object)
        .add_systems(Last, (destroy_object, create_object))
        .run();
}

fn setup_lights(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube = Cuboid {
        half_size: Vec3::new(0.4, 0.4, 0.4),
    };
    let mesh = meshes.add(cube.mesh());
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 500000.0,
            range: 10000.0,
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, 10.0, 0.0))
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Z),
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh,
        material: materials.add(Color::WHITE),
        transform: Transform::from_xyz(0.0, 10.0, 0.0),
        ..default()
    });
}

fn cursor_grab(mut q_window: Query<&mut Window, With<PrimaryWindow>>) {
    let mut primary_window = q_window.single_mut();
    primary_window.cursor.grab_mode = CursorGrabMode::Locked;
    primary_window.cursor.visible = false;
}

fn object_selection(
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
            entity_commands.insert(LookingAt {
                entity,
                normal: intersection.normal,
            });
        }
    };
}

fn draw_box_aroud_object(mut gizmos: Gizmos, looked_at_query: Query<&Transform, With<LookingAt>>) {
    let cube = Cuboid {
        half_size: Vec3::new(0.42, 0.42, 0.42),
    };

    for looked_at in looked_at_query.iter() {
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
    let cube_size = 0.4;
    let cube = Cuboid {
        half_size: Vec3::new(cube_size, cube_size, cube_size),
    };
    let mesh_h = meshes.add(cube.mesh());

    if buttons.just_pressed(MouseButton::Right) {
        for (transform, looking_at) in looking_at_query.iter() {
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
