use bevy::{asset::LoadedFolder, prelude::*, render::texture::ImageSampler};
use std::marker::PhantomData;

use super::{Block, BlockInfoFolder, BlocksTexturesFolder};
use crate::{
    common::{create_texture_atlas, AppState, Atlas, SetupState},
    registry::BlockRegistry,
};

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
            next_state.set(AppState::Setup(SetupState::Blocks));
        }
    }
}

pub fn stitch_blocks_texture_atlas(
    mut commands: Commands,
    blocks_textures: Res<BlocksTexturesFolder>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    mut textures: ResMut<Assets<Image>>,
    mut texture_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let folder = loaded_folders.get(&blocks_textures.0).unwrap();
    let (atlas_layout, atlas_texture) =
        create_texture_atlas(folder, None, Some(ImageSampler::nearest()), &mut textures);
    let atlas_layout = texture_layouts.add(atlas_layout);
    let material_h = materials.add(StandardMaterial {
        base_color_texture: Some(atlas_texture.clone()),
        ..default()
    });
    commands.insert_resource(Atlas {
        texture: atlas_texture,
        layout: atlas_layout,
        material: Some(material_h),
        phantom: PhantomData::<Block>,
    });
}

pub fn load_blocks_folder(mut commands: Commands, asset_server: Res<AssetServer>) {
    let blocks_folder = asset_server.load_folder("blocks");
    commands.insert_resource(BlockInfoFolder(blocks_folder));
}

pub fn check_blocks(
    mut next_state: ResMut<NextState<AppState>>,
    block_info_folder: Res<BlockInfoFolder>,
    mut events: EventReader<AssetEvent<LoadedFolder>>,
) {
    for event in events.read() {
        if event.is_loaded_with_dependencies(&block_info_folder.0) {
            next_state.set(AppState::Game);
        }
    }
}

pub fn populate_block_registry(
    mut commands: Commands,
    blocks_info: Res<BlockInfoFolder>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    blocks: ResMut<Assets<Block>>,
    mut registry: ResMut<BlockRegistry>,
) {
    let folder = loaded_folders.get(&blocks_info.0).unwrap();
    for handle in folder.handles.iter() {
        let handle = handle.clone().typed_unchecked::<Block>();
        let id = handle.id();
        let Some(block) = blocks.get(id) else {
            warn!(
                "{:?} did not resolve to a `Block` asset.",
                handle.path().unwrap()
            );
            continue;
        };
        registry.register(block.id.clone(), handle);
    }
    // Dropping the handle to the blocks texture folder
    // since we have already stitched the atlas
    commands.remove_resource::<BlocksTexturesFolder>();
}
