use std::collections::HashMap;

use winit::event::ElementState;
use winit::event::VirtualKeyCode;

pub struct State {
    pub held: bool,
}

pub struct Mouse {
    pub delta: cgmath::Vector2<f32>,
    pub wheel_delta: f32,
    pub lmb: State,
    pub mmb: State,
    pub rmb: State,
}

pub struct InputMap {
    keys: HashMap<VirtualKeyCode, State>,
    pub mouse: Mouse,
}

impl InputMap {

    pub fn new() -> InputMap {
        InputMap {
            keys: HashMap::new(),
            mouse: Mouse { 
                delta: cgmath::Vector2::new(0.0, 0.0),
                wheel_delta: 0.0,
                lmb: State { held: false }, 
                mmb: State { held: false }, 
                rmb: State { held: false },
            },
        }
    }

    // Update the status of all keys, and if a new one is pressed, insert it into the input map.
    pub fn update(&mut self, event: &winit::event::DeviceEvent) {

        match event {

            winit::event::DeviceEvent::Key(input) => {

                match input.virtual_keycode {

                    Some(code) => {

                        match self.keys.get_mut(&code) {

                            Some(state) => {
                                state.held = input.state == ElementState::Pressed;
                            },

                            None => {
                                self.keys.insert(code, State { held: input.state == ElementState::Pressed });
                            }

                        }

                    },

                    None => ()

                }
            },

            winit::event::DeviceEvent::Button { button, state } => {
                match button {
                    1 => {
                        self.mouse.lmb.held = state == &ElementState::Pressed;
                    },

                    2 => {
                        self.mouse.mmb.held = state == &ElementState::Pressed;
                    },

                    3 => {
                        self.mouse.rmb.held = state == &ElementState::Pressed;
                    },

                    _ => ()
                }
            },

            winit::event::DeviceEvent::MouseWheel { delta } => {
                
                match delta {
                    winit::event::MouseScrollDelta::LineDelta(.., y) => {
                        self.mouse.wheel_delta = *y;
                    }

                    _ => ()
                }
            },

            winit::event::DeviceEvent::MouseMotion { delta } => {
                self.mouse.delta = cgmath::Vector2::new(delta.0 as f32, delta.1 as f32);
            },

            _ => ()

        }
    }

    pub fn post_update(&mut self) {
        self.mouse.delta = cgmath::Vector2::new(0.0, 0.0);
        self.mouse.wheel_delta = 0.0;
    }

    pub fn get_key(&mut self, key: VirtualKeyCode) -> &State {

        if self.keys.get(&key).is_some() { 
            self.keys.get(&key).unwrap()
        }
        else { 
            self.keys.insert(key, State{ held: false });
            self.keys.get(&key).unwrap()
        }

    }

}