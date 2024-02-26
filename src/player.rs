use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_rapier3d::{control::KinematicCharacterController, prelude::*};

use crate::{
    camera::{CameraPerspective, FirstPersonCamera},
    chunk::ChunkTranslation,
    config::GameConfig,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, move_player)
            .add_systems(Update, rotate_player_and_camera);
    }
}

#[derive(Component)]
pub struct Player {
    pub camera_yaw: f32,
    pub camera_pitch: f32,
    pub velocity: Vec3,
    pub friction: f32,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            camera_yaw: 0.0,
            camera_pitch: 0.0,
            velocity: Vec3::ZERO,
            friction: 0.8,
        }
    }
}

fn spawn_player(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let player = Cuboid {
        half_size: Vec3::new(0.3, 0.75, 0.3),
    };
    let material = materials.add(Color::RED);
    let mesh = meshes.add(player.mesh());

    let player_ent = commands
        .spawn(Player::default())
        .insert(ChunkTranslation { x: 0, y: 0, z: 0 })
        .insert(PbrBundle {
            mesh,
            material,
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            ..default()
        })
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::cuboid(0.3, 0.75, 0.3))
        .insert(KinematicCharacterController::default())
        .id();

    let camera_ent = commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(0.0, 2.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            },
            FirstPersonCamera::default(),
        ))
        .id();

    commands.entity(player_ent).add_child(camera_ent);
}

fn rotate_player_and_camera(
    mut camera_query: Query<(&mut Transform, &FirstPersonCamera), Without<Player>>,
    mut player_query: Query<(&mut Transform, &mut Player), Without<FirstPersonCamera>>,
    mut mouse_input: EventReader<MouseMotion>,
    time: Res<Time>,
    camera_perspective: Res<CameraPerspective>,
) {
    let (mut camera_transform, camera_options) = camera_query.single_mut();
    let (mut player_transform, mut player_options) = player_query.single_mut();
    let mut delta = Vec2::ZERO;

    for ev in mouse_input.read() {
        delta += ev.delta;
    }
    if delta.is_nan() {
        return;
    }

    player_options.camera_yaw -= delta.x * camera_options.sensitivity * time.delta_seconds();
    player_options.camera_pitch += delta.y * camera_options.sensitivity * time.delta_seconds();
    player_options.camera_pitch = player_options.camera_pitch.clamp(-89.9, 89.9);

    let yaw_radians = player_options.camera_yaw.to_radians();
    let pitch_radians = player_options.camera_pitch.to_radians();

    camera_transform.rotation = Quat::from_axis_angle(-Vec3::X, pitch_radians);

    // Third person camera needs additional orbit-like translation
    if *camera_perspective == CameraPerspective::ThirdPerson {
        camera_transform.translation = camera_transform.back().xyz() * 6.0;
    }

    player_transform.rotation = Quat::from_axis_angle(Vec3::Y, yaw_radians);
}

fn move_player(
    time: Res<Time>,
    k_input: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
    mut player_query: Query<(&mut KinematicCharacterController, &mut Player, &Transform)>,
) {
    let k_config = &config.key_config;
    let (mut player_controller, mut player_options, player_tf) = player_query.single_mut();
    let (axis_x, axis_y, axis_z) = (
        axis_movement(
            &k_input,
            k_config.player_controls.strafe_right,
            k_config.player_controls.strafe_left,
        ),
        axis_movement(
            &k_input,
            k_config.player_controls.jump,
            k_config.player_controls.crouch,
        ),
        axis_movement(
            &k_input,
            k_config.player_controls.move_back,
            k_config.player_controls.move_forward,
        ),
    );

    let rotation = player_tf.rotation;
    let forward_vector = rotation.mul_vec3(Vec3::Z).normalize();
    let forward_walk_vector = Vec3::new(forward_vector.x, 0.0, forward_vector.z).normalize();
    let strafe_vector = Quat::from_rotation_y(90.0f32.to_radians())
        .mul_vec3(forward_walk_vector)
        .normalize();

    let accel = strafe_vector * axis_x + forward_walk_vector * axis_z + Vec3::Y * axis_y;
    let accel = if accel.length() != 0.0 {
        accel.normalize()
    } else {
        Vec3::ZERO
    };

    let friction = if player_options.velocity.length() != 0.0 {
        player_options.velocity.normalize() * -1.0 * player_options.friction
    } else {
        Vec3::ZERO
    };

    player_options.velocity += accel * time.delta_seconds();
    let delta_friction = friction * time.delta_seconds();
    player_options.velocity = if (player_options.velocity + delta_friction).signum()
        != player_options.velocity.signum()
    {
        Vec3::ZERO
    } else {
        player_options.velocity + delta_friction
    };
    player_controller.translation = Some(player_options.velocity);
}

fn axis_movement(input: &Res<ButtonInput<KeyCode>>, plus: KeyCode, minus: KeyCode) -> f32 {
    let mut axis = 0.0;
    if input.pressed(plus) {
        axis += 1.0;
    }
    if input.pressed(minus) {
        axis -= 1.0;
    }
    axis
}
