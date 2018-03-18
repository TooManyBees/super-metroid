#![feature(proc_macro)]
extern crate proc_samus;
extern crate lib_samus;

use std::{mem, slice};
use std::os::raw::c_void;
use std::boxed::Box;
use lib_samus::pose::*;
use lib_samus::StateMachine;

proc_samus::samus_poses!([
    0x00, // elevator pose
    0x01, // facing_right_normal
    0x02, // facing_left_normal
    0x09, // moving_right_not_aiming
    0x0A, // moving_left_not_aiming
    0x0B, // moving_right_gun_extended_not_aiming
    0x0C, // moving_left_gun_extended_not_aiming
    0x13, // jump_facing_right_gun_extended_not_aiming_or_moving
    0x14, // jump_facing_left_gun_extended_not_aiming_or_moving
    0x19, // spin_jump_right
    0x1A, // spin_jump_left
    0x1B, // space_jump_right
    0x1C, // space_jump_left
    0x25, // standing_turn_right_to_left
    0x26, // standing_turn_left_to_right
    0x27, // crouching_right
    0x28, // crouching_left
    0x29, // falling_right_normal
    0x2A, // falling_left_normal
    0x2F, // jumping_turn_right_to_left
    0x30, // jumping_turn_left_to_right
    0x35, // crouch_transition_facing_right
    0x36, // crouch_transition_facing_left
    0x3B, // standing_from_crouch_facing_right
    0x3C, // standing_from_crouch_facing_left
    0x43, // crouching_turn_right_to_left
    0x44, // crouching_turn_left_to_right
    0x49, // moonwalk_right_facing_left
    0x4A, // moonwalk_left_facing_right
    0x4B, // jump_transition_stand_crouch_facing_right
    0x4C, // jump_transition_stand_crouch_facing_left
    0x4D, // jump_facing_right_gun_not_extended_not_aiming_or_moving
    0x4E, // jump_facing_left_gun_not_extended_not_aiming_or_moving
    0x51, // jump_forward_facing_right_gun_extended
    0x52, // jump_forward_facing_left_gun_extended
    0x67, // falling_facing_right_fired_a_shot
    0x68, // falling_facing_left_fired_a_shot
    0x81, // screw_attack_right
    0x82, // screw_attack_left
    0x87, // falling_turn_right_to_left
    0x88, // falling_turn_left_to_right
    0xA4, // landing_facing_right
    0xA5, // landing_facing_left
    0xA6, // landing_spinjump_facing_right
    0xA7, // landing_spinjump_facing_left
    0xE6, // landing_facing_right_firing
    0xE7, // landing_facing_left_firing
]);

proc_samus::samus_palettes!();

#[no_mangle]
pub extern fn init() -> *const StateMachine<'static> {
    let state = Box::new(StateMachine::new(0x00, poses::lookup));
    Box::into_raw(state)
}

#[no_mangle]
pub extern fn allocate(len: usize) -> *mut c_void {
    let mut buf = Vec::with_capacity(len);
    let ptr = buf.as_mut_ptr();
    mem::forget(buf);
    ptr as *mut c_void
}

fn decode_input(key: u8) -> Option<ControllerInput> {
    match key {
        32 => Some(ControllerInput::Jump), // space
        17 => Some(ControllerInput::Shoot), // control
        37 => Some(ControllerInput::Left), // left arrow
        38 => Some(ControllerInput::Up), // up arrow
        39 => Some(ControllerInput::Right), // right arrow
        40 => Some(ControllerInput::Down), // down arrow
        _ => None,
    }
}

#[no_mangle]
pub extern fn input(state_ptr: *mut StateMachine<'static>, key: u8) -> bool {
    let mut state = unsafe { Box::from_raw(state_ptr) };
    let ret = if let Some(i) = decode_input(key) {
        state.input(i)
    } else {
        false
    };
    mem::forget(state);
    ret
}

#[no_mangle]
pub extern fn input_end(state_ptr: *mut StateMachine<'static>, key: u8) -> bool {
    let mut state = unsafe { Box::from_raw(state_ptr) };
    let ret = if let Some(i) = decode_input(key).map(|i| state.current_input() - i) {
        state.input(i)
    } else {
        false
    };
    mem::forget(state);
    ret
}

#[no_mangle]
pub extern fn fall(state_ptr: *mut StateMachine<'static>) -> bool {
    let mut state = unsafe { Box::from_raw(state_ptr) };
    let ret = state.fall();
    mem::forget(state);
    ret
}

#[no_mangle]
pub extern fn land(state_ptr: *mut StateMachine<'static>) -> bool {
    let mut state = unsafe { Box::from_raw(state_ptr) };
    let ret = state.land();
    mem::forget(state);
    ret
}

#[no_mangle]
pub extern fn next_frame(state_ptr: *mut StateMachine<'static>, pointer: *mut u8, width: usize, height: usize) -> f64 {
    let mut state = unsafe { Box::from_raw(state_ptr) };
    let buffer = unsafe { slice::from_raw_parts_mut(pointer, width * height * 4) };

    let (current_frame, duration) = state.next();

    let offset_x = width / 2 - current_frame.zero_x as usize;
    let offset_y = height / 2 - current_frame.zero_y as usize;

    let top = offset_y;
    let bottom = offset_y + current_frame.height as usize;
    let left = offset_x;
    let right = offset_x + current_frame.width as usize;

    for (i, pixel) in buffer.chunks_mut(4).enumerate() {
        let (cx, cy) = (i % width, i / height);
        if cy < top || cy >= bottom || cx < left || cx >= right {
            pixel[0] = 0;
            pixel[1] = 0;
            pixel[2] = 0;
            pixel[3] = 0;
            continue;
        }
        let j = (cy - top) * current_frame.width as usize + cx - left; // index in frame
        let palette_index = current_frame.buffer[j];
        if palette_index == 0 {
            pixel[0] = 0;
            pixel[1] = 0;
            pixel[2] = 0;
            pixel[3] = 0;
        } else {
            let color = &palette::PALETTE[palette_index as usize];
            pixel[0] = color.0;
            pixel[1] = color.1;
            pixel[2] = color.2;
            pixel[3] = 0xFF;
        };
    }

    mem::forget(state);
    (1000u64 / 30u64 * duration as u64) as f64
}
