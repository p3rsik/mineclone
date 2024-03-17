use bevy::{asset::LoadedFolder, prelude::*};
use serde::Deserialize;

use crate::{
    common::{AppState, SetupState},
    registry::BlockRegistry,
};

use self::{asset::BlockAssetLoader, systems::*};

pub mod asset;
mod systems;

pub const BLOCK_HALF_SIZE: f32 = 0.5;

pub struct BlockPlugin;

impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlockRegistry>()
            .init_asset::<Block>()
            .init_asset_loader::<BlockAssetLoader>()
            .add_systems(
                OnEnter(AppState::Setup(SetupState::Textures)),
                load_blocks_textures_folder,
            )
            .add_systems(
                Update,
                check_textures.run_if(in_state(AppState::Setup(SetupState::Textures))),
            )
            .add_systems(
                OnExit(AppState::Setup(SetupState::Textures)),
                stitch_blocks_texture_atlas,
            )
            .add_systems(
                OnEnter(AppState::Setup(SetupState::Blocks)),
                load_blocks_folder,
            )
            .add_systems(
                Update,
                check_blocks.run_if(in_state(AppState::Setup(SetupState::Blocks))),
            )
            .add_systems(
                OnExit(AppState::Setup(SetupState::Blocks)),
                populate_block_registry,
            );
    }
}

#[derive(Asset, TypePath, Clone, Debug)]
pub struct Block {
    pub id: BlockId,
    pub name: String,
    pub textures: BlockTextures<AssetId<Image>>,
    pub opacity: Opacity,
}

#[derive(Clone, Debug)]
pub struct BlockMesh {
    pub opacity: Opacity,
    pub textures: BlockTextures<Rect>,
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
        front: T,
        back: T,
        left: T,
        right: T,
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

#[derive(Resource)]
pub struct BlockInfoFolder(pub Handle<LoadedFolder>);

impl From<String> for BlockId {
    fn from(value: String) -> Self {
        BlockId(value)
    }
}

impl From<&str> for BlockId {
    fn from(value: &str) -> Self {
        BlockId(String::from(value))
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

impl<T> BlockTextures<T> {
    pub fn map<F, U>(self, mut f: F) -> BlockTextures<U>
    where
        F: FnMut(T) -> U,
    {
        match self {
            BlockTextures::All {
                top,
                bottom,
                front,
                back,
                left,
                right,
            } => BlockTextures::All {
                top: f(top),
                bottom: f(bottom),
                front: f(front),
                back: f(back),
                left: f(left),
                right: f(right),
            },
            BlockTextures::TopBottomAndSide { top, bottom, side } => {
                BlockTextures::TopBottomAndSide {
                    top: f(top),
                    bottom: f(bottom),
                    side: f(side),
                }
            }
            BlockTextures::TopAndSide { top, side } => BlockTextures::TopAndSide {
                top: f(top),
                side: f(side),
            },
            BlockTextures::Single(v) => BlockTextures::Single(f(v)),
        }
    }

    pub fn top(&self) -> &T {
        match self {
            BlockTextures::All {
                top,
                bottom: _,
                front: _,
                back: _,
                left: _,
                right: _,
            } => top,
            BlockTextures::TopBottomAndSide {
                top,
                bottom: _,
                side: _,
            } => top,
            BlockTextures::TopAndSide { top, side: _ } => top,
            BlockTextures::Single(v) => v,
        }
    }

    pub fn bottom(&self) -> &T {
        match self {
            BlockTextures::All {
                top: _,
                bottom,
                front: _,
                back: _,
                left: _,
                right: _,
            } => bottom,
            BlockTextures::TopBottomAndSide {
                top: _,
                bottom,
                side: _,
            } => bottom,
            BlockTextures::TopAndSide { top, side: _ } => top,
            BlockTextures::Single(v) => v,
        }
    }

    pub fn front(&self) -> &T {
        match self {
            BlockTextures::All {
                top: _,
                bottom: _,
                front,
                back: _,
                left: _,
                right: _,
            } => front,
            BlockTextures::TopBottomAndSide {
                top: _,
                bottom: _,
                side,
            } => side,
            BlockTextures::TopAndSide { top: _, side } => side,
            BlockTextures::Single(v) => v,
        }
    }

    pub fn back(&self) -> &T {
        match self {
            BlockTextures::All {
                top: _,
                bottom: _,
                front: _,
                back,
                left: _,
                right: _,
            } => back,
            BlockTextures::TopBottomAndSide {
                top: _,
                bottom: _,
                side,
            } => side,
            BlockTextures::TopAndSide { top: _, side } => side,
            BlockTextures::Single(v) => v,
        }
    }

    pub fn left(&self) -> &T {
        match self {
            BlockTextures::All {
                top: _,
                bottom: _,
                front: _,
                back: _,
                left,
                right: _,
            } => left,
            BlockTextures::TopBottomAndSide {
                top: _,
                bottom: _,
                side,
            } => side,
            BlockTextures::TopAndSide { top: _, side } => side,
            BlockTextures::Single(v) => v,
        }
    }

    pub fn right(&self) -> &T {
        match self {
            BlockTextures::All {
                top: _,
                bottom: _,
                front: _,
                back: _,
                left: _,
                right,
            } => right,
            BlockTextures::TopBottomAndSide {
                top: _,
                bottom: _,
                side,
            } => side,
            BlockTextures::TopAndSide { top: _, side } => side,
            BlockTextures::Single(v) => v,
        }
    }
}
