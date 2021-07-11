mod renderer;
mod chunk;
mod input_handler;
mod camera;
mod color;
mod player;
mod collision;

use std::cmp::min;
use std::f32::consts::PI;
use std::time::{SystemTime, UNIX_EPOCH};

use cgmath::num_traits::clamp;
use chunk::Chunk;
use image::DynamicImage;
use noise::*;

use winit::event::VirtualKeyCode as Key;
use winit::{event:: {Event, WindowEvent}, event_loop:: { ControlFlow, EventLoop }, window:: { WindowBuilder }};
use futures::executor::block_on;

use color::Color;
use crate::game::chunk::block::BlockList;

use self::chunk::block::Block;
use self::renderer::Renderer;
use self::renderer::mesh::Mesh;
use std::collections::HashMap;
use cgmath::{MetricSpace, Rad, VectorSpace};
use cgmath::Vector3;
use cgmath::Matrix3;

const RENDER_DISTANCE: u32 = 2;
const DESTROY_DISTANCE: u32 = RENDER_DISTANCE + 2;
const CHUNKS_GEN_PER_FRAME: u32 = 2;

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
        renderer.uniforms.data.light_color = Color::lerp(self.moonlight_color, self.daylight_color, transition).into();
        renderer.uniforms.data.light_direction = moonlight_direction.lerp(sunlight_direction, transition).into();

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

pub fn run() {

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Voxel Engine")
        .with_inner_size( winit::dpi::LogicalSize::new(1280, 720) )
        .build(&event_loop)
        .unwrap();
    let mut has_focus = true;
    let mut minimized = false;
    match window.set_cursor_grab(true) { _ => () };
    window.set_cursor_visible(false);
    
    let mut world = World::new();
    println!("seed: {}", world.seed);

    let mut renderer = block_on(renderer::Renderer::new(&window));
    let mut input = input_handler::InputMap::new();
    let mut player = player::Player::new(&renderer);
    
    let mut previous_frame_time = std::time::Instant::now();
    event_loop.run(move | event, _, control_flow | {

        *control_flow = ControlFlow::Poll;

        match event {

            Event::WindowEvent { event, .. } => match event {

                WindowEvent::Resized(new_size) => {

                    if new_size.width == 0 || new_size.height == 0 {
                        minimized = true;
                    }
                    else {
                        minimized = false;
                    }

                    if !minimized {
                        renderer.resize(new_size);
                        player.camera.resize(new_size.width, new_size.height);
                    }

                },

                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                },

                WindowEvent::Focused(focus) => {
                    has_focus = focus;
                    match window.set_cursor_grab(focus) { _ => () }
                    window.set_cursor_visible(!focus);
                }

                _ => ()

            },

            Event::RedrawRequested(_) => {
                if !minimized {

                    let mut pool: Vec<&Mesh> = Vec::new();
                    for (_at, chunk) in &world.chunks {
                        match &chunk.mesh {
                            Some(mesh) => {
                                pool.push(mesh);
                            }
                            None => ()
                        }
                    }
                    
                    renderer.render(&player.camera, &pool, world.sky_color);
                }
            },

            Event::DeviceEvent { event, .. } => {
                
                if has_focus {
                    input.update(&event);
                }
            },

            Event::MainEventsCleared => {

                window.request_redraw();
                
                // Calculating delta time.
                let now = std::time::Instant::now();
                let delta = (now - previous_frame_time).as_secs_f32();
                previous_frame_time = now;
                //println!("frame time: {}", delta.as_secs_f32());

                player.update(&mut input, &world, delta);
                let fcam_pos: Vector3<f32> = Vector3::new(
                    player.camera.position.x as f32 / chunk::CHUNK_SIZE as f32,
                    player.camera.position.y as f32 / chunk::CHUNK_SIZE as f32, 
                    player.camera.position.z as f32 / chunk::CHUNK_SIZE as f32
                );
                let icam_pos: Vector3<i32> = Vector3::new(fcam_pos.x as i32, fcam_pos.y as i32, fcam_pos.z as i32);

                world.update(&mut renderer, delta);
                world.generate_chunk(&icam_pos, &fcam_pos);
                
                let mut chunks_to_loop: Vec<Vector3<i32>> = Vec::new();
                let mut chunks_to_destroy: Vec<Vector3<i32>> = Vec::new();
                for (at, chunk) in &world.chunks {
                    
                    if chunk.center().distance(fcam_pos) > DESTROY_DISTANCE as f32 {
                        chunks_to_destroy.push(*at);
                    }
                    else if chunk.active_neighbors != 6 {
                        chunks_to_loop.push(*at);
                    }

                }

                for at in chunks_to_destroy {
                    let result = world.chunks.remove(&at);
                    match result {
                        Some(_chunk) => (),
                        None => ()
                    }
                }
                
                for at in chunks_to_loop {

                    let mut n: u8 = 0;
                    n += world.generate_chunk(&(at + Chunk::FORWARD), &fcam_pos) as u8;
                    n += world.generate_chunk(&(at + Chunk::BACKWARD), &fcam_pos) as u8;
                    n += world.generate_chunk(&(at + Chunk::LEFT), &fcam_pos) as u8;
                    n += world.generate_chunk(&(at + Chunk::RIGHT), &fcam_pos) as u8;
                    n += world.generate_chunk(&(at + Chunk::UP), &fcam_pos) as u8;
                    n += world.generate_chunk(&(at + Chunk::DOWN), &fcam_pos) as u8;
                    world.chunks.get_mut(&at).unwrap().active_neighbors = n;
                }

                if input.get_key(Key::Escape).held {
                    match window.set_cursor_grab(false) { _ => () }
                    window.set_cursor_visible(true);
                }

                
                let mut chunks_to_regen: Vec<Vector3<i32>> = Vec::new();
                for (p, chunk) in &mut world.chunks {
                    if chunk.should_regen_mesh {
                        chunks_to_regen.push(*p);
                    }
                }

                for p in chunks_to_regen {
                    world.update_chunk_mesh(&p, &renderer);
                    match world.chunks.get_mut(&p) { Some(chunk) => chunk.should_regen_mesh = false, None => () }
                }

                input.post_update();

            }

            _ => ()
        }
    });
}