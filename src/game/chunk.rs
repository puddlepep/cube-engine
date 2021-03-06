pub mod block;

use block::Block;
use super::{World, renderer::mesh::Mesh};
use super::renderer::vertex::Vertex;
use cgmath::Vector3;

pub const CHUNK_SIZE: usize = 16;

pub const GRID_MAX: usize = CHUNK_SIZE - 1;

pub struct Chunk {
    pub position: cgmath::Vector3<i32>,
    pub grid: Box<[[[u32; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>,
    pub mesh: Option<Mesh>,
    
    pub should_regen_mesh: bool,
    //pub active_neighbors: u8,
}

pub enum Face {
    Front,
    Back,
    Left,
    Right,
    Up,
    Down,
}

impl Chunk {

    pub const FORWARD: Vector3<i32> = Vector3::new(0, 0, 1);
    pub const BACKWARD: Vector3<i32> = Vector3::new(0, 0, -1);
    pub const UP: Vector3<i32> = Vector3::new(0, 1, 0);
    pub const DOWN: Vector3<i32> = Vector3::new(0, -1, 0);
    pub const LEFT: Vector3<i32> = Vector3::new(-1, 0, 0);
    pub const RIGHT: Vector3<i32> = Vector3::new(1, 0, 0);

    pub fn get_world_position(&self) -> Vector3<f32> {
        Vector3::new(
            self.position.x as f32 + 0.5,
            self.position.y as f32 + 0.5,
            self.position.z as f32 + 0.5,
        ) * CHUNK_SIZE as f32
    }

    pub fn get_noise_value_at(position: cgmath::Vector3<i32>, world: &World) -> f32 {

        let max_height: f32 = 64.0;
        let noise_scale = 0.03;
        let nx = position.x as f32 * noise_scale;
        let ny = position.y as f32 * noise_scale;
        let nz = position.z as f32 * noise_scale;

        let mut value = world.noise.get_noise3d(nx, ny, nz);
        value += 1.0 - ((position.y as f32 + 64.0) / max_height);
        value
    }

    pub fn new(position: cgmath::Vector3<i32>, world: &World) -> Chunk {

        let mut grid = Box::new([[[0; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]);

        for z in 0..CHUNK_SIZE {
            for y in (0..CHUNK_SIZE).rev() {
                for x in 0..CHUNK_SIZE {
                    
                    let block_pos = (position * CHUNK_SIZE as i32) + cgmath::Vector3::new(x as i32, y as i32, z as i32);
                    
                    let value: f32;
                    if block_pos.y < -64 {
                        value = 1.0;
                    }
                    else if block_pos.y > 64 {
                        value = 0.0;
                    }
                    else {
                        value = Chunk::get_noise_value_at(block_pos, world);
                    }
            
                    let mut covered = false;

                    if y == GRID_MAX {
                        let block_above = Chunk::get_noise_value_at(block_pos + Vector3::new(0, 1, 0), world);
                        if block_above > 0.0 { covered = true; }
                    }
                    else if grid[x][y+1][z] != 0 {
                        covered = true;
                    }

                    if value > 0.0 {

                        let stone = world.block_list.get_block("stone").unwrap().id;
                        let dirt = world.block_list.get_block("dirt").unwrap().id;
                        let grass = world.block_list.get_block("grass").unwrap().id;

                        if value > 0.1 || covered {
                            if value > 0.15 {
                                grid[x][y][z] = stone;
                            }
                            else {
                                grid[x][y][z] = dirt;
                            }
                        }
                        else {
                            grid[x][y][z] = grass;
                        }
                    }

                    // if value > 0.5 {
                    //     if value > 0.6 || covered {
                    //         if value > 0.65 {
                    //             grid[x][y][z] = world.block_list.get_block(String::from("stone")).unwrap().id;
                    //         }
                    //         else {
                    //             grid[x][y][z] = world.block_list.get_block(String::from("dirt")).unwrap().id;
                    //         }
                    //     }
                    //     else {
                    //         grid[x][y][z] = world.block_list.get_block(String::from("grass")).unwrap().id;
                    //     }
                    // }
                }
            }
        }

        Chunk { grid, position, mesh: None, should_regen_mesh: true } //active_neighbors: 0, should_regen_mesh: false }

    }

    pub fn center(&self) -> Vector3<f32> {
        Vector3::new(self.position.x as f32 + 0.5, self.position.y as f32 + 0.5, self.position.z as f32 + 0.5)
    }

    // Assumed ordered CCW.
    pub fn triangulate_quad(p1: Vector3<f32>, p2: Vector3<f32>, p3: Vector3<f32>, p4: Vector3<f32>, vertices: &mut Vec<Vertex>, indices: &mut Vec<u16>, normal: Vector3<f32>, tex_coords: ([f32; 2], [f32; 2], [f32; 2], [f32; 2])) {
        
        vertices.push(Vertex { position: p1.into(), normal: normal.into(), tex_coords: tex_coords.0 });
        vertices.push(Vertex { position: p2.into(), normal: normal.into(), tex_coords: tex_coords.1 });
        vertices.push(Vertex { position: p3.into(), normal: normal.into(), tex_coords: tex_coords.2 });
        vertices.push(Vertex { position: p4.into(), normal: normal.into(), tex_coords: tex_coords.3 });

        let l = vertices.len() as u16;
        indices.push(l - 4);
        indices.push(l - 3);
        indices.push(l - 2);

        indices.push(l - 4);
        indices.push(l - 2);
        indices.push(l - 1);

    }

    pub fn build_face(origin: Vector3<f32>, face: Face, vertices: &mut Vec<Vertex>, indices: &mut Vec<u16>, block: &Block, atlas: &image::DynamicImage) {

        let a = origin;
        let b = a + Block::RIGHT;
        let c = b + Block::FORWARD;
        let d = c + Block::LEFT;

        let e = a + Block::UP;
        let f = e + Block::RIGHT;
        let g = f + Block::FORWARD;
        let h = g + Block::LEFT;

        match face {

            Face::Front => {
                Chunk::triangulate_quad(h, g, c, d, vertices, indices, Block::FORWARD, block.get_tex_coords(block::Side::Front, atlas));
            }

            Face::Back => {
                Chunk::triangulate_quad(f, e, a, b, vertices, indices, Block::BACKWARD, block.get_tex_coords(block::Side::Back, atlas));
            }

            Face::Left => {
                Chunk::triangulate_quad(e, h, d, a, vertices, indices, Block::LEFT, block.get_tex_coords(block::Side::Left, atlas));
            }

            Face::Right => {
                Chunk::triangulate_quad(g, f, b, c, vertices, indices, Block::RIGHT, block.get_tex_coords(block::Side::Right, atlas));
            }

            Face::Up => {
                Chunk::triangulate_quad(g, h, e, f, vertices, indices, Block::UP, block.get_tex_coords(block::Side::Top, atlas));
            }

            Face::Down => {
                Chunk::triangulate_quad(c, b, a, d, vertices, indices, Block::DOWN, block.get_tex_coords(block::Side::Bottom, atlas));
            }

        }
    }

    // Returns the vertices and indices of a chunk mesh.
    pub fn generate_mesh_parts(&self, world: &World) -> (Vec<Vertex>, Vec<u16>) {
        
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u16> = Vec::new();

        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {

                    let block_id = self.grid[x][y][z];
                    if block_id != 0 {
            
                        let offset: Vector3<f32> = Vector3::new(x as f32, y as f32, z as f32);
                        let chunk_position: Vector3<f32> = Vector3::new(self.position.x as f32, self.position.y as f32, self.position.z as f32) * CHUNK_SIZE as f32;
            
                        let origin = chunk_position + offset;
                        let atlas = &world.block_atlas;

                        let block = world.block_list.blocks.get(block_id as usize).unwrap();
                        
                        if x == 0 {
                            match world.chunks.get(&(self.position + Chunk::LEFT)) {
                                Some(chunk) => {
                                    if chunk.grid[GRID_MAX][y][z] == 0 {
                                        Chunk::build_face(origin, Face::Left, &mut vertices, &mut indices, block, atlas);
                                    }
                                }
                                None => {
                                    Chunk::build_face(origin, Face::Left, &mut vertices, &mut indices, block, atlas);
                                }
                            }
                        }
                        else if self.grid[x-1][y][z] == 0 {
                            Chunk::build_face(origin, Face::Left, &mut vertices, &mut indices, block, atlas);
                        }

                        if x == GRID_MAX {
                            match world.chunks.get(&(self.position + Chunk::RIGHT)) {
                                Some(chunk) => {
                                    if chunk.grid[0][y][z] == 0 {
                                        Chunk::build_face(origin, Face::Right, &mut vertices, &mut indices, block, atlas);
                                    }
                                }
                                None => {
                                    Chunk::build_face(origin, Face::Right, &mut vertices, &mut indices, block, atlas);
                                }
                            }
                        }
                        else if self.grid[x+1][y][z] == 0 {
                            Chunk::build_face(origin, Face::Right, &mut vertices, &mut indices, block, atlas);
                        }
            
                        if y == 0 {
                            match world.chunks.get(&(self.position + Chunk::DOWN)) {
                                Some(chunk) => {
                                    if chunk.grid[x][GRID_MAX][z] == 0 {
                                        Chunk::build_face(origin, Face::Down, &mut vertices, &mut indices, block, atlas);
                                    }
                                }
                                None => {
                                    Chunk::build_face(origin, Face::Down, &mut vertices, &mut indices, block, atlas);
                                }
                            }
                        }
                        else if self.grid[x][y-1][z] == 0 {
                            Chunk::build_face(origin, Face::Down, &mut vertices, &mut indices, block, atlas);
                        }
            
                        if y == GRID_MAX {
                            match world.chunks.get(&(self.position + Chunk::UP)) {
                                Some(chunk) => {
                                    if chunk.grid[x][0][z] == 0 {
                                        Chunk::build_face(origin, Face::Up, &mut vertices, &mut indices, block, atlas);
                                    }
                                }
                                None => {
                                    Chunk::build_face(origin, Face::Up, &mut vertices, &mut indices, block, atlas);
                                }
                            }
                        }
                        else if self.grid[x][y+1][z] == 0 {
                            Chunk::build_face(origin, Face::Up, &mut vertices, &mut indices, block, atlas);
                        }

                        if z == 0 {
                            match world.chunks.get(&(self.position + Chunk::BACKWARD)) {
                                Some(chunk) => {
                                    if chunk.grid[x][y][GRID_MAX] == 0 {
                                        Chunk::build_face(origin, Face::Front, &mut vertices, &mut indices, block, atlas);
                                    }
                                }
                                None => {
                                    Chunk::build_face(origin, Face::Front, &mut vertices, &mut indices, block, atlas);
                                }
                            }
                        }
                        else if self.grid[x][y][z-1] == 0 {
                            Chunk::build_face(origin, Face::Front, &mut vertices, &mut indices, block, atlas);
                        }
            
                        if z == GRID_MAX {
                            match world.chunks.get(&(self.position + Chunk::FORWARD)) {
                                Some(chunk) => {
                                    if chunk.grid[x][y][0] == 0 {
                                        Chunk::build_face(origin, Face::Back, &mut vertices, &mut indices, block, atlas);
                                    }
                                }
                                None => {
                                    Chunk::build_face(origin, Face::Back, &mut vertices, &mut indices, block, atlas);
                                }
                            }
                        }
                        else if self.grid[x][y][z+1] == 0 {
                            Chunk::build_face(origin, Face::Back, &mut vertices, &mut indices, block, atlas);
                        }
            
                    }
                }
            }
        }

        (vertices, indices)

        //self.should_regen_mesh = false;
        //if vertices.len() == 0 { self.mesh = None; }
        //else { self.mesh = Some(Mesh::new(device, vertices, indices)); }
    }
}