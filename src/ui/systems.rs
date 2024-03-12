use std::marker::PhantomData;

use bevy::{
    prelude::*,
    render::texture::ImageSampler,
    window::{CursorGrabMode, PrimaryWindow},
};

use crate::{
    common::{AppState, Atlas, SetupState},
    config::GameConfig,
};

use super::{InInventory, Ui, UiTextures};

pub fn load_ui_texture_atlas(mut commands: Commands, asset_server: Res<AssetServer>) {
    let ui_textures = asset_server.load("textures/ui_atlas.png");
    commands.insert_resource(UiTextures(ui_textures));
}

pub fn check_textures(
    mut next_state: ResMut<NextState<AppState>>,
    ui_textures: Res<UiTextures>,
    mut events: EventReader<AssetEvent<Image>>,
) {
    for event in events.read() {
        if event.is_loaded_with_dependencies(&ui_textures.0) {
            next_state.set(AppState::Setup(SetupState::Textures));
        }
    }
}

pub fn add_ui_atlas(
    mut commands: Commands,
    mut texture_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut textures: ResMut<Assets<Image>>,
    ui_textures: Res<UiTextures>,
) {
    let atlas_layout = TextureAtlasLayout::from_grid(Vec2::splat(16.0), 128, 128, None, None);

    let image = textures.get_mut(&ui_textures.0).unwrap();
    image.sampler = ImageSampler::nearest();

    commands.insert_resource(Atlas {
        texture: ui_textures.0.clone(),
        layout: texture_layouts.add(atlas_layout),
        material: None,
        phantom: PhantomData::<Ui>,
    });
    commands.remove_resource::<UiTextures>();
}

pub fn show_crosshair(mut commands: Commands, ui_atlas: Res<Atlas<Ui>>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                ImageBundle {
                    style: Style {
                        width: Val::Px(24.0),
                        height: Val::Px(24.0),
                        ..default()
                    },
                    image: UiImage::new(ui_atlas.texture.clone()),
                    ..default()
                },
                TextureAtlas {
                    layout: ui_atlas.layout.clone(),
                    index: 0,
                },
            ));
        });
}

pub fn cursor_grab_toggle(
    mut q_window: Query<&mut Window, With<PrimaryWindow>>,
    mut in_inventory: ResMut<InInventory>,
    k_input: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
) {
    if k_input.just_pressed(config.key_config.game_controls.inventory_key) {
        let mut primary_window = q_window.single_mut();
        match *in_inventory {
            InInventory::Out => {
                primary_window.cursor.grab_mode = CursorGrabMode::None;
                primary_window.cursor.visible = true;
                *in_inventory = InInventory::In;
            }
            InInventory::In => {
                primary_window.cursor.grab_mode = CursorGrabMode::Locked;
                primary_window.cursor.visible = false;
                *in_inventory = InInventory::Out;
            }
        };
    };
}

pub fn cursor_grab(mut q_window: Query<&mut Window, With<PrimaryWindow>>) {
    let mut primary_window = q_window.single_mut();
    primary_window.cursor.grab_mode = CursorGrabMode::Locked;
    primary_window.cursor.visible = false;
}

pub fn recenter_cursor(mut q_window: Query<&mut Window, With<PrimaryWindow>>) {
    let mut primary_window = q_window.single_mut();
    let center = Vec2::new(primary_window.width() / 2.0, primary_window.height() / 2.0);
    primary_window.set_cursor_position(Some(center));
}
