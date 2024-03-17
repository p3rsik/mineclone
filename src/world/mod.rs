use bevy::{prelude::*, utils::HashMap};

use crate::{
    block::BlockId,
    chunk::{Chunk, ChunkDimensions, ChunkTranslation},
    common::AppState,
};

use self::systems::*;

mod systems;

pub struct GameWorldPlugin;

impl Plugin for GameWorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameWorld>()
            .add_systems(OnEnter(AppState::Game), setup_global_light)
            .add_systems(Update, (day_night_cycle).run_if(in_state(AppState::Game)));
    }
}

#[derive(Resource)]
pub struct GameWorld {
    pub chunk_data: HashMap<ChunkTranslation, Chunk>,
    pub chunk_dimensions: ChunkDimensions,
}

impl Default for GameWorld {
    fn default() -> Self {
        GameWorld {
            chunk_data: HashMap::new(),
            chunk_dimensions: ChunkDimensions {
                width: 16,
                height: 16,
                depth: 16,
            },
        }
    }
}

impl GameWorld {
    pub fn get_block_at(&mut self, pos: Vec3) -> Option<BlockId> {
        let chunk_translation = ChunkTranslation::get_chunk_translation(pos, self.chunk_dimensions);
        let chunk_dimensions = self.chunk_dimensions;
        let chunk = self.get_chunk_at(chunk_translation);
        chunk.get_block_at(pos, chunk_dimensions)
    }

    // Sets block at position to the new one returning what was there previously
    pub fn set_block_at(&mut self, block_id: Option<BlockId>, pos: Vec3) -> Option<BlockId> {
        let chunk_translation = ChunkTranslation::get_chunk_translation(pos, self.chunk_dimensions);
        let chunk_dimensions = self.chunk_dimensions;
        let chunk = self.get_chunk_at_mut(chunk_translation);
        chunk.set_block_at(pos, block_id, chunk_dimensions)
    }

    // Generates chunk and applies player changes to it
    pub fn get_chunk_at_mut(
        &mut self,
        chunk_translation: ChunkTranslation,
        // modified data to apply while generating
    ) -> &mut Chunk {
        let dimensions = self.chunk_dimensions;
        let chunk = self.chunk_data.entry(chunk_translation).or_insert({
            let mut unique_blocks = Vec::new();
            let mut block_data =
                vec![None; dimensions.width * dimensions.height * dimensions.depth];

            for x in 0..dimensions.width {
                for y in 0..dimensions.height {
                    for z in 0..dimensions.depth {
                        let index =
                            x * dimensions.width * dimensions.height + y * dimensions.width + z;
                        // if there is a chunk in memory already
                        // TODO replace with actual world generation
                        // if we're at chunks that are under 0 then generate stone
                        if chunk_translation.y < 0 {
                            block_data[index] = Some(BlockId::from("mineclone:stone"));
                            if !unique_blocks.contains(&BlockId::from("mineclone:stone")) {
                                unique_blocks.push(BlockId::from("mineclone:stone"));
                            }
                        } else {
                            // else generate air
                            block_data[index] = None;
                        }
                    }
                }
            }

            Chunk {
                block_data,
                unique_blocks,
                translation: chunk_translation,
            }
        });
        chunk
    }

    pub fn get_chunk_at(&mut self, chunk_translation: ChunkTranslation) -> &Chunk {
        self.get_chunk_at_mut(chunk_translation)
    }
}
