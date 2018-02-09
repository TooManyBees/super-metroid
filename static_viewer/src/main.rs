#![feature(proc_macro)]
extern crate proc_samus;
extern crate piston_window;
extern crate sm;

use std::{thread, time};
use piston_window::*;
use sm::pose::*;

proc_samus::samus_poses!([0x0B]);

proc_samus::samus_palettes!();

fn main() {
    let mut pose = poses::lookup(0x0B).clone();

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

    while let Some(event) = window.next() {
        window.draw_2d(&event, |context, graphics| {
            if let Some(_) = event.render_args() {
                clear([0.0; 4], graphics);

                let (composite, duration) = if let Next::Frame(f, d) = pose.next() {
                    (f, d)
                } else {
                    unreachable!()
                };

                let offset_x = window_width / 2 - composite.zero_x as usize;
                let offset_y = window_height / 2 - composite.zero_y as usize;

                for (i, p) in composite.buffer.iter().enumerate() {
                    if *p == 0 {
                        continue;
                    }
                    let (px, py) = (offset_x + i % composite.width as usize, offset_y + i / composite.width as usize);
                    let (r, g, b) = palette[*p as usize];
                    rectangle(
                        [r, g, b, 1.0],
                        [(px*zoom) as f64, (py*zoom) as f64, zoom as f64, zoom as f64],
                        context.transform,
                        graphics,
                    )
                }
                let duration = time::Duration::from_millis(duration as u64 * 16);
                thread::sleep(duration);
            }
        });
    }
}
