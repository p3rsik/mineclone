use bevy::prelude::*;

use crate::{
    block::BlockId,
    chunk::{
        debug::{show_chunk_border, toggle_show_chunks, ShowChunks},
        systems::*,
    },
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
            .add_systems(
                Update,
                (
                    load_chunks,
                    create_object,
                    toggle_show_chunks,
                    show_chunk_border,
                    mark_chunks,
                    reload_chunk_mesh,
                    unload_chunks,
                    destroy_object,
                ),
            );
    }
}

#[derive(Component, Debug)]
pub struct Chunk {
    pub block_data: Vec<BlockId>,
    pub translation: ChunkTranslation,
    pub dimensions: ChunkDimensions,
}

impl Chunk {
    pub fn remove_block_at(&mut self, translation: &Vec3) {
        let index = (translation.x as isize + (self.dimensions.width / 2) as isize)
            * (self.dimensions.width as isize)
            * (self.dimensions.height as isize)
            + (translation.y as isize + (self.dimensions.height / 2) as isize)
                * (self.dimensions.width as isize)
            + (translation.z as isize + (self.dimensions.depth / 2) as isize);
        self.block_data[index as usize] = BlockId(0);
        println!(
            "remove_block_at({} = {}) -> {:?}",
            translation, index, self.block_data[index as usize]
        );
    }
    pub fn set_block_at(&mut self, translation: &Vec3) {
        let index = (translation.x as isize + (self.dimensions.width / 2) as isize)
            * (self.dimensions.width as isize)
            * (self.dimensions.height as isize)
            + (translation.y as isize + (self.dimensions.height / 2) as isize)
                * (self.dimensions.width as isize)
            + (translation.z as isize + (self.dimensions.depth / 2) as isize);
        self.block_data[index as usize] = BlockId(1);
        println!(
            "set_block_at({} = {}) -> {:?}",
            translation, index, self.block_data[index as usize]
        );
    }
    pub fn get_block_at(&self, translation: &Vec3) -> BlockId {
        let index = (translation.x as isize + (self.dimensions.width / 2) as isize)
            * (self.dimensions.width as isize)
            * (self.dimensions.height as isize)
            + (translation.y as isize + (self.dimensions.height / 2) as isize)
                * (self.dimensions.width as isize)
            + (translation.z as isize + (self.dimensions.depth / 2) as isize);
        self.block_data[index as usize].clone()
    }
    pub fn get_local_block_pos(&self, translation: &Vec3) -> Vec3 {
        let width = self.dimensions.width as f32;
        let height = self.dimensions.height as f32;
        let depth = self.dimensions.depth as f32;
        let origin = Vec3::new(
            (self.translation.x as f32 + 0.5) * width,
            (self.translation.y as f32 + 0.5) * height,
            (self.translation.z as f32 + 0.5) * depth,
        );
        println!(
            "get_local_block_pos({}) -> {}",
            translation,
            *translation - origin
        );
        *translation - origin
    }
    pub fn get_chunk_translation(
        global_point: &Vec3,
        dimensions: &ChunkDimensions,
    ) -> ChunkTranslation {
        ChunkTranslation {
            x: global_point.x.div_euclid(dimensions.width as f32) as isize,
            y: global_point.y.div_euclid(dimensions.height as f32) as isize,
            z: global_point.z.div_euclid(dimensions.depth as f32) as isize,
        }
    }
}

pub struct ChunkLoadData {
    translation: ChunkTranslation,
    dimensions: ChunkDimensions,
    global_pos: Vec3,
}

#[derive(Event)]
pub enum ChunkEvent {
    Reload(Entity),
    Load(ChunkLoadData),
    Remove(Entity),
}

#[derive(Resource, Component, Clone, Debug)]
// Dimensions must be even
pub struct ChunkDimensions {
    pub width: usize,
    pub height: usize,
    pub depth: usize,
}

impl Default for ChunkDimensions {
    fn default() -> Self {
        ChunkDimensions {
            width: 2,
            height: 2,
            depth: 2,
        }
    }
}

#[derive(Component, Clone, PartialEq, Debug, Eq, Hash)]
// this is a coordinates of the chunk that do not relate to global transform
pub struct ChunkTranslation {
    pub x: isize,
    pub y: isize,
    pub z: isize,
}
