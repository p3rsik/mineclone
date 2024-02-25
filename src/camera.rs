use bevy::prelude::*;

use crate::{config::KeyConfig, player::Player};

pub struct CameraPlugin;

#[derive(Resource, Default)]
pub enum CameraPerspective {
    #[default]
    FirstPerson,
    ThirdPerson,
    ThirdPersonInverted,
}

impl CameraPerspective {
    fn next(&mut self) {
        match self {
            Self::FirstPerson => *self = Self::ThirdPerson,
            Self::ThirdPerson => *self = Self::ThirdPersonInverted,
            Self::ThirdPersonInverted => *self = Self::FirstPerson,
        }
    }
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CameraPerspective::default())
            .add_systems(Startup, spawn_camera)
            .add_systems(Update, change_perspective)
            .add_systems(Update, change_camera_origin);
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
            transform: Transform::from_xyz(0.0, 2.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        FirstPersonCamera::default(),
    ));
}

fn change_perspective(
    k_input: Res<ButtonInput<KeyCode>>,
    k_config: Res<KeyConfig>,
    mut camera_perspective: ResMut<CameraPerspective>,
) {
    if k_input.just_pressed(k_config.camera_controls.switch_perspective) {
        camera_perspective.next();
    }
}

fn change_camera_origin(
    camera_perspective: Res<CameraPerspective>,
    mut camera_query: Query<&mut Transform, (With<FirstPersonCamera>, Without<Player>)>,
    player_query: Query<&Transform, (With<Player>, Without<FirstPersonCamera>)>,
) {
    if camera_perspective.is_changed() {
        let mut camera_transform = camera_query.single_mut();
        let player_transform = player_query.single();
        // We cycle through camera perspectives one-by-one, so we can be sure, that when we arrive at
        // one of them, it would be from the previous perspective in order.
        // TODO implement minecraft style third-person camera
        match *camera_perspective {
            CameraPerspective::FirstPerson => {
                let rotation = player_transform.rotation;
                camera_transform.rotation = rotation;
                camera_transform.translation = player_transform.translation + Vec3::Y * 2.0;
            }
            CameraPerspective::ThirdPerson => {
                let rotation = player_transform.rotation;
                let forward_vec = rotation.mul_vec3(Vec3::Z).normalize();
                let final_vec = Vec3::new(forward_vec.x * 5.0, 3.0, forward_vec.z * 5.0);
                camera_transform.translation += final_vec;
            }
            // TODO figure this out
            CameraPerspective::ThirdPersonInverted => {
                let rotation = player_transform.rotation;
                let backward_vec = rotation.mul_vec3(Vec3::Z).normalize();
                let final_vec = Vec3::new(backward_vec.x * 5.0, 3.0, backward_vec.z * 5.0);
                camera_transform.rotate_y(180.0f32.to_radians());
                camera_transform.translation += final_vec;
            }
        };
    }
}
