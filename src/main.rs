extern crate piston_window;
extern crate byteorder;
extern crate gif;

mod bitplanes;
mod enemy;
mod centered_canvas;
mod sprite;
mod util;
mod write_gif;

use bitplanes::*;
use enemy::DNA;
use sprite::Sprite;
use write_gif::write_sprite_to_gif;
use byteorder::{ByteOrder, LittleEndian};
use util::{bgr555_rgbf32, bgr555_rgb888};
use std::{thread, time};

use piston_window::*;

const ROM: &'static [u8] = include_bytes!("data/Super Metroid (Japan, USA) (En,Ja).sfc");

#[allow(unused)]
fn render_sprite_sheet(creature: DNA) {
    let gfx = creature.graphics();
    let rgb_palette: Vec<_> = creature.palette()
        .chunks(2)
        .map(|bgr| bgr555_rgbf32(LittleEndian::read_u16(bgr)))   
        .collect();

    let tiles: Vec<_> = Bitplanes::new(gfx).collect();

    let opengl = OpenGL::V3_2;
    let zoom = 2usize;
    let mut window: PistonWindow = WindowSettings::new(
        creature.name().unwrap_or("?".to_string()),
        [128 * zoom as u32, (tiles.len()* zoom / 2) as u32])
            .exit_on_esc(true)
            .opengl(opengl)
            .vsync(true)
            .build()
            .unwrap();

    while let Some(event) = window.next() {
        window.draw_2d(&event, |context, graphics| {
            clear([0.0; 4], graphics);

            for (i, tile) in tiles.iter().enumerate() {
                let (tile_x, tile_y) = (i % 16, i / 16);
                for (j, index) in tile.iter().enumerate() {
                    let (r, g, b) = rgb_palette[*index as usize];
                    let (x, y) = (tile_x * 8 + j % 8, tile_y * 8 + j / 8);
                    rectangle(
                        [r, g, b, 1.0],
                        [(x * zoom) as f64, (y * zoom) as f64, zoom as f64, zoom as f64],
                        context.transform,
                        graphics
                    );
                }
            }

        });
    }
}

#[allow(unused)]
fn render_animation(creature: DNA, num_frames: usize) {
    let gfx = creature.graphics();
    let rgb_palette: Vec<_> = creature.palette()
        .chunks(2)
        .map(|bgr| bgr555_rgbf32(LittleEndian::read_u16(bgr)))   
        .collect();

    let tiles: Vec<_> = Bitplanes::new(gfx).collect();
    let frames: Vec<_> = creature.frames(num_frames).iter().map(|f| f.composited(&tiles)).collect();
    let mut sprite = Sprite::new(frames);

    let opengl = OpenGL::V3_2;
    let zoom = 2usize;
    let mut window: PistonWindow =WindowSettings::new(
        creature.name().unwrap_or("?".to_string()),
        [sprite.width() as u32 * zoom as u32, sprite.height() as u32 * zoom as u32])
            .exit_on_esc(true)
            .opengl(opengl)
            .vsync(true)
            .build()
            .unwrap();
    while let Some(event) = window.next() {
        window.draw_2d(&event, |context, graphics| {
            clear([0.0; 4], graphics);

            let ref composite = sprite.frame();
            for (i, p) in composite.buffer.iter().enumerate() {
                if *p == 0 {
                    continue;
                }
                let (px, py) = (i % composite.width as usize, i / composite.width as usize);
                let (r, g, b) = rgb_palette[*p as usize];
                rectangle(
                    [r, g, b, 1.0],
                    [(px * zoom) as f64, (py * zoom) as f64, zoom as f64, zoom as f64],
                    context.transform,
                    graphics,
                )
            }
            let duration = time::Duration::from_millis(composite.duration as u64 * 16);
            thread::sleep(duration);
        });
    }
}

#[allow(unused)]
fn render_gif(creature: DNA, num_frames: usize) {
    let gfx = creature.graphics();
    let rgb_palette: Vec<_> = creature.palette()
        .chunks(2)
        .map(|bgr| bgr555_rgb888(LittleEndian::read_u16(bgr)))
        .collect();

    let tiles: Vec<_> = Bitplanes::new(gfx).collect();
    let frames: Vec<_> = creature.frames(num_frames).iter().map(|f| f.composited(&tiles)).collect();

    write_sprite_to_gif(&creature.name().unwrap_or("enemy".to_string()), &frames, &rgb_palette);
}

fn main() {
    let creature = DNA::read_from_rom(&ROM, 0xA0E63F);
    // let creature = DNA::read_from_rom(&ROM, 0xA0DD7F);
    // let creature = DNA::read_from_rom(&ROM, 0xA0EEBF);

    render_sprite_sheet(creature);
    // render_animation(creature, 6);
    // render_gif(creature, 6);
}
