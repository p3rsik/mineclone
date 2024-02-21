use bevy::{input::mouse::MouseMotion, prelude::*};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, camera_movement)
            .add_systems(Update, mouse_movement);
    }
}

#[derive(Component)]
pub struct FirstPersonCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub sensitivity: f32,
    pub velocity: Vec3,
    pub friction: f32,
}

impl Default for FirstPersonCamera {
    fn default() -> Self {
        FirstPersonCamera {
            yaw: 0.0,
            pitch: 0.0,
            sensitivity: 8.0,
            velocity: Vec3::ZERO,
            friction: 0.8,
        }
    }
}

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.0, -2.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        FirstPersonCamera::default(),
    ));
}

pub fn mouse_movement(
    mut query: Query<(&mut Transform, &mut FirstPersonCamera)>,
    mut ev_motion: EventReader<MouseMotion>,
    // ev_movement: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let (mut camera_transform, mut options) = query.single_mut();
    let mut delta = Vec2::ZERO;
    for ev in ev_motion.read() {
        delta += ev.delta;
    }

    if delta.is_nan() {
        return;
    }

    options.yaw -= delta.x * options.sensitivity * time.delta_seconds();
    options.pitch += delta.y * options.sensitivity * time.delta_seconds();
    options.pitch = options.pitch.clamp(-89.0, 89.9);

    let yaw_radians = options.yaw.to_radians();
    let pitch_radians = options.pitch.to_radians();

    camera_transform.rotation = Quat::from_axis_angle(Vec3::Y, yaw_radians)
        * Quat::from_axis_angle(-Vec3::X, pitch_radians);
}

pub fn camera_movement(
    time: Res<Time>,
    k_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut FirstPersonCamera, &mut Transform)>,
) {
    let (mut options, mut transform) = query.single_mut();
    let (axis_x, axis_y, axis_z) = (
        axis_movement(&k_input, KeyCode::KeyD, KeyCode::KeyA),
        axis_movement(&k_input, KeyCode::Space, KeyCode::ShiftLeft),
        axis_movement(&k_input, KeyCode::KeyS, KeyCode::KeyW),
    );

    let rotation = transform.rotation;
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

    let friction = if options.velocity.length() != 0.0 {
        options.velocity.normalize() * -1.0 * options.friction
    } else {
        Vec3::ZERO
    };

    options.velocity += accel * time.delta_seconds();
    let delta_friction = friction * time.delta_seconds();
    options.velocity = if (options.velocity + delta_friction).signum() != options.velocity.signum()
    {
        Vec3::ZERO
    } else {
        options.velocity + delta_friction
    };
    transform.translation += options.velocity;
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
