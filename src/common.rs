use std::marker::PhantomData;

use bevy::{asset::LoadedFolder, prelude::*, render::texture::ImageSampler};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, States)]
pub enum AppState {
    Setup(SetupState),
    Game,
}

impl Default for AppState {
    fn default() -> Self {
        AppState::Setup(SetupState::Textures)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum SetupState {
    #[default]
    Textures,
    Blocks,
}

// Generalized resource to hold different kind of atlasses
#[derive(Resource)]
pub struct Atlas<T> {
    pub texture: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub phantom: PhantomData<T>,
}

// copied straight from bevy examples
pub fn create_texture_atlas(
    folder: &LoadedFolder,
    padding: Option<UVec2>,
    sampling: Option<ImageSampler>,
    textures: &mut ResMut<Assets<Image>>,
) -> (TextureAtlasLayout, Handle<Image>) {
    // Build a texture atlas using the individual sprites
    let mut texture_atlas_builder =
        TextureAtlasBuilder::default().padding(padding.unwrap_or_default());
    for handle in folder.handles.iter() {
        let id = handle.id().typed_unchecked::<Image>();
        let Some(texture) = textures.get(id) else {
            warn!(
                "{:?} did not resolve to an `Image` asset.",
                handle.path().unwrap()
            );
            continue;
        };

        texture_atlas_builder.add_texture(Some(id), texture);
    }

    let (texture_atlas_layout, texture) = texture_atlas_builder.finish().unwrap();
    let texture = textures.add(texture);

    // Update the sampling settings of the texture atlas
    let image = textures.get_mut(&texture).unwrap();
    image.sampler = sampling.unwrap_or_default();

    (texture_atlas_layout, texture)
}
