#![feature(proc_macro)]
extern crate proc_samus;
extern crate lib_samus;
extern crate minifb;

use std::{time, thread};
use minifb::{Key, KeyRepeat, Scale, WindowOptions, Window};

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

const WIDTH: usize = 64;
const HEIGHT: usize = 64;

fn main() {
    let mut samus = StateMachine::new(0x00, poses::lookup);

    let mut window = Window::new("samus", WIDTH, HEIGHT, WindowOptions {
        scale: Scale::X4,
        ..Default::default()
    }).expect("couldn't create window!");

    let mut palette = [(0u32, 0u32, 0u32); 32];
    for (i, c) in (&palette::PALETTE).iter().enumerate() {
        palette[i] = (c.0 as u32, c.1 as u32, c.2 as u32);
    }

    let buffer: &mut [u32] = &mut [0; WIDTH * HEIGHT];
    let mut current_frame: &Frame = &Frame { buffer: &[], width: 0, height: 0, zero_x: 0, zero_y: 0 };
    let mut current_pose_id = 0;
    let mut current_pose_name: &str = "blank!";

    while window.is_open() && !window.is_key_down(Key::Escape) {
            let mut current_input = ControllerInput::empty();
            if window.is_key_pressed(Key::Right, KeyRepeat::No) {
                current_input |= ControllerInput::Right;
            }
            if window.is_key_pressed(Key::Left, KeyRepeat::No) {
                current_input |= ControllerInput::Left;
            }
            if window.is_key_pressed(Key::Up, KeyRepeat::No) {
                current_input |= ControllerInput::Up;
            }
            if window.is_key_pressed(Key::Down, KeyRepeat::No) {
                current_input |= ControllerInput::Down;
            }
            if window.is_key_pressed(Key::Space, KeyRepeat::No) {
                current_input |= ControllerInput::Jump;
            }
            if !current_input.is_empty() {
                samus.input(current_input);
            }
            if window.is_key_pressed(Key::F, KeyRepeat::No) {
                samus.fall();
            }
            if window.is_key_pressed(Key::L, KeyRepeat::No) {
                samus.land();
            }

            let (current_frame, duration) = samus.next();
            current_pose_id = samus.pose_state();
            current_pose_name = samus.pose_name();

            let d = time::Duration::from_millis(1000u64 / 30u64 * duration as u64 );

            let offset_x = WIDTH / 2 - current_frame.zero_x as usize;
            let offset_y = HEIGHT / 2 - current_frame.zero_y as usize;

            let top = offset_y;
            let bottom = offset_y + current_frame.height as usize;
            let left = offset_x;
            let right = offset_x + current_frame.width as usize;

            for (i, p) in buffer.iter_mut().enumerate() {
                let (cx, cy) = (i % WIDTH, i / WIDTH);
                if cy < top || cy >= bottom || cx < left || cx >= right {
                    *p = 0;
                    continue;
                }
                let j = (cy - top) * current_frame.width as usize + cx - left; // index in frame
                let palette_index = current_frame.buffer[j];
                *p = if palette_index == 0 {
                    0
                } else {
                    let pixel = palette[palette_index as usize];
                    0xFF << 24 | (pixel.0 << 16) | (pixel.1 << 8) | (pixel.2)
                };
            }

            window.update_with_buffer(buffer).unwrap();
            thread::sleep(d);
    }
}
