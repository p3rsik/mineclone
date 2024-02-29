use bevy::{
    prelude::*,
    render::{mesh::Indices, render_asset::RenderAssetUsages},
};

use crate::block::BLOCK_HALF_SIZE;

use super::Chunk;

enum Face {
    Front,
    Back,
    Right,
    Left,
    Top,
    Bottom,
}

fn get_face_mesh(face: Face, pos: Vec3) -> [([f32; 3], [f32; 3], [f32; 2]); 4] {
    let min = pos;
    let max = pos + Vec3::splat(BLOCK_HALF_SIZE * 2.0);
    // Truthfully stolen from bevy cuboid Meshable instance :)
    // Suppose Y-up right hand, and camera look from +Z to -Z
    match face {
        Face::Front => [
            ([min.x, min.y, max.z], [0.0, 0.0, 1.0], [0.0, 0.0]),
            ([max.x, min.y, max.z], [0.0, 0.0, 1.0], [1.0, 0.0]),
            ([max.x, max.y, max.z], [0.0, 0.0, 1.0], [1.0, 1.0]),
            ([min.x, max.y, max.z], [0.0, 0.0, 1.0], [0.0, 1.0]),
        ],
        Face::Back => [
            ([min.x, max.y, min.z], [0.0, 0.0, -1.0], [1.0, 0.0]),
            ([max.x, max.y, min.z], [0.0, 0.0, -1.0], [0.0, 0.0]),
            ([max.x, min.y, min.z], [0.0, 0.0, -1.0], [0.0, 1.0]),
            ([min.x, min.y, min.z], [0.0, 0.0, -1.0], [1.0, 1.0]),
        ],
        Face::Right => [
            ([max.x, min.y, min.z], [1.0, 0.0, 0.0], [0.0, 0.0]),
            ([max.x, max.y, min.z], [1.0, 0.0, 0.0], [1.0, 0.0]),
            ([max.x, max.y, max.z], [1.0, 0.0, 0.0], [1.0, 1.0]),
            ([max.x, min.y, max.z], [1.0, 0.0, 0.0], [0.0, 1.0]),
        ],
        Face::Left => [
            ([min.x, min.y, max.z], [-1.0, 0.0, 0.0], [1.0, 0.0]),
            ([min.x, max.y, max.z], [-1.0, 0.0, 0.0], [0.0, 0.0]),
            ([min.x, max.y, min.z], [-1.0, 0.0, 0.0], [0.0, 1.0]),
            ([min.x, min.y, min.z], [-1.0, 0.0, 0.0], [1.0, 1.0]),
        ],
        Face::Top => [
            ([max.x, max.y, min.z], [0.0, 1.0, 0.0], [1.0, 0.0]),
            ([min.x, max.y, min.z], [0.0, 1.0, 0.0], [0.0, 0.0]),
            ([min.x, max.y, max.z], [0.0, 1.0, 0.0], [0.0, 1.0]),
            ([max.x, max.y, max.z], [0.0, 1.0, 0.0], [1.0, 1.0]),
        ],
        Face::Bottom => [
            ([max.x, min.y, max.z], [0.0, -1.0, 0.0], [0.0, 0.0]),
            ([min.x, min.y, max.z], [0.0, -1.0, 0.0], [1.0, 0.0]),
            ([min.x, min.y, min.z], [0.0, -1.0, 0.0], [1.0, 1.0]),
            ([max.x, min.y, min.z], [0.0, -1.0, 0.0], [0.0, 1.0]),
        ],
    }
}

impl Meshable for Chunk {
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
                    if self.get_block_at(&pos).0 == 0 {
                        continue;
                    }
                    // if we're at the front chunk border or there is no block in front
                    if z == 0 || (z > 0 && self.get_block_at(&(pos - Vec3::Z)).0 == 0) {
                        vertices.push(get_face_mesh(Face::Back, pos * BLOCK_HALF_SIZE * 2.0));
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
                    // if we're at the back chunk border or there is no block before
                    if (z == self.dimensions.depth - 1)
                        || (z < self.dimensions.depth - 1
                            && self.get_block_at(&(pos + Vec3::Z)).0 == 0)
                    {
                        vertices.push(get_face_mesh(Face::Front, pos * BLOCK_HALF_SIZE * 2.0));
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
                    // if we're at the left chunk border or there is no block on the left
                    if x == 0 || (x > 0 && self.get_block_at(&(pos - Vec3::X)).0 == 0) {
                        vertices.push(get_face_mesh(Face::Left, pos * BLOCK_HALF_SIZE * 2.0));
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

                    // if we're at the right chunk border or there is no block on the right
                    if (x == self.dimensions.width - 1)
                        || (x < self.dimensions.width - 1
                            && self.get_block_at(&(pos + Vec3::X)).0 == 0)
                    {
                        vertices.push(get_face_mesh(Face::Right, pos * BLOCK_HALF_SIZE * 2.0));
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

                    // if we're at the bottom chunk border or there is no block beneath
                    if (y == 0) || (y > 0 && self.get_block_at(&(pos - Vec3::Y)).0 == 0) {
                        vertices.push(get_face_mesh(Face::Bottom, pos * BLOCK_HALF_SIZE * 2.0));
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

                    // if we're at the top chunk border or there is no block at the top
                    if (y == self.dimensions.height - 1)
                        || (y < self.dimensions.height - 1
                            && self.get_block_at(&(pos + Vec3::Y)).0 == 0)
                    {
                        vertices.push(get_face_mesh(Face::Top, pos * BLOCK_HALF_SIZE * 2.0));
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
