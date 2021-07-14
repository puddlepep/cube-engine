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

use self::input_handler::InputMap;
use self::player::Player;
use self::renderer::Renderer;
use self::renderer::mesh::Mesh;
use cgmath::{MetricSpace, Vector3};

const RENDER_DISTANCE: u32 = 8;
const DESTROY_DISTANCE: u32 = RENDER_DISTANCE + 2;
const CHUNKS_GEN_PER_FRAME: u32 = 2;
const FIXED_UPDATES_PER_SECOND: u32 = 120;

struct Game {

    delta: f32,
    time_since_last_tick: f32,
    ticks_per_second: f32,
    previous_frame_time: std::time::Instant,

    world: World,
    renderer: Renderer,
    input: InputMap,
    player: Player,

    is_focused: bool,
    is_minimized: bool,
    is_paused: bool,

}



// functions extracted from the event_loop closure because it was
// getting very crowded and frustrating to navigate in there.

// Executes consistently every x amount of time.
fn fixed_update(game: &mut Game) {
    game.player.update(&mut game.input, &game.world);
}

// Executes every time before a frame is rendered.
fn update(game: &mut Game) {

    let fcam_pos: Vector3<f32> = Vector3::new(
        game.player.camera.position.x as f32 / chunk::CHUNK_SIZE as f32,
        game.player.camera.position.y as f32 / chunk::CHUNK_SIZE as f32, 
        game.player.camera.position.z as f32 / chunk::CHUNK_SIZE as f32
    );
    let icam_pos: Vector3<i32> = Vector3::new(fcam_pos.x as i32, fcam_pos.y as i32, fcam_pos.z as i32);

    game.world.update(&mut game.renderer, game.delta);
    game.world.generate_chunk(&icam_pos, &fcam_pos);
    
    let mut chunks_to_loop: Vec<Vector3<i32>> = Vec::new();
    let mut chunks_to_destroy: Vec<Vector3<i32>> = Vec::new();
    for (at, chunk) in &game.world.chunks {
        
        if chunk.center().distance(fcam_pos) > DESTROY_DISTANCE as f32 {
            chunks_to_destroy.push(*at);
        }
        else if chunk.active_neighbors != 6 {
            chunks_to_loop.push(*at);
        }

    }

    for at in chunks_to_destroy {
        let result = game.world.chunks.remove(&at);
        match result {
            Some(_chunk) => (),
            None => ()
        }
    }
    
    for at in chunks_to_loop {

        let mut n: u8 = 0;
        n += game.world.generate_chunk(&(at + Chunk::FORWARD), &fcam_pos) as u8;
        n += game.world.generate_chunk(&(at + Chunk::BACKWARD), &fcam_pos) as u8;
        n += game.world.generate_chunk(&(at + Chunk::LEFT), &fcam_pos) as u8;
        n += game.world.generate_chunk(&(at + Chunk::RIGHT), &fcam_pos) as u8;
        n += game.world.generate_chunk(&(at + Chunk::UP), &fcam_pos) as u8;
        n += game.world.generate_chunk(&(at + Chunk::DOWN), &fcam_pos) as u8;
        game.world.chunks.get_mut(&at).unwrap().active_neighbors = n;
    }

    
    let mut chunks_to_regen: Vec<Vector3<i32>> = Vec::new();
    for (p, chunk) in &mut game.world.chunks {
        if chunk.should_regen_mesh {
            chunks_to_regen.push(*p);
        }
    }

    for p in chunks_to_regen {
        game.world.update_chunk_mesh(&p, &game.renderer);
        match game.world.chunks.get_mut(&p) { Some(chunk) => chunk.should_regen_mesh = false, None => () }
    }
}



fn render(game: &mut Game) {

    if !game.is_minimized {
        let mut pool: Vec<&Mesh> = Vec::new();
        for (_at, chunk) in &game.world.chunks {
            match &chunk.mesh {
                Some(mesh) => {
                    pool.push(mesh);
                }
                None => ()
            }
        }
        
        game.renderer.render(&game.player.camera, &pool, game.world.sky_color);
    }
}



fn resize(game: &mut Game, new_size: &winit::dpi::PhysicalSize<u32>) {
    if new_size.width == 0 || new_size.height == 0 {
        game.is_minimized = true;
    }
    else {
        game.is_minimized = false;
    }

    if !game.is_minimized {
        game.renderer.resize(*new_size);
        game.player.camera.resize(new_size.width, new_size.height);
    }
}

// --------------------------------------------------------------------------









pub fn run() {


    // Initializing the basics required for rendering.
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Voxel Engine")
        .with_inner_size( winit::dpi::LogicalSize::new(1280, 720) )
        .build(&event_loop)
        .unwrap();
    match window.set_cursor_grab(true) { _ => () };
    window.set_cursor_visible(false);

    // Setting up the game struct.
    let _world = World::new();
    let _renderer = block_on(renderer::Renderer::new(&window));
    let _input = input_handler::InputMap::new();
    let _player = player::Player::new(&_renderer);
    
    let mut game = Game {
        world: _world,
        renderer: _renderer,
        input: _input,
        player: _player,
        delta: 0.0,
        time_since_last_tick: 0.0,
        ticks_per_second: 1.0 / FIXED_UPDATES_PER_SECOND as f32,
        previous_frame_time: std::time::Instant::now(),
        
        is_focused: true,
        is_minimized: false,
        is_paused: false,
    };
    println!("seed: {}", game.world.seed);


    // THE main loop!!!!!!!!!!!!!
    event_loop.run(move | event, _, control_flow | {

        *control_flow = ControlFlow::Poll;

        match event {

            Event::WindowEvent { event, .. } => match event {

                WindowEvent::Resized(new_size) => {
                    resize(&mut game, &new_size);
                },

                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                },

                WindowEvent::Focused(focus) => {
                    game.is_focused = focus;
                    match window.set_cursor_grab(focus) { _ => () }
                    window.set_cursor_visible(!focus);
                }

                _ => ()

            },

            Event::RedrawRequested(_) => {
                render(&mut game);
            },

            Event::DeviceEvent { event, .. } => {
                if game.is_focused {
                    game.input.update(&event);
                }
            },

            Event::MainEventsCleared => {

                // Calculating delta time.
                let now = std::time::Instant::now();
                game.delta = (now - game.previous_frame_time).as_secs_f32();
                game.previous_frame_time = now;
                game.time_since_last_tick += game.delta;

                if game.input.get_key(Key::Escape).just_pressed {
                    match window.set_cursor_grab(game.is_paused) { _ => () }
                    window.set_cursor_visible(!game.is_paused);
                    game.is_paused = !game.is_paused;
                }

                while game.time_since_last_tick > game.ticks_per_second {
                    game.time_since_last_tick -= game.ticks_per_second;
                    fixed_update(&mut game);
                }

                update(&mut game);

                game.input.post_update();
                window.request_redraw();

            }

            _ => ()
        }
    });
}