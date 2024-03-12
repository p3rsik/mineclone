use bevy::prelude::*;

#[derive(Default)]
pub struct KeyConfig {
    pub camera_controls: CameraControls,
    pub player_controls: PlayerControls,
    pub debug_controls: DebugControls,
    pub game_controls: GameControls,
}

pub struct GameControls {
    pub inventory_key: KeyCode,
}

impl Default for GameControls {
    fn default() -> Self {
        GameControls {
            inventory_key: KeyCode::KeyE,
        }
    }
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

pub struct DebugControls {
    pub toggle_chunk_border: KeyCode,
}

impl Default for DebugControls {
    fn default() -> Self {
        DebugControls {
            toggle_chunk_border: KeyCode::F9,
        }
    }
}

pub struct ChunkConfig {
    // render_distance must be even
    pub render_distance: usize,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        ChunkConfig { render_distance: 4 }
    }
}

#[derive(Resource, Default)]
pub struct GameConfig {
    pub key_config: KeyConfig,
    pub chunk_config: ChunkConfig,
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
        app.insert_resource(GameConfig::default());
    }
}
