use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct KeyConfig {
    pub camera_controls: CameraControls,
    pub player_controls: PlayerControls,
}

pub struct CameraControls {
    pub switch_perspective: KeyCode,
}

impl Default for CameraControls {
    fn default() -> Self {
        CameraControls {
            switch_perspective: KeyCode::F5,
        }
    }
}

pub struct PlayerControls {
    pub move_forward: KeyCode,
    pub move_back: KeyCode,
    pub strafe_left: KeyCode,
    pub strafe_right: KeyCode,
    pub jump: KeyCode,
    pub crouch: KeyCode,
}

impl Default for PlayerControls {
    fn default() -> Self {
        PlayerControls {
            move_forward: KeyCode::KeyW,
            move_back: KeyCode::KeyS,
            strafe_left: KeyCode::KeyA,
            strafe_right: KeyCode::KeyD,
            jump: KeyCode::Space,
            crouch: KeyCode::ShiftLeft,
        }
    }
}

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(KeyConfig::default());
    }
}
