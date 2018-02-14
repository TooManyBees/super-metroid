#![feature(proc_macro)]
extern crate proc_samus;
extern crate lib_samus;
extern crate piston_window;

use std::{time};
use piston_window::*;
use lib_samus::pose::*;

use lib_samus::StateMachine;

// proc_samus::samus_poses!([0x00, 0x01, 0x02, 0x09, 0x0A, 0x0B]);
proc_samus::samus_poses!([]);

proc_samus::samus_palettes!();

fn main() {
    let mut samus = StateMachine::new(0x0B, poses::lookup);

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

    while let Some(event) = window.next() {
        if let Some(b) = event.press_args() {
            match b {
                Button::Keyboard(Key::Right) => {
                    let next_pose = samus.pose_state() + 1;
                    samus.goto(next_pose);
                },
                Button::Keyboard(Key::Left) => {
                    let state = samus.pose_state();
                    let next_pose = if state == 0 {
                        0
                    } else {
                        state - 1
                    };
                    samus.goto(next_pose);
                },
                _ => {},
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
