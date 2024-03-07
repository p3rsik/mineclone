use bevy::{prelude::*, utils::HashMap};

use crate::{
    block::BlockId,
    chunk::{Chunk, ChunkDimensions, ChunkTranslation},
};

pub struct GameWorldPlugin;

impl Plugin for GameWorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameWorld>();
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
    // Generates chunk and applies player changes to it
    pub fn get_chunk_at(
        &mut self,
        translation: ChunkTranslation,
        // modified data to apply while generating
    ) -> Chunk {
        let dimensions = self.chunk_dimensions.clone();
        let mut unique_blocks = Vec::new();
        let mut block_data =
            vec![BlockId::air(); dimensions.width * dimensions.height * dimensions.depth];
        let maybe_chunk = self.chunk_data.get(&translation);
        for x in 0..dimensions.width {
            for y in 0..dimensions.height {
                for z in 0..dimensions.depth {
                    let index = x * dimensions.width * dimensions.height + y * dimensions.width + z;
                    // if there is a chunk in memory already
                    if let Some(chunk) = maybe_chunk {
                        // if there is a block in storage - use it
                        if let Some(block_id) = chunk.block_data.get(index) {
                            if *block_id != BlockId::air() && !unique_blocks.contains(block_id) {
                                unique_blocks.push(block_id.clone());
                            }
                            block_data[index] = block_id.clone();
                            continue;
                        }
                    } else {
                        // TODO replace with actual world generation
                        // if we're at chunks that are under 0 then generate stone
                        if translation.y < 0 {
                            if index == 7 {
                                block_data[index] = BlockId::air();
                                continue;
                            }
                            block_data[index] = BlockId::air();
                        } else {
                            // else generate air
                            block_data[index] = BlockId::air();
                        }
                    };
                }
            }
        }
        if maybe_chunk.is_none() {
            self.chunk_data.insert(
                translation.clone(),
                Chunk {
                    block_data: block_data.clone(),
                    translation: translation.clone(),
                    dimensions: dimensions.clone(),
                    unique_blocks: unique_blocks.clone(),
                },
            );
        }
        Chunk {
            block_data,
            translation,
            dimensions,
            unique_blocks,
        }
    }
    // Saves player changes to chunk
    pub fn save_chunk(&mut self, chunk: &Chunk) {
        let dimensions = chunk.dimensions.clone();
        if let Some(chunk_prev) = self.chunk_data.get_mut(&chunk.translation) {
            for x in 0..dimensions.width {
                for y in 0..dimensions.height {
                    for z in 0..dimensions.depth {
                        let index =
                            x * dimensions.width * dimensions.height + y * dimensions.width + z;
                        chunk_prev.block_data[index] = chunk.block_data[index].clone();
                    }
                }
            }
        } else {
            let mut block_data =
                vec![BlockId::air(); dimensions.width * dimensions.height * dimensions.depth];

            for x in 0..dimensions.width {
                for y in 0..dimensions.height {
                    for z in 0..dimensions.depth {
                        let index =
                            x * dimensions.width * dimensions.height + y * dimensions.width + z;
                        block_data[index] = chunk.block_data[index].clone();
                    }
                }
            }
            self.chunk_data.insert(
                chunk.translation.clone(),
                Chunk {
                    block_data,
                    translation: chunk.translation.clone(),
                    dimensions,
                    unique_blocks: chunk.unique_blocks.clone(),
                },
            );
        };
    }
}
