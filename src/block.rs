use bevy::prelude::*;

// pub struct BlockPlugin;

// impl Plugin for BlockPlugin {
//     fn build(&self, app: &mut App) {
//         todo!()
//     }
// }

#[derive(Component, Clone, Debug)]
pub struct BlockId(pub usize);

pub const BLOCK_HALF_SIZE: f32 = 0.5;
