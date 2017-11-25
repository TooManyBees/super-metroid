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
use std::{env, thread, time, process};

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

enum Action {
    Spritesheet,
    Animate,
    Gif,
}

static HINT_STRING: &'static str =
    "Try:\n\
    A0E63F (evir)\n\
    A0DD7F (metroid)\n\
    A0EEBF (super metroid)";

static FLAG_STRING: &'static str =
    "Try:\n\
    -s (spritesheet)\n\
    -a (animate)\n\
    -g (gif)";

static HELP_STRING: &'static str =
    "[-s | -a | -g] [@n] <addr>\n\
    -s = spritesheet, default\n\
    -a = animate\n\
    -g = gif\n\
    n = number of frames (for animate / gif modes)\n\
    addr = SNES address in hex";

fn main() {
    use Action::*;

    if env::args().count() <= 1 {
        eprintln!("Usage: {} {}", env::args().nth(0).unwrap_or("programname".to_string()), HELP_STRING);
        process::exit(1);
    }

    let (action, addr): (Action, Option<u32>) = env::args().skip(1)
        .fold((Spritesheet, None), |acc, arg| {
            if arg.starts_with("-") {
                match arg.as_str() {
                    "-s" => (Spritesheet, acc.1),
                    "-a" => (Animate, acc.1),
                    "-g" => (Gif, acc.1),
                    s @ _ => {
                        eprintln!("Unknown flag {:?}. {}", s, FLAG_STRING);
                        process::exit(1)
                    },
                }
            } else {
                if let Ok(addr) = u32::from_str_radix(&arg, 16) {
                    (acc.0, Some(addr))
                } else {
                    eprintln!("Couldn't parse address {:?} as hex. {}", arg, HINT_STRING);
                    process::exit(1)
                }
            }
        });

    if let Some(addr) = addr {
        let creature = DNA::read_from_rom(&ROM, addr);
        match action {
            Spritesheet => render_sprite_sheet(creature),
            Animate => render_animation(creature, 6),
            Gif => render_gif(creature, 6),
        }
    } else {
        eprintln!("Required SNES address missing.");
        process::exit(1);
    }
}
