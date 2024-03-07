use bevy::{
    asset::{AssetLoader, AsyncReadExt},
    prelude::*,
    utils::BoxedFuture,
};
use serde_json::from_slice;
use thiserror::Error;

use super::{Block, BlockJson};

#[derive(Default)]
pub struct BlockAssetLoader;

#[derive(Debug, Error)]
pub enum BlockLoaderError {
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not parse json: {0}")]
    JsonError(#[from] serde_json::Error),
}

impl AssetLoader for BlockAssetLoader {
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
            let block_textures = block_json.textures.map(|v| load_context.load(v).id());

            Ok(Block {
                id: block_json.id.into(),
                name: block_json.name,
                textures: block_textures,
                opacity: block_json.opacity.into(),
            })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["block"]
    }
}
