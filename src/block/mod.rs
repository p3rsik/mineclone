use bevy::{asset::LoadedFolder, prelude::*};
use serde::Deserialize;

use crate::{common::AppState, registry::BlockRegistry};

use self::systems::{check_textures, load_blocks_textures_folder, stitch_blocks_texture_atlas};

pub mod asset;
mod systems;

pub const BLOCK_HALF_SIZE: f32 = 0.5;
pub const BLOCK_ID_AIR: BlockId = BlockId(String::from("mineclone:air"));

pub struct BlockPlugin;

impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlockRegistry>()
            .add_systems(OnEnter(AppState::Setup), load_blocks_textures_folder)
            .add_systems(Update, check_textures.run_if(in_state(AppState::Setup)))
            .add_systems(OnExit(AppState::Setup), stitch_blocks_texture_atlas);
    }
}

#[derive(Asset, TypePath, Clone, Debug)]
pub struct Block {
    pub id: BlockId,
    pub name: String,
    pub textures: BlockTextures<AssetId<Image>>,
    pub opacity: Opacity,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BlockId(pub String);

// Textures of a block
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
// serde(untagged) will try every constructor in order
// so I positioned them from most fully defined to the least
pub enum BlockTextures<T> {
    // different texture for every side
    All {
        top: T,
        bottom: T,
        side_front: T,
        side_back: T,
        side_left: T,
        side_right: T,
    },
    // top, bottom and one for all sieds
    TopBottomAndSide {
        top: T,
        bottom: T,
        side: T,
    },
    // one texture for top & bottom and one for all sides
    TopAndSide {
        top: T,
        side: T,
    },
    // one texture for all sides
    Single(T),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Opacity {
    // Not transparent
    Opaque,
    // level of transparency(the bigger the more transparent)
    Transparent(u8),
}

// this one is used when we first parse the block info
pub type BlockTexturesPaths = BlockTextures<String>;

#[derive(Debug, Deserialize)]
pub struct BlockJson {
    pub id: String,
    pub name: String,
    pub textures: BlockTexturesPaths,
    pub opacity: u8,
}

// this holds handle to the loaded textures
// we want to drop this after you load blocks
// !BUT! not before, since then blocks would reload them
// to get AssetId<Image> needed for indexing the atlas
#[derive(Resource)]
pub struct BlocksTexturesFolder(pub Handle<LoadedFolder>);

impl From<String> for BlockId {
    fn from(value: String) -> Self {
        BlockId(value)
    }
}

impl From<u8> for Opacity {
    fn from(value: u8) -> Self {
        match value {
            0 => Opacity::Opaque,
            v => Opacity::Transparent(v),
        }
    }
}
