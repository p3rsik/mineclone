use bevy::prelude::*;

use crate::{
    block::BlockId,
    chunk::{
        debug::{show_chunk_border, toggle_show_chunks, ShowChunks},
        systems::*,
    },
    common::AppState,
};

pub mod debug;
pub mod mesh;
mod systems;

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChunkDimensions::default())
            .insert_resource(ShowChunks::DontShow)
            .add_event::<ChunkEvent>()
            .configure_sets(
                Update,
                (ChunkSystems::PlayerInput, ChunkSystems::ChunkReload)
                    .chain()
                    .run_if(in_state(AppState::Game)),
            )
            .add_systems(
                Update,
                (
                    (show_chunk_border.run_if(resource_equals(ShowChunks::Show))),
                    (
                        mark_chunks,
                        // create_object,
                        destroy_object,
                        toggle_show_chunks,
                    )
                        .in_set(ChunkSystems::PlayerInput),
                    (load_chunks, unload_chunks, reload_chunk)
                        .in_set(ChunkSystems::ChunkReload)
                        .run_if(on_event::<ChunkEvent>()),
                ),
            );
    }
}

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub enum ChunkSystems {
    PlayerInput,
    ChunkReload,
}

#[derive(Component, Debug, Clone)]
pub struct ChunkMarker {
    pub translation: ChunkTranslation,
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub block_data: Vec<Option<BlockId>>,
    pub translation: ChunkTranslation,
    pub unique_blocks: Vec<BlockId>,
}

impl Chunk {
    pub fn set_block_at(
        &mut self,
        pos: Vec3,
        block_id: Option<BlockId>,
        dimensions: ChunkDimensions,
    ) -> Option<BlockId> {
        let pos = Chunk::get_local_block_pos(pos, self.translation, dimensions);
        let index = (pos.x as isize + (dimensions.width / 2) as isize)
            * (dimensions.width as isize)
            * (dimensions.height as isize)
            + (pos.y as isize + (dimensions.height / 2) as isize) * (dimensions.width as isize)
            + (pos.z as isize + (dimensions.depth / 2) as isize);
        let res = self.block_data[index as usize].clone();
        self.block_data[index as usize] = block_id;
        res
    }
    pub fn get_block_at(&self, pos: Vec3, dimensions: ChunkDimensions) -> Option<BlockId> {
        let pos = Chunk::get_local_block_pos(pos, self.translation, dimensions);
        let index = (pos.x as isize + (dimensions.width / 2) as isize)
            * (dimensions.width as isize)
            * (dimensions.height as isize)
            + (pos.y as isize + (dimensions.height / 2) as isize) * (dimensions.width as isize)
            + (pos.z as isize + (dimensions.depth / 2) as isize);
        self.block_data[index as usize].clone()
    }
    pub fn get_local_block_pos(
        pos: Vec3,
        translation: ChunkTranslation,
        dimensions: ChunkDimensions,
    ) -> Vec3 {
        let width = dimensions.width as f32;
        let height = dimensions.height as f32;
        let depth = dimensions.depth as f32;
        let origin = Vec3::new(
            (translation.x as f32 + 0.5) * width,
            (translation.y as f32 + 0.5) * height,
            (translation.z as f32 + 0.5) * depth,
        );
        pos - origin
    }
}

pub struct ChunkLoadData {
    translation: ChunkTranslation,
    global_pos: Vec3,
}

#[derive(Event)]
pub enum ChunkEvent {
    Reload(Entity),
    Load(ChunkLoadData),
    Remove(Entity),
}

#[derive(Resource, Component, Clone, Copy, Debug)]
// Dimensions must be even
pub struct ChunkDimensions {
    pub width: usize,
    pub height: usize,
    pub depth: usize,
}

impl Default for ChunkDimensions {
    fn default() -> Self {
        ChunkDimensions {
            width: 16,
            height: 16,
            depth: 16,
        }
    }
}

// this is a coordinates of the chunk that have no relation to global transform
#[derive(Component, Clone, Copy, PartialEq, Debug, Eq, Hash)]
pub struct ChunkTranslation {
    pub x: isize,
    pub y: isize,
    pub z: isize,
}

impl ChunkTranslation {
    pub fn get_chunk_translation(
        global_point: Vec3,
        dimensions: ChunkDimensions,
    ) -> ChunkTranslation {
        ChunkTranslation {
            x: global_point.x.div_euclid(dimensions.width as f32) as isize,
            y: global_point.y.div_euclid(dimensions.height as f32) as isize,
            z: global_point.z.div_euclid(dimensions.depth as f32) as isize,
        }
    }
}
