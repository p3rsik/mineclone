use bevy::{
    asset::{AssetLoader, AsyncReadExt},
    prelude::*,
    utils::BoxedFuture,
};
use serde_json::from_slice;
use thiserror::Error;

use super::{Block, BlockJson, BlockTextures, BlockTexturesPaths};

#[derive(Default)]
struct BlockLoader;

#[derive(Debug, Error)]
pub enum BlockLoaderError {
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not parse json: {0}")]
    JsonError(#[from] serde_json::Error),
}

impl AssetLoader for BlockLoader {
    type Asset = Block;

    type Settings = ();

    type Error = BlockLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            let block_json: BlockJson = from_slice(&bytes)?;
            // at this point texture atlas must already be stitched
            // and all textures must be loaded, so this must return handles
            // to existing assets instead of reloading them
            let block_textures = match block_json.textures {
                BlockTexturesPaths::All {
                    top,
                    bottom,
                    side_front,
                    side_back,
                    side_left,
                    side_right,
                } => BlockTextures::All {
                    top: load_context.load(top).id(),
                    bottom: load_context.load(bottom).id(),
                    side_front: load_context.load(side_front).id(),
                    side_back: load_context.load(side_back).id(),
                    side_left: load_context.load(side_left).id(),
                    side_right: load_context.load(side_right).id(),
                },
                BlockTexturesPaths::TopBottomAndSide { top, bottom, side } => {
                    BlockTextures::TopBottomAndSide {
                        top: load_context.load(top).id(),
                        bottom: load_context.load(bottom).id(),
                        side: load_context.load(side).id(),
                    }
                }
                BlockTexturesPaths::TopAndSide { top, side } => BlockTextures::TopAndSide {
                    top: load_context.load(top).id(),
                    side: load_context.load(side).id(),
                },
                BlockTexturesPaths::Single(texture) => {
                    BlockTextures::Single(load_context.load(texture).id())
                }
            };

            Ok(Block {
                id: block_json.id.into(),
                name: block_json.name,
                textures: block_textures,
                opacity: block_json.opacity.into(),
            })
        })
    }
}
