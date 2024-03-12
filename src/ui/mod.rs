use bevy::prelude::*;

use self::systems::*;
use crate::common::{AppState, SetupState};

mod systems;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InInventory>()
            .add_systems(
                OnEnter(AppState::Setup(SetupState::Ui)),
                load_ui_texture_atlas,
            )
            .add_systems(
                Update,
                check_textures.run_if(in_state(AppState::Setup(SetupState::Ui))),
            )
            .add_systems(OnExit(AppState::Setup(SetupState::Ui)), add_ui_atlas)
            .add_systems(Update, cursor_grab_toggle.run_if(in_state(AppState::Game)))
            .add_systems(
                Update,
                recenter_cursor.run_if(
                    in_state(AppState::Game)
                        // .and_then(resource_changed::<InInventory>)
                        // .and_then(not(resource_added::<InInventory>))
                        .and_then(resource_equals(InInventory::Out)),
                ),
            )
            .add_systems(OnEnter(AppState::Game), (show_crosshair, cursor_grab));
    }
}

// Marker for UI atlas
pub struct Ui;

#[derive(Resource)]
pub struct UiTextures(pub Handle<Image>);

#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub enum InInventory {
    #[default]
    Out, // we're in the game
    In, // we're in the inventory
}
