mod renderer;
mod chunk;
mod input_handler;
mod camera;
mod color;
mod player;
mod collision;
mod world;

use chunk::Chunk;

use winit::event::VirtualKeyCode as Key;
use winit::{event:: {Event, WindowEvent}, event_loop:: { ControlFlow, EventLoop }, window:: { WindowBuilder }};
use futures::executor::block_on;

use crate::game::world::World;

use self::renderer::Renderer;
use self::renderer::mesh::Mesh;
use cgmath::{MetricSpace, Vector3};
const RENDER_DISTANCE: u32 = 8;
const DESTROY_DISTANCE: u32 = RENDER_DISTANCE + 2;
const CHUNKS_GEN_PER_FRAME: u32 = 4;

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