use bevy::prelude::*;

use crate::{config::GameConfig, player::Player};

pub struct CameraPlugin;

#[derive(Resource, Default, PartialEq)]
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

fn change_perspective(
    k_input: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
    mut camera_perspective: ResMut<CameraPerspective>,
) {
    if k_input.just_pressed(config.key_config.camera_controls.switch_perspective) {
        camera_perspective.next();
    }
}

fn change_camera_origin(
    camera_perspective: Res<CameraPerspective>,
    mut camera_query: Query<&mut Transform, (With<FirstPersonCamera>, Without<Player>)>,
) {
    if camera_perspective.is_changed() {
        let mut camera_transform = camera_query.single_mut();
        // We cycle through camera perspectives one-by-one, so we can be sure, that when we arrive at
        // one of them, it would be from the previous perspective in order.
        // TODO implement minecraft style third-person camera
        match *camera_perspective {
            CameraPerspective::FirstPerson => {
                camera_transform.translation = Vec3::new(0.0, 2.0, 0.0);
            }
            CameraPerspective::ThirdPerson => {
                camera_transform.translation = Vec3::new(0.0, 3.0, 5.0);
            }
            // TODO figure this out
            CameraPerspective::ThirdPersonInverted => {
                camera_transform.translation = Vec3::new(0.0, 3.0, -5.0)
            }
        };
    }
}
