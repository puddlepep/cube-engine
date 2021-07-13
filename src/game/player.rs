use cgmath::{InnerSpace, Vector3};

use super::World;
use super::collision;
use super::input_handler::InputMap;
use super::camera::{self, Camera};
use winit::event::VirtualKeyCode as Key;

pub struct Player {

    pub camera: super::camera::Camera,
    pub speed: f32,
    pub acceleration: f32,
    pub jump_force: f32,
    pub position: cgmath::Vector3<f32>,
    pub velocity: cgmath::Vector3<f32>,
    pub gravity_vel: cgmath::Vector3<f32>,
    pub gravity: f32,

}

impl Player {

    pub fn new(renderer: &super::Renderer) -> Player {
        Player {
            camera: Camera {
                position: (0.0, 0.0, 0.0).into(),
                yaw: 0.0,
                pitch: 0.0,
                speed: 10.0,
                sensitivity: 0.5,
                fovy: cgmath::Deg(90.0).into(),
                near: 0.1,
                far: 300.0,

                width: renderer.swap_chain_desc.width,
                height: renderer.swap_chain_desc.height,
            },
            speed: 20.0,
            acceleration: 5.0,
            jump_force: 0.06,
            position: cgmath::Vector3::new(0.0, 0.0, 0.0),
            velocity: cgmath::Vector3::new(0.0, 0.0, 0.0),
            gravity_vel: cgmath::Vector3::new(0.0, 0.0, 0.0),
            gravity: 0.2,
        }
    }

    pub fn get_block_pos(&self, position: Vector3<f32>) -> Vector3<i32> {
        let p = position;
        Vector3::new(p.x.floor() as i32, p.y.floor() as i32, p.z.ceil() as i32)
    }

    pub fn update(&mut self, input: &mut InputMap, world: &World, delta: f32) {

        let speed = self.speed * delta / 5.0;
        let sensitivity = self.camera.sensitivity * delta;
        
        let (forward, right, _) = self.camera.get_headings();
        let mut forward = forward - Vector3::new(0.0, forward.y, 0.0);

        if forward.magnitude() != 0.0 { forward = forward.normalize(); }
        
        let x_input = -(input.get_key(Key::A).held as i32) + input.get_key(Key::D).held as i32;
        let z_input = -(input.get_key(Key::S).held as i32) + input.get_key(Key::W).held as i32;
        let mut xz_dir = (forward * z_input as f32) + (right * x_input as f32);
        
        if xz_dir.magnitude() != 0.0 { xz_dir = xz_dir.normalize(); }
        let mut xz_vel = xz_dir * speed;
        
        let mut is_on_floor = false;
        self.gravity_vel += Vector3::new(0.0, -self.gravity * delta, 0.0);

        let x_collider = collision::CircleCollider::new(self.position + Vector3::new(xz_vel.x, 0.0, 0.0), 0.25);
        let z_collider = collision::CircleCollider::new(self.position + Vector3::new(0.0, 0.0, xz_vel.z), 0.25);
        let gravity_collider = collision::CircleCollider::new(self.position + self.gravity_vel, 0.25);
        for x in -2..2 {
            for z in -2..2 {

                let gravity_block_position = self.get_block_pos(self.position + self.gravity_vel) + Vector3::new(x, 0, z);
                let x_block_position = self.get_block_pos(self.position + Vector3::new(xz_vel.x, 0.0, 0.0)) + Vector3::new(x, 0, z);
                let z_block_position = self.get_block_pos(self.position + Vector3::new(0.0, 0.0, xz_vel.z)) + Vector3::new(x, 0, z);

                let gravity_block = world.get_block_at(gravity_block_position);
                let x_block = world.get_block_at(x_block_position);
                let z_block = world.get_block_at(z_block_position);

                // gravity --
                if gravity_block.is_some() && gravity_block.unwrap().id != 0 {
                    let cube_collider = collision::CubeCollider::new(Vector3::new(
                        gravity_block_position.x as f32, 
                        gravity_block_position.y as f32, 
                        gravity_block_position.z as f32
                    ), Vector3::new(1.0, 1.0, -1.0));

                    if collision::circle_cube(&gravity_collider, &cube_collider).is_some() {
                        self.position.y = self.position.y.round();
                        self.gravity_vel.y = 0.0;
                        is_on_floor = true;
                    }
                }

                // x axis
                if x_block.is_some() && x_block.unwrap().id != 0 {
                    let cube_collider = collision::CubeCollider::new(Vector3::new (
                        x_block_position.x as f32,
                        x_block_position.y as f32,
                        x_block_position.z as f32,
                    ), Vector3::new(1.0, 1.0, -1.0));

                    if collision::circle_cube(&x_collider, &cube_collider).is_some() {
                        xz_vel.x = 0.0;
                    }
                }

                // z axis
                if z_block.is_some() && z_block.unwrap().id != 0 {
                    let cube_collider = collision::CubeCollider::new(Vector3::new(
                        z_block_position.x as f32,
                        z_block_position.y as f32,
                        z_block_position.z as f32,
                    ), Vector3::new(1.0, 1.0, -1.0));

                    if collision::circle_cube(&z_collider, &cube_collider).is_some() {
                        xz_vel.z = 0.0;
                    }
                }
            }
        }
   
        if is_on_floor && input.get_key(Key::Space).held {
            self.gravity_vel.y = self.jump_force;
        }

        self.position += self.gravity_vel + xz_vel;
        self.camera.position = self.position + Vector3::new(0.0, 1.5, 0.0);

        let mouse_x = input.mouse.delta.x;
        let mouse_y = input.mouse.delta.y;

        self.camera.pitch -= mouse_y * sensitivity;
        self.camera.yaw += mouse_x * sensitivity;

        if self.camera.pitch < camera::PITCH_MIN { self.camera.pitch = camera::PITCH_MIN; }
        if self.camera.pitch > camera::PITCH_MAX { self.camera.pitch = camera::PITCH_MAX; }
        
    }

}
