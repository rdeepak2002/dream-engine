use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;
// use winit::event::VirtualKeyCode;
use winit::keyboard::Key;

use dream_math::Vector2;

static INPUT: Lazy<Arc<RwLock<Input>>> = Lazy::new(|| Arc::new(RwLock::new(Input::default())));

pub fn set_keyboard_state(key_code: Key, state: f32) {
    let i = INPUT.clone();
    let input = i.write();
    input.unwrap().key_states.insert(key_code, state);
}

pub fn get_keyboard_state(key_code: Key) -> f32 {
    let i = INPUT.clone();
    let input = i.read();
    let res = *input
        .unwrap()
        .key_states
        .get(&key_code)
        .unwrap_or(&(0.0f32));
    res
}

pub fn set_mouse_left_pressed(is_pressed: bool) {
    let i = INPUT.clone();
    let input = i.write();
    input.unwrap().mouse_left_is_pressed = is_pressed;
}

pub fn is_mouse_left_pressed() -> bool {
    let i = INPUT.clone();
    let input = i.read();
    let res = input.unwrap().mouse_left_is_pressed;
    res
}

pub fn set_mouse_right_pressed(is_pressed: bool) {
    let i = INPUT.clone();
    let input = i.write();
    input.unwrap().mouse_right_is_pressed = is_pressed;
}

pub fn is_mouse_right_pressed() -> bool {
    let i = INPUT.clone();
    let input = i.read();
    let res = input.unwrap().mouse_right_is_pressed;
    res
}

pub fn set_mouse_move(dx_dy: Vector2<f32>) {
    let i = INPUT.clone();
    let input = i.write();
    input.unwrap().mouse_move = dx_dy;
}

pub fn get_mouse_move() -> Vector2<f32> {
    let i = INPUT.clone();
    let input = i.read();
    let res = input.unwrap().mouse_move;
    res
}

pub fn set_mouse_scroll(dm: f32) {
    let i = INPUT.clone();
    let input = i.write();
    input.unwrap().mouse_scroll = dm;
}

pub fn get_mouse_scroll() -> f32 {
    let i = INPUT.clone();
    let input = i.read();
    let res = input.unwrap().mouse_scroll;
    res
}

pub fn set_renderer_panel_active(is_active: bool) {
    let i = INPUT.clone();
    let input = i.write();
    input.unwrap().renderer_panel_active = is_active;
}

pub fn is_renderer_panel_active() -> bool {
    let i = INPUT.clone();
    let input = i.read();
    let res = input.unwrap().renderer_panel_active;
    res
}

#[derive(Default, Debug)]
struct Input {
    key_states: HashMap<Key, f32>,
    mouse_left_is_pressed: bool,
    mouse_right_is_pressed: bool,
    mouse_move: Vector2<f32>,
    mouse_scroll: f32,
    renderer_panel_active: bool,
}
