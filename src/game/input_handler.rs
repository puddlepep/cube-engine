use std::collections::HashMap;

use winit::event::ElementState;
use winit::event::VirtualKeyCode;

#[derive(Debug, Copy, Clone)]
pub struct State {
    pub held: bool,
    pub just_pressed: bool,
    pub just_released: bool,
}

impl State {
    pub fn new() -> State {
        State {
            held: false,
            just_pressed: false,
            just_released: false,
        }
    }
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
                lmb: State::new(),
                mmb: State::new(),
                rmb: State::new(),
            },
        }
    }

    pub fn update_key(&mut self, key: &VirtualKeyCode, pressed: bool) {

        match self.keys.get_mut(key) {

            // If the key does exist in the current input map, update it.
            Some(state) => {

                // Depending on the current and previous state of the key, determine whether or not
                // the key was 'just_pressed', and update the state accordingly.
                if pressed {
                    if state.held == false {
                        *state = State {
                            held: true,
                            just_pressed: true,
                            just_released: false,
                        }
                    }
                    else {
                        *state = State {
                            held: true,
                            just_pressed: false,
                            just_released: false,
                        }
                    }
                }
                else {

                    if state.held == true {
                        *state = State {
                            held: false,
                            just_pressed: false,
                            just_released: true,
                        }
                    }
                    else {
                        *state = State {
                            held: false,
                            just_pressed: false,
                            just_released: false,
                        }
                    }

                }
            },

            // If the key does not currently exist in the input map, add it to the list of tracked keys.
            None => {
                if pressed {
                    self.keys.insert(*key, State {
                        held: true,
                        just_pressed: true,
                        just_released: false,
                    });
                }
                else {
                    self.keys.insert(*key, State {
                        held: false,
                        just_pressed: false,
                        just_released: false,
                    });
                }
            }

        }
    }

    // Update the status of all keys, and if a new one is pressed, insert it into the input map.
    pub fn update(&mut self, event: &winit::event::DeviceEvent) {

        match event {

            winit::event::DeviceEvent::Key(input) => {

                match input.virtual_keycode {

                    Some(key) => {
                        self.update_key(&key, input.state == ElementState::Pressed);
                    },

                    None => ()

                }
            },

            winit::event::DeviceEvent::Button { button, state } => {
                match button {
                    1 => {
                        if state == &ElementState::Pressed {
                            if self.mouse.lmb.held == false {
                                self.mouse.lmb = State {
                                    held: true,
                                    just_pressed: true,
                                    just_released: false,
                                }
                            }
                            else {
                                self.mouse.lmb = State {
                                    held: true,
                                    just_pressed: false,
                                    just_released: false,
                                }
                            }
                        }
                        else {
                            if self.mouse.lmb.held == true {
                                self.mouse.lmb = State {
                                    held: false,
                                    just_pressed: false,
                                    just_released: true,
                                }
                            }
                            else {
                                self.mouse.lmb = State {
                                    held: false,
                                    just_pressed: false,
                                    just_released: false,
                                }
                            }
                        }
                    },

                    2 => {
                        if state == &ElementState::Pressed {
                            if self.mouse.mmb.held == false {
                                self.mouse.mmb = State {
                                    held: true,
                                    just_pressed: true,
                                    just_released: false,
                                }
                            }
                            else {
                                self.mouse.mmb = State {
                                    held: true,
                                    just_pressed: false,
                                    just_released: false,
                                }
                            }
                        }
                        else {
                            if self.mouse.mmb.held == true {
                                self.mouse.mmb = State {
                                    held: false,
                                    just_pressed: false,
                                    just_released: true,
                                }
                            }
                            else {
                                self.mouse.mmb = State {
                                    held: false,
                                    just_pressed: false,
                                    just_released: false,
                                }
                            }
                        }
                    },

                    3 => {
                        if state == &ElementState::Pressed {
                            if self.mouse.rmb.held == false {
                                self.mouse.rmb = State {
                                    held: true,
                                    just_pressed: true,
                                    just_released: false,
                                }
                            }
                            else {
                                self.mouse.rmb = State {
                                    held: true,
                                    just_pressed: false,
                                    just_released: false,
                                }
                            }
                        }
                        else {
                            if self.mouse.rmb.held == true {
                                self.mouse.rmb = State {
                                    held: false,
                                    just_pressed: false,
                                    just_released: true,
                                }
                            }
                            else {
                                self.mouse.rmb = State {
                                    held: false,
                                    just_pressed: false,
                                    just_released: false,
                                }
                            }
                        }
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

    pub fn get_key(&mut self, key: VirtualKeyCode) -> State {

        let state: &State;

        if self.keys.get(&key).is_some() { 
            state = self.keys.get(&key).unwrap();
        }
        else { 
            self.keys.insert(key, State::new());
            state = self.keys.get(&key).unwrap();
        }

        let result = state.clone();
        self.update_key(&key, result.held);
        result

    }

}