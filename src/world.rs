use bevy::{prelude::*, utils::HashMap};

use crate::{
    block::BlockId,
    chunk::{Chunk, ChunkDimensions, ChunkTranslation},
};

pub struct GameWorldPlugin;

impl Plugin for GameWorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameWorld::default());
    }
}

#[derive(Resource)]
pub struct GameWorld {
    pub chunk_data: HashMap<ChunkTranslation, Chunk>,
}

impl Default for GameWorld {
    fn default() -> Self {
        GameWorld {
            chunk_data: HashMap::new(),
        }
    }
}

impl GameWorld {
    // Generates chunk and applies player changes to it
    pub fn get_chunk(
        &self,
        translation: ChunkTranslation,
        dimensions: ChunkDimensions,
        // modified data to apply while generating
    ) -> Chunk {
        let mut block_data =
            vec![BlockId(0); dimensions.width * dimensions.height * dimensions.depth];
        for x in 0..dimensions.width {
            for y in 0..dimensions.height {
                for z in 0..dimensions.depth {
                    let index = x * dimensions.width * dimensions.height + y * dimensions.width + z;
                    // if there is a chunk in memory already
                    if let Some(chunk) = self.chunk_data.get(&translation) {
                        // if there is a block in storage - use it
                        if let Some(block_id) = chunk.block_data.get(index) {
                            block_data[index] = block_id.clone();
                            continue;
                        }
                    } else {
                        // TODO replace with actual world generation
                        // if we're at chunks that are under 0 then generate stone
                        if translation.y < 0 {
                            if index == 7 {
                                block_data[index] = BlockId(0);
                                continue;
                            }
                            block_data[index] = BlockId(1);
                        } else {
                            // else generate air
                            block_data[index] = BlockId(0);
                        }
                    };
                }
            }
        }
        Chunk {
            block_data,
            translation,
            dimensions,
        }
    }
    // Saves player changes to chunk
    pub fn save_chunk(
        &mut self,
        translation: ChunkTranslation,
        dimensions: ChunkDimensions,
        chunk: &Chunk,
    ) {
        if let Some(chunk_prev) = self.chunk_data.get_mut(&translation) {
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
                vec![BlockId(0); dimensions.width * dimensions.height * dimensions.depth];

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
                translation.clone(),
                Chunk {
                    block_data,
                    translation,
                    dimensions,
                },
            );
        };
    }
}
