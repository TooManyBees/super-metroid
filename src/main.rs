extern crate piston_window;
extern crate byteorder;
extern crate gif;

mod bitplanes;
mod enemy;
mod centered_canvas;
mod sprite;
mod util;
mod write_gif;

mod samus;
mod frame_map;

use enemy::DNA;
use sprite::Sprite;
use write_gif::write_sprite_to_gif;
use byteorder::{ByteOrder, LittleEndian};
use util::{bgr555_rgbf32, bgr555_rgb888, print_hex, zip3};
use std::{env, thread, time, process};

use frame_map::FrameMap;

use piston_window::*;

const ROM: &'static [u8] = include_bytes!("data/Super Metroid (Japan, USA) (En,Ja).sfc");

// fn find_in_rom(slice: &[u8]) -> Option<usize> {
//     let len = slice.len();
//     (0..ROM.len()).find(|i| &ROM[*i..*i+len] == slice)
// }

fn render_samus(mut sprite: Sprite) {
    let p = 0xD9400; // lol trolled, not a snes address. There goes 1 day... :/
    let rgb_palette: Vec<_> = ROM[p..p+32]
        .chunks(2)
        .map(|bgr| bgr555_rgbf32(LittleEndian::read_u16(bgr)))
        .collect();

    let opengl = OpenGL::V3_2;
    let zoom = 2usize;
    let mut window: PistonWindow =WindowSettings::new("samus",
        [sprite.width() as u32 * zoom as u32, sprite.height() as u32 * zoom as u32])
            .exit_on_esc(true)
            .opengl(opengl)
            .vsync(true)
            .build()
            .unwrap();
    while let Some(event) = window.next() {
        window.draw_2d(&event, |context, graphics| {
            if let Some(_) = event.render_args() {
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
                let duration = time::Duration::from_millis(composite.duration as u64);
                thread::sleep(duration);
            }
        });
    }
}

fn render_sprite_sheet(creature: DNA) {
    let rgb_palette: Vec<_> = creature.palette()
        .chunks(2)
        .map(|bgr| bgr555_rgbf32(LittleEndian::read_u16(bgr)))   
        .collect();

    let tiles = creature.graphics();

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
            if let Some(_) = event.render_args() {
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
            }
        });
    }
}

fn render_animation(creature: DNA, num_frames: usize) {
    let rgb_palette: Vec<_> = creature.palette()
        .chunks(2)
        .map(|bgr| bgr555_rgbf32(LittleEndian::read_u16(bgr)))   
        .collect();

    let tiles = creature.graphics();
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
            if let Some(_) = event.render_args() {
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
                let duration = time::Duration::from_millis(composite.duration as u64);
                thread::sleep(duration);
            }
        });
    }
}

#[allow(unused)]
fn render_gif(creature: DNA, num_frames: usize) {
    let rgb_palette: Vec<_> = creature.palette()
        .chunks(2)
        .map(|bgr| bgr555_rgb888(LittleEndian::read_u16(bgr)))
        .collect();

    let tiles = creature.graphics();
    let frames: Vec<_> = creature.frames(num_frames).iter().map(|f| f.composited(&tiles)).collect();

    write_sprite_to_gif(&creature.name().unwrap_or("enemy".to_string()), &frames, &rgb_palette);
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
    "[-s | -a | -g] [:n] <addr>\n\
    -s = spritesheet, default\n\
    -a = animate\n\
    -g = gif\n\
    n = number of frames (for animate / gif modes)\n\
    addr = SNES address in hex";

fn main() {
    use Subject::*;
    use Format::*;

    // let pose = (0x49, 6);

    // let durations = samus::lookup_frame_durations(&ROM, pose.0, pose.1);

    // let tile_sets = samus::graphics(&ROM, pose.0, pose.1);
    // let frames: Vec<_> = zip3(samus::tilemaps(&ROM, pose.0, pose.1), tile_sets, durations)
    //     .map(|(fs, ts, d)| FrameMap::composite(&fs, &ts, *d as u16)).collect();
    // let mut sprite = Sprite::new(frames);

    // use util::snespc2;

    // let p = 0xD9400; // lol trolled, not a snes address. There goes 1 day... :/
    // let rgb_palette: Vec<_> = ROM[p..p+32]
    //     .chunks(2)
    //     .map(|bgr| bgr555_rgbf32(LittleEndian::read_u16(bgr)))
    //     .collect();

    // let opengl = OpenGL::V3_2;
    // let zoom = 4usize;
    // let mut window: PistonWindow = WindowSettings::new("hi there",
    //     // [128 * zoom as u32, 128 * zoom as u32])
    //     [sprite.width() as u32 * zoom as u32, sprite.height() as u32 * zoom as u32])
    //         .exit_on_esc(true)
    //         .opengl(opengl)
    //         .vsync(true)
    //         .build()
    //         .unwrap();

    // while let Some(event) = window.next() {
    //     window.draw_2d(&event, |context, graphics| {
    //         if let Some(_) = event.render_args() {
    //             clear([0.0; 4], graphics);

    //             // for (i, tile) in tiles.iter().enumerate() {
    //             //     let (tile_x, tile_y) = (i % 16, i / 16);
    //             //     for (j, index) in tile.iter().enumerate() {
    //             //         let (r, g, b) = rgb_palette[*index as usize];
    //             //         let (x, y) = (tile_x * 8 + j % 8, tile_y * 8 + j / 8);
    //             //         rectangle(
    //             //             [r, g, b, 1.0],
    //             //             [(x * zoom) as f64, (y * zoom) as f64, zoom as f64, zoom as f64],
    //             //             context.transform,
    //             //             graphics
    //             //         );
    //             //     }
    //             // }

    //             let composite = sprite.frame();
    //             for (i, p) in composite.buffer.iter().enumerate() {
    //                 if *p == 0 {
    //                     continue;
    //                 }
    //                 let (px, py) = (i % composite.width as usize, i / composite.width as usize);
    //                 let (r, g, b) = rgb_palette[*p as usize];
    //                 rectangle(
    //                     [r, g, b, 1.0],
    //                     [(px * zoom) as f64, (py * zoom) as f64, zoom as f64, zoom as f64],
    //                     context.transform,
    //                     graphics,
    //                 )
    //             }
    //             let duration = time::Duration::from_millis(composite.duration as u64);
    //             thread::sleep(duration);
    //         }
    //     });
    // }



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
            let durations = samus::lookup_frame_durations(&ROM, addr as usize, action.frames);
            let tile_sets = samus::graphics(&ROM, addr as usize, action.frames);
            let frames: Vec<_> = zip3(samus::tilemaps(&ROM, addr as usize, action.frames), tile_sets, durations)
                .map(|(fs, ts, d)| FrameMap::composite(&fs, &ts, *d as u16)).collect();
            let sprite = Sprite::new(frames);
            render_samus(sprite);
        },
        (Some(Enemy), Some(addr)) => {
            let creature = DNA::read_from_rom(&ROM, addr);
            match action.format {
                Spritesheet => render_sprite_sheet(creature),
                Animate => render_animation(creature, action.frames),
                Gif => render_gif(creature, action.frames),
            }
        },
        _ => {
            eprintln!("Required subject and/or SNES address missing");
            process::exit(1);
        },
    };
}
