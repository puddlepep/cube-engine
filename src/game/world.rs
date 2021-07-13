use std::{cmp::min, collections::HashMap, f32::consts::PI, time::{SystemTime, UNIX_EPOCH}};

use cgmath::{Matrix3, MetricSpace, Rad, Vector3, VectorSpace, num_traits::clamp};
use image::DynamicImage;
use noise::{OpenSimplex, Seedable};

use super::{CHUNKS_GEN_PER_FRAME, RENDER_DISTANCE, chunk::{self, Chunk, block::{Block, BlockList}}, color::Color, renderer::Renderer};

fn smoothstep(edge0: f32, edge1: f32, input: f32) -> f32 {
    let x = clamp((input - edge0) / (edge1 - edge0), 0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

pub struct World {
    pub chunks: HashMap<Vector3<i32>, Chunk>,
    pub chunk_queue: Vec<Vector3<i32>>,
    pub seed: u32,
    pub simplex: OpenSimplex,
    pub block_list: BlockList,
    pub block_atlas: DynamicImage,

    pub sky_color: Color,
    pub day_sky_color: Color,
    pub night_sky_color: Color,

    pub daylight_color: Color,
    pub moonlight_color: Color,

    pub time: f64,
}

impl World {

    // in seconds
    pub const DAY_LENGTH: f64 = 120.0;

    // the percent of day used for transitioning between night and day.
    pub const TRANSITION_PORTION: f32 = 0.05;

    pub fn new() -> World {

        let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u32;

        World {
            chunks: HashMap::new(),
            chunk_queue: Vec::new(),
            seed,
            simplex: OpenSimplex::new().set_seed(seed),
            block_list: BlockList::initialize(),
            block_atlas: image::open("./src/game/data/blocks/atlas.png").unwrap(),

            sky_color: Color::from_u32(120, 190, 255),
            day_sky_color: Color::from_u32(120, 190, 255),
            night_sky_color: Color::from_u32(4, 4, 10),

            daylight_color: Color::from_rgb(1.0, 1.0, 1.0),
            moonlight_color: Color::from_u32(64, 90, 128),

            time: 0.0,
        }
    }

    pub fn update(&mut self, renderer: &mut Renderer, delta: f32) {
        
        self.time += delta as f64;
        
        let local_time = self.time % World::DAY_LENGTH;
        let angle = Rad((local_time / World::DAY_LENGTH) as f32 * (PI * 2.0));
        
        let sunlight_direction = Matrix3::from_angle_z(angle) * Vector3::new(1.0, 0.0, 0.0);
        let moonlight_direction = Matrix3::from_angle_z(angle - Rad(PI)) * Vector3::new(1.0, 0.0, 0.0);

        let trans_angle_delta = PI * World::TRANSITION_PORTION * 0.5;
        let night_trans = smoothstep(PI - trans_angle_delta, PI + trans_angle_delta, angle.0);

        let day_trans_0 = smoothstep(2.0 * PI, (2.0 * PI) - trans_angle_delta, angle.0) * 0.5 + 0.5;
        let day_trans_1 = smoothstep(trans_angle_delta, 0.0, angle.0) * 0.5;
        let day_trans = if angle.0 > 0.0 && angle.0 < PI { day_trans_1 } else { day_trans_0 };

        // 0..1, night..day
        let transition: f32;
        if angle.0 > PI / 2.0 && angle.0 < (3.0 * PI) / 2.0 {
            transition = 1.0 - night_trans;
        }
        else {
            transition = 1.0 - day_trans;
        }

        self.sky_color = Color::lerp(self.night_sky_color, self.day_sky_color, transition);
        renderer.default_uniforms.data.light_color = Color::lerp(self.moonlight_color, self.daylight_color, transition).into();
        renderer.default_uniforms.data.light_direction = moonlight_direction.lerp(sunlight_direction, transition).into();

        // Go through the chunk queue one frame at a time, so as to smooth out the FPS a bit.

        let x = min(self.chunk_queue.len(), CHUNKS_GEN_PER_FRAME as usize);
        for _ in 0..x {
            let position = self.chunk_queue.remove(0);
            let mut chunk = Chunk::new(position, self);

            chunk.generate_mesh(&renderer.device, self);
            self.chunks.insert(position, chunk);
        }

    }

    // Attempts to append a chunk to the generation queue.
    // The return result is whether or not the chunk exists or was entered into the queue.
    pub fn generate_chunk(&mut self, at: &Vector3<i32>, player_pos: &Vector3<f32>) -> bool {

        match self.chunks.get(at) {
            Some(_) => {
                true
            },
            None => {
    
                if self.chunk_queue.contains(at) { return true; }

                let fat =  Vector3::new(at.x as f32 + 0.5, at.y as f32 + 0.5, at.z as f32 + 0.5);
                let dist = fat.distance(*player_pos);
    
                if dist <= RENDER_DISTANCE as f32  {

                    self.chunk_queue.push(*at);
                    true
                }
                else {
                    false
                }
    
            }
        }
    }

    pub fn update_chunk_mesh(&mut self, at: &Vector3<i32>, renderer: &Renderer) {

        match self.chunks.remove(at) {
            Some(mut chunk) => {
                chunk.generate_mesh(&renderer.device, self);
                self.chunks.insert(*at, chunk);
            },
            None => ()
        }
    }

    pub fn get_block_at(&self, position: Vector3<i32>) -> Option<&Block> {

        let cs = chunk::CHUNK_SIZE as i32;
        let csf = chunk::CHUNK_SIZE as f32;

        let chunk_position = Vector3::new(position.x as f32, position.y as f32, position.z as f32) / csf;
        let chunk_position = Vector3::new(chunk_position.x.floor() as i32, chunk_position.y.floor() as i32, chunk_position.z.floor() as i32);

        let block_position = (position % cs + Vector3::new(cs, cs, cs)) % cs;
        // ^ modulo, as '%' is actually remainder.

        match self.chunks.get(&chunk_position) {
            Some(chunk) => {
                let id = chunk.grid[block_position.x as usize][block_position.y as usize][block_position.z as usize];
                self.block_list.blocks.get(id as usize)
            },
            None => None
        }
    
    }
    
    
}
