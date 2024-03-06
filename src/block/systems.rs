use bevy::{asset::LoadedFolder, prelude::*, render::texture::ImageSampler};
use std::marker::PhantomData;

use super::{Block, BlocksTexturesFolder};
use crate::common::{create_texture_atlas, AppState, Atlas};

pub fn load_blocks_textures_folder(mut commands: Commands, asset_server: Res<AssetServer>) {
    let blocks_textures = asset_server.load_folder("textures/blocks");
    commands.insert_resource(BlocksTexturesFolder(blocks_textures));
}

pub fn check_textures(
    mut next_state: ResMut<NextState<AppState>>,
    block_textures_folder: Res<BlocksTexturesFolder>,
    mut events: EventReader<AssetEvent<LoadedFolder>>,
) {
    for event in events.read() {
        if event.is_loaded_with_dependencies(&block_textures_folder.0) {
            next_state.set(AppState::Game);
        }
    }
}

pub fn stitch_blocks_texture_atlas(
    mut commands: Commands,
    blocks_textures: Res<BlocksTexturesFolder>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    mut textures: ResMut<Assets<Image>>,
    mut texture_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let folder = loaded_folders.get(&blocks_textures.0).unwrap();
    let (atlas_layout, atlas_texture) =
        create_texture_atlas(&folder, None, Some(ImageSampler::nearest()), &mut textures);
    let atlas_layout = texture_layouts.add(atlas_layout);
    commands.insert_resource(Atlas {
        texture: atlas_texture,
        layout: atlas_layout,
        phantom: PhantomData::<Block>,
    });
}
