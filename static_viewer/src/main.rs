#![feature(proc_macro)]
extern crate proc_samus;
extern crate lib_samus;
extern crate piston_window;

use std::{time};
use piston_window::*;

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
]);

proc_samus::samus_palettes!();

fn main() {
    let mut samus = StateMachine::new(0x00, poses::lookup);

    let opengl = OpenGL::V3_2;
    let zoom = 4usize;
    let (window_width, window_height) = (64, 64);

    let mut window: PistonWindow = WindowSettings::new("samus",
        [(window_width * zoom) as u32, (window_height * zoom) as u32])
            .exit_on_esc(true)
            .opengl(opengl)
            .vsync(true)
            .build()
            .unwrap();

    let mut palette = [(0f32, 0f32, 0f32); 32];
    for (i, c) in (&palette::PALETTE).iter().enumerate() {
        palette[i] = (c.0 as f32 / 255.0, c.1 as f32 / 255.0, c.2 as f32 / 255.0);
    }

    let factory = window.factory.clone();
    let mut glyphs = Glyphs::new("../data/cour.ttf", factory, TextureSettings::new()).expect("font failed");

    let mut next_frame_time = time::Instant::now();
    let mut current_frame: &Frame = &Frame { buffer: &[], width: 0, height: 0, zero_x: 0, zero_y: 0 };
    let mut current_pose_id = 0;
    let mut current_pose_name: &str = "blank!";
    let mut current_input = ControllerInput::empty();

    while let Some(event) = window.next() {
        if let Some(b) = event.press_args() {
            let input = match b {
                Button::Keyboard(Key::Right) => {
                    Some((samus.current_input() - ControllerInput::Left) | ControllerInput::Right)
                },
                Button::Keyboard(Key::Left) => {
                    Some((samus.current_input() - ControllerInput::Right) | ControllerInput::Left)
                },
                _ => None,
            };
            if let Some(input) = input {
                current_input = input;
                println!("{:?}", current_input);
                if samus.input(input) {
                    next_frame_time = time::Instant::now();
                }
            }
        }

        if let Some(_) = event.update_args() {
            let now = time::Instant::now();
            if now >= next_frame_time {

                let (composite, duration) = samus.next();
                current_frame = composite;
                current_pose_id = samus.pose_state();
                current_pose_name = samus.pose_name();

                let d = time::Duration::from_millis(1000u64 / 30u64 * duration as u64 );
                next_frame_time = now + d;
            }
        }

        window.draw_2d(&event, |context, graphics| {
            clear([0.0; 4], graphics);

            let offset_x = window_width / 2 - current_frame.zero_x as usize;
            let offset_y = window_height / 2 - current_frame.zero_y as usize;

            for (i, p) in current_frame.buffer.iter().enumerate() {
                if *p == 0 {
                    continue;
                }
                let (px, py) = (offset_x + i % current_frame.width as usize, offset_y + i / current_frame.width as usize);
                let (r, g, b) = palette[*p as usize];
                rectangle(
                    [r, g, b, 1.0],
                    [(px*zoom) as f64, (py*zoom) as f64, zoom as f64, zoom as f64],
                    context.transform,
                    graphics,
                )
            }
            Text::new_color([1.0, 1.0, 1.0, 1.0], 10).draw(
                &format!("{:02X} {}", current_pose_id, current_pose_name),
                &mut glyphs,
                &context.draw_state,
                context.transform.trans(0.0, 12.0),
                graphics,
            ).expect("Couldn't draw pose name");
        });
    }
}
