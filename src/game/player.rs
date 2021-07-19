use cgmath::{InnerSpace, Vector3};

use super::World;
use super::camera::frustum::Frustum;
use super::collision;
use super::input_handler::InputMap;
use super::camera::{self, Camera};
use winit::event::VirtualKeyCode as Key;

pub struct Player {

    pub camera: super::camera::Camera,
    pub speed: f32,
    pub jump_force: f32,
    pub position: cgmath::Vector3<f32>,
    pub gravity_vel: cgmath::Vector3<f32>,
    pub gravity: f32,

    freecam_mode: bool,

}

impl Player {

    pub fn new(renderer: &super::Renderer) -> Player {
        Player {
            camera: Camera {
                position: (0.0, 0.0, 0.0).into(),
                yaw: 0.0,
                pitch: 0.0,
                speed: 10.0,
                sensitivity: 0.005,
                fovy: cgmath::Deg(90.0).into(),
                near: 0.1,
                far: 300.0,

                width: renderer.swap_chain_desc.width,
                height: renderer.swap_chain_desc.height,
                frustum: Frustum::new(),
            },
            speed: 0.03,
            jump_force: 0.07,
            position: cgmath::Vector3::new(0.0, 0.0, 0.0),
            gravity_vel: cgmath::Vector3::new(0.0, 0.0, 0.0),
            gravity: 0.002,

            freecam_mode: true,
        }
    }

    pub fn get_block_pos(&self, position: Vector3<f32>) -> Vector3<i32> {
        let p = position;
        Vector3::new(p.x.floor() as i32, p.y.floor() as i32, p.z.ceil() as i32)
    }

    pub fn update(&mut self, input: &mut InputMap, world: &World) {

        if input.get_key(Key::F).just_pressed {
            self.freecam_mode = !self.freecam_mode;
            println!("toggled freecam mode");
        }

        if input.get_key(Key::F3).just_pressed {
            println!("coordinates: {:?}", self.position);
        }

        let speed = self.speed;
        let sensitivity = self.camera.sensitivity;
        
        let (forward, right, up) = self.camera.get_headings();
        let mut forward_horizontal = forward - Vector3::new(0.0, forward.y, 0.0);

        if forward_horizontal.magnitude() != 0.0 { forward_horizontal = forward_horizontal.normalize(); }
        
        let x_input = -(input.get_key(Key::A).held as i32) + input.get_key(Key::D).held as i32;
        let z_input = -(input.get_key(Key::S).held as i32) + input.get_key(Key::W).held as i32;
        let mut xz_dir = (forward_horizontal * z_input as f32) + (right * x_input as f32);
        
        if xz_dir.magnitude() != 0.0 { xz_dir = xz_dir.normalize(); }
        let mut velocity = xz_dir * speed;
        
        let mut is_on_floor = false;

        if !self.freecam_mode {

            self.gravity_vel += Vector3::new(0.0, -self.gravity, 0.0);
    
            let x_collider = collision::CircleCollider::new(self.position + Vector3::new(velocity.x, 0.0, 0.0), 0.25);
            let z_collider = collision::CircleCollider::new(self.position + Vector3::new(0.0, 0.0, velocity.z), 0.25);
            let gravity_collider = collision::CircleCollider::new(self.position + self.gravity_vel, 0.25);
            for x in -2..2 {
                for z in -2..2 {
    
                    let gravity_block_position = self.get_block_pos(self.position + self.gravity_vel) + Vector3::new(x, 0, z);
                    let x_block_position = self.get_block_pos(self.position + Vector3::new(velocity.x, 0.0, 0.0)) + Vector3::new(x, 0, z);
                    let z_block_position = self.get_block_pos(self.position + Vector3::new(0.0, 0.0, velocity.z)) + Vector3::new(x, 0, z);
    
                    let gravity_block = world.get_block_at(gravity_block_position);
                    let x_block = world.get_block_at(x_block_position);
                    let z_block = world.get_block_at(z_block_position);
    
                    // y axis
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
                            velocity.x = 0.0;
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
                            velocity.z = 0.0;
                        }
                    }
                }
            }

            if is_on_floor && input.get_key(Key::Space).held {
                self.gravity_vel.y = self.jump_force;
            }
        }
        else {
            
            let speed = speed * 4.0;
            self.gravity_vel = Vector3::new(0.0, 0.0, 0.0);
            let fw = forward * speed * z_input as f32;
            let rt = right * speed * x_input as f32;
            velocity = fw + rt;

            if input.get_key(Key::Space).held {
                velocity += up * speed;
            }
            if input.get_key(Key::LShift).held {
                velocity -= up * speed;
            }
        }
   
        self.position += self.gravity_vel + velocity;
        self.camera.position = self.position + Vector3::new(0.0, 1.5, 0.0);

        let mouse_x = input.mouse.delta.x;
        let mouse_y = input.mouse.delta.y;

        self.camera.pitch -= mouse_y * sensitivity;
        self.camera.yaw += mouse_x * sensitivity;

        if self.camera.pitch < camera::PITCH_MIN { self.camera.pitch = camera::PITCH_MIN; }
        if self.camera.pitch > camera::PITCH_MAX { self.camera.pitch = camera::PITCH_MAX; }
        
        // don't forget to update the frustum planes!
        let planes = self.camera.frustum.get_planes(&self.camera);
        self.camera.frustum.planes = planes;

    }

}
