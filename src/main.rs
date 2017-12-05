extern crate piston_window;
extern crate byteorder;
extern crate gif;

mod bitplanes;
mod enemy;
mod centered_canvas;
mod sprite;
mod util;
mod write_gif;

mod snes;
mod samus;
mod frame_map;

use snes::{Rom, PcAddress, SnesAddress};
use enemy::DNA;
use sprite::{Sprite, SpriteView};
use write_gif::write_sprite_to_gif;
use byteorder::{ByteOrder, LittleEndian};
use util::{bgr555_rgbf32, zip3};
use std::{env, thread, time, process};

use frame_map::FrameMap;

use piston_window::*;

const ROM_DATA: &'static [u8] = include_bytes!("data/Super Metroid (Japan, USA) (En,Ja).sfc");
const ROM: Rom = Rom(ROM_DATA);

fn render_animation(sprite: Sprite) {
    let opengl = OpenGL::V3_2;
    let zoom = 2usize;
    let (window_width, window_height) = (128, 128);
    let mut window: PistonWindow =WindowSettings::new("samus",
        [128 * zoom as u32, 128 * zoom as u32])
            .exit_on_esc(true)
            .opengl(opengl)
            .vsync(true)
            .build()
            .unwrap();
    let palette = sprite.palettef32();
    let mut spriteview = SpriteView::new(&sprite);

    while let Some(event) = window.next() {
        window.draw_2d(&event, |context, graphics| {
            if let Some(_) = event.render_args() {
                clear([0.0; 4], graphics);

                let ref composite = spriteview.frame();
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
                        [(px * zoom) as f64, (py * zoom) as f64, zoom as f64, zoom as f64],
                        context.transform,
                        graphics,
                    )
                }
                let duration = time::Duration::from_millis(composite.duration as u64 * 16);
                thread::sleep(duration);
            }
        });
    }
}

fn render_tile_map(tiles: Vec<[u8; 64]>, palette: Vec<u16>) {
    let palette: Vec<_> = palette.iter().map(bgr555_rgbf32).collect();
    let opengl = OpenGL::V3_2;
    let zoom = 2usize;
    let mut window: PistonWindow = WindowSettings::new("creature",
        [128 * zoom as u32, (tiles.len()* zoom / 2) as u32])
            .exit_on_esc(true)
            .opengl(opengl)
            .vsync(true)
            .build()
            .unwrap();

    while let Some(event) = window.next() {
        window.draw_2d(&event, |context, graphics| {
            if let Some(_) = event.render_args() {
                clear([0.0; 4], graphics);

                for (i, tile) in tiles.iter().enumerate() {
                    let (tile_x, tile_y) = (i % 16, i / 16);
                    for (j, index) in tile.iter().enumerate() {
                        let (r, g, b) = palette[*index as usize];
                        let (x, y) = (tile_x * 8 + j % 8, tile_y * 8 + j / 8);
                        rectangle(
                            [r, g, b, 1.0],
                            [(x * zoom) as f64, (y * zoom) as f64, zoom as f64, zoom as f64],
                            context.transform,
                            graphics
                        );
                    }
                }
            }
        });
    }
}

struct Action {
    frames: usize,
    address: Option<u32>,
    subject: Option<Subject>,
    format: Format,
}

enum Subject {
    Enemy,
    Samus,
}

enum Format {
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
    "[-s | -a | -g] <addr>\n\
    -s = spritesheet, default\n\
    -a = animate\n\
    -g = gif\n\
    addr = SNES address in hex";

fn main() {
    use Subject::*;
    use Format::*;

    if env::args().count() <= 1 {
        eprintln!("Usage: {} {}", env::args().nth(0).unwrap_or("programname".to_string()), HELP_STRING);
        process::exit(1);
    }

    let action = env::args().skip(1)
        .fold(Action {
            format: Spritesheet,
            subject: None,
            address: None,
            frames: 1,
        }, |mut action, arg| {
            if arg.starts_with("-") {
                action.format = match arg.as_str() {
                    "-s" => Spritesheet,
                    "-a" => Animate,
                    "-g" => Gif,
                    s @ _ => {
                        eprintln!("Unknown flag {:?}. {}", s, FLAG_STRING);
                        process::exit(1)
                    },
                };
            } else if arg.starts_with(":") {
                let s: String = arg.chars().skip(1).collect();
                if let Ok(num_frames) = usize::from_str_radix(&s, 10) {
                    action.frames = num_frames;
                } else {
                    eprintln!("Couldn't parse number of frames {:?}.", s);
                    process::exit(1);
                }
            } else {
                if arg == "samus" {
                    action.subject = Some(Samus);
                } else if arg == "enemy" {
                    action.subject = Some(Enemy);
                } else if let Ok(addr) = u32::from_str_radix(&arg, 16) {
                    action.address = Some(addr);
                } else {
                    eprintln!("Couldn't parse address {:?} as hex. {}", arg, HINT_STRING);
                    process::exit(1)
                }
            }
            action
        });

    match (action.subject, action.address) {
        (Some(Samus), Some(addr)) => {
            let durations = samus::lookup_frame_durations(&ROM, addr as usize);
            let tile_maps = samus::tilemaps(&ROM, addr as usize, durations.len());
            let tile_sets = samus::graphics(&ROM, addr as usize, durations.len());
            let frames: Vec<_> = zip3(tile_maps, tile_sets, durations)
                .map(|(tm, ts, ds)| FrameMap::composite(&tm, &ts, *ds as u16)).collect();
            let p = PcAddress(0xD9400); // lol trolled, not a snes address. There goes 1 day... :/
            let palette: Vec<_> = ROM.read(p, 32)
                .chunks(2)
                .map(LittleEndian::read_u16)
                .collect();
            let sprite = Sprite::new(frames, palette);
            render_animation(sprite);
        },
        (Some(Enemy), Some(addr)) => {
            let addr = if addr == (addr & 0xFFFF) {
                0xA00000 | addr
            } else {
                addr
            };
            let creature = DNA::read_from_rom(&ROM, SnesAddress(addr));
            let palette: Vec<_> = creature.palette().chunks(2)
                .map(LittleEndian::read_u16).collect();
            let tiles = creature.graphics();

            match action.format {
                Spritesheet => {
                    render_tile_map(tiles, palette);
                },
                Animate => {
                    let frames: Vec<_> = creature.frames().iter().map(|f| f.composited(&tiles)).collect();
                    let sprite = Sprite::new(frames, palette);
                    render_animation(sprite);
                },
                Gif => {
                    let frames: Vec<_> = creature.frames().iter().map(|f| f.composited(&tiles)).collect();
                    let sprite = Sprite::new(frames, palette);
                    write_sprite_to_gif(
                        &creature.name().unwrap_or("enemy".to_string()),
                        sprite.frames(),
                        &sprite.palette888()
                    ).expect("YOUR GIF DIED MISSION FAILED");
                },
            }
        },
        _ => {
            eprintln!("Required subject and/or SNES address missing");
            process::exit(1);
        },
    };
}
