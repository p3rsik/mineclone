use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera);
        // .add_systems(Update, camera_movement);
    }
}

#[derive(Component)]
pub struct FirstPersonCamera {
    pub sensitivity: f32,
    pub velocity: Vec3,
    pub friction: f32,
}

impl Default for FirstPersonCamera {
    fn default() -> Self {
        FirstPersonCamera {
            sensitivity: 8.0,
            velocity: Vec3::ZERO,
            friction: 0.8,
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 0.75, -2.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        FirstPersonCamera::default(),
    ));
}
