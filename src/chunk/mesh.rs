use bevy::{
    prelude::*,
    render::{mesh::Indices, render_asset::RenderAssetUsages},
    utils::HashMap,
};

use super::{Chunk, ChunkDimensions};
use crate::{
    block::{Block, BlockMesh, Opacity, BLOCK_HALF_SIZE},
    registry::BlockRegistry,
};

#[derive(Clone, Debug)]
pub struct ChunkMesh {
    block_data: Vec<Option<BlockMesh>>,
    atlas_size: Vec2,
    dimensions: ChunkDimensions,
}

impl ChunkMesh {
    pub fn new(
        chunk: &Chunk,
        atlas: &TextureAtlasLayout,
        registry: &Res<BlockRegistry>,
        blocks: &Res<Assets<Block>>,
    ) -> ChunkMesh {
        let mut block_meshes = HashMap::with_capacity(chunk.unique_blocks.len());
        for block_id in chunk.unique_blocks.iter() {
            let handle = registry.get(block_id).unwrap();
            let block = blocks.get(handle).unwrap();
            block_meshes.insert(
                block_id.clone(),
                BlockMesh {
                    opacity: block.opacity.clone(),
                    textures: block
                        .textures
                        .clone()
                        .map(|v| atlas.textures[atlas.get_texture_index(v).unwrap()]),
                },
            );
        }
        ChunkMesh {
            dimensions: chunk.dimensions.clone(),
            atlas_size: atlas.size,
            block_data: chunk
                .block_data
                .iter()
                .map(|some_id| {
                    some_id
                        .as_ref()
                        .and_then(|block_id| block_meshes.get(block_id).cloned())
                })
                .collect(),
        }
    }

    fn get_block_at(&self, translation: &Vec3) -> Option<BlockMesh> {
        let index = (translation.x as isize + (self.dimensions.width / 2) as isize)
            * (self.dimensions.width as isize)
            * (self.dimensions.height as isize)
            + (translation.y as isize + (self.dimensions.height / 2) as isize)
                * (self.dimensions.width as isize)
            + (translation.z as isize + (self.dimensions.depth / 2) as isize);
        self.block_data[index as usize].clone()
    }

    fn get_block_opacity_at(&self, translation: &Vec3) -> Opacity {
        let index = (translation.x as isize + (self.dimensions.width / 2) as isize)
            * (self.dimensions.width as isize)
            * (self.dimensions.height as isize)
            + (translation.y as isize + (self.dimensions.height / 2) as isize)
                * (self.dimensions.width as isize)
            + (translation.z as isize + (self.dimensions.depth / 2) as isize);
        let block = self.block_data[index as usize].clone();

        if let Some(block) = block {
            block.opacity
        } else {
            Opacity::Transparent(255)
        }
    }
}

enum Face {
    Front,
    Back,
    Right,
    Left,
    Top,
    Bottom,
}

fn get_face_mesh(
    face: Face,
    pos: Vec3,
    atlas_size: Vec2,
    uv: Rect,
) -> [([f32; 3], [f32; 3], [f32; 2]); 4] {
    let min = pos;
    let max = pos + Vec3::splat(BLOCK_HALF_SIZE * 2.0);
    let leftx = uv.min.x / atlas_size.x;
    let rightx = uv.max.x / atlas_size.x;
    let boty = uv.min.y / atlas_size.y;
    let topy = uv.max.y / atlas_size.y;
    // Truthfully stolen from bevy cuboid Meshable instance :)
    // Suppose Y-up right hand, and camera look from +Z to -Z
    match face {
        Face::Front => [
            ([min.x, min.y, max.z], [0.0, 0.0, 1.0], [leftx, boty]),
            ([max.x, min.y, max.z], [0.0, 0.0, 1.0], [rightx, boty]),
            ([max.x, max.y, max.z], [0.0, 0.0, 1.0], [rightx, topy]),
            ([min.x, max.y, max.z], [0.0, 0.0, 1.0], [leftx, topy]),
        ],
        Face::Back => [
            ([min.x, max.y, min.z], [0.0, 0.0, -1.0], [leftx, topy]),
            ([max.x, max.y, min.z], [0.0, 0.0, -1.0], [rightx, topy]),
            ([max.x, min.y, min.z], [0.0, 0.0, -1.0], [rightx, boty]),
            ([min.x, min.y, min.z], [0.0, 0.0, -1.0], [leftx, boty]),
        ],
        Face::Right => [
            ([max.x, min.y, min.z], [1.0, 0.0, 0.0], [leftx, boty]),
            ([max.x, max.y, min.z], [1.0, 0.0, 0.0], [leftx, topy]),
            ([max.x, max.y, max.z], [1.0, 0.0, 0.0], [rightx, topy]),
            ([max.x, min.y, max.z], [1.0, 0.0, 0.0], [rightx, boty]),
        ],
        Face::Left => [
            ([min.x, min.y, max.z], [-1.0, 0.0, 0.0], [leftx, boty]),
            ([min.x, max.y, max.z], [-1.0, 0.0, 0.0], [leftx, topy]),
            ([min.x, max.y, min.z], [-1.0, 0.0, 0.0], [rightx, topy]),
            ([min.x, min.y, min.z], [-1.0, 0.0, 0.0], [rightx, boty]),
        ],
        Face::Top => [
            ([max.x, max.y, min.z], [0.0, 1.0, 0.0], [rightx, boty]),
            ([min.x, max.y, min.z], [0.0, 1.0, 0.0], [leftx, boty]),
            ([min.x, max.y, max.z], [0.0, 1.0, 0.0], [leftx, topy]),
            ([max.x, max.y, max.z], [0.0, 1.0, 0.0], [rightx, topy]),
        ],
        Face::Bottom => [
            ([max.x, min.y, max.z], [0.0, -1.0, 0.0], [rightx, boty]),
            ([min.x, min.y, max.z], [0.0, -1.0, 0.0], [leftx, boty]),
            ([min.x, min.y, min.z], [0.0, -1.0, 0.0], [leftx, topy]),
            ([max.x, min.y, min.z], [0.0, -1.0, 0.0], [rightx, topy]),
        ],
    }
}

impl Meshable for ChunkMesh {
    type Output = Mesh;

    fn mesh(&self) -> Self::Output {
        // keeps track of curent index for indices
        let mut indice = 0;
        let mut indices = Vec::new();
        let mut vertices = Vec::new();
        for x in 0..self.dimensions.width {
            for y in 0..self.dimensions.height {
                for z in 0..self.dimensions.depth {
                    // this is a position of block in the chunk(center of the block)
                    let pos = Vec3::new(
                        (x as isize - (self.dimensions.width / 2) as isize) as f32,
                        (y as isize - (self.dimensions.height / 2) as isize) as f32,
                        (z as isize - (self.dimensions.depth / 2) as isize) as f32,
                    );
                    // if current block is air, we don't need to do anything
                    if self.get_block_at(&pos).is_none() {
                        continue;
                    }
                    // if we're at the front chunk border or there is no block in front
                    if z == 0
                        || (z > 0 && self.get_block_opacity_at(&(pos - Vec3::Z)) != Opacity::Opaque)
                    {
                        if let Some(block) = self.get_block_at(&pos) {
                            vertices.push(get_face_mesh(
                                Face::Back,
                                pos * BLOCK_HALF_SIZE * 2.0,
                                self.atlas_size,
                                *block.textures.front(),
                            ));
                            indices.extend_from_slice(&[
                                indice,
                                indice + 1,
                                indice + 2,
                                indice + 2,
                                indice + 3,
                                indice,
                            ]);
                            indice += 4;
                        }
                    }
                    // if we're at the back chunk border or there is no block before
                    if (z == self.dimensions.depth - 1)
                        || (z < self.dimensions.depth - 1
                            && self.get_block_opacity_at(&(pos + Vec3::Z)) != Opacity::Opaque)
                    {
                        if let Some(block) = self.get_block_at(&pos) {
                            vertices.push(get_face_mesh(
                                Face::Front,
                                pos * BLOCK_HALF_SIZE * 2.0,
                                self.atlas_size,
                                *block.textures.back(),
                            ));
                            indices.extend_from_slice(&[
                                indice,
                                indice + 1,
                                indice + 2,
                                indice + 2,
                                indice + 3,
                                indice,
                            ]);
                            indice += 4;
                        }
                    }
                    // if we're at the left chunk border or there is no block on the left
                    if x == 0
                        || (x > 0 && self.get_block_opacity_at(&(pos - Vec3::X)) != Opacity::Opaque)
                    {
                        if let Some(block) = self.get_block_at(&pos) {
                            vertices.push(get_face_mesh(
                                Face::Left,
                                pos * BLOCK_HALF_SIZE * 2.0,
                                self.atlas_size,
                                *block.textures.left(),
                            ));
                            indices.extend_from_slice(&[
                                indice,
                                indice + 1,
                                indice + 2,
                                indice + 2,
                                indice + 3,
                                indice,
                            ]);
                            indice += 4;
                        }
                    }

                    // if we're at the right chunk border or there is no block on the right
                    if (x == self.dimensions.width - 1)
                        || (x < self.dimensions.width - 1
                            && self.get_block_opacity_at(&(pos + Vec3::X)) != Opacity::Opaque)
                    {
                        if let Some(block) = self.get_block_at(&pos) {
                            vertices.push(get_face_mesh(
                                Face::Right,
                                pos * BLOCK_HALF_SIZE * 2.0,
                                self.atlas_size,
                                *block.textures.right(),
                            ));
                            indices.extend_from_slice(&[
                                indice,
                                indice + 1,
                                indice + 2,
                                indice + 2,
                                indice + 3,
                                indice,
                            ]);
                            indice += 4;
                        }
                    }

                    // if we're at the bottom chunk border or there is no block beneath
                    if (y == 0)
                        || (y > 0 && self.get_block_opacity_at(&(pos - Vec3::Y)) != Opacity::Opaque)
                    {
                        if let Some(block) = self.get_block_at(&pos) {
                            vertices.push(get_face_mesh(
                                Face::Bottom,
                                pos * BLOCK_HALF_SIZE * 2.0,
                                self.atlas_size,
                                *block.textures.bottom(),
                            ));
                            indices.extend_from_slice(&[
                                indice,
                                indice + 1,
                                indice + 2,
                                indice + 2,
                                indice + 3,
                                indice,
                            ]);
                            indice += 4;
                        }
                    }

                    // if we're at the top chunk border or there is no block at the top
                    if (y == self.dimensions.height - 1)
                        || (y < self.dimensions.height - 1
                            && self.get_block_opacity_at(&(pos + Vec3::Y)) != Opacity::Opaque)
                    {
                        if let Some(block) = self.get_block_at(&pos) {
                            vertices.push(get_face_mesh(
                                Face::Top,
                                pos * BLOCK_HALF_SIZE * 2.0,
                                self.atlas_size,
                                *block.textures.top(),
                            ));
                            indices.extend_from_slice(&[
                                indice,
                                indice + 1,
                                indice + 2,
                                indice + 2,
                                indice + 3,
                                indice,
                            ]);
                            indice += 4;
                        }
                    }
                }
            }
        }
        let positions: Vec<_> = vertices.iter().flatten().map(|(p, _, _)| *p).collect();
        let normals: Vec<_> = vertices.iter().flatten().map(|(_, n, _)| *n).collect();
        let uvs: Vec<_> = vertices.iter().flatten().map(|(_, _, uv)| *uv).collect();
        let indices = Indices::U32(indices);

        Mesh::new(
            bevy::render::render_resource::PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(indices)
    }
}
