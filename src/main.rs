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
use util::bgr555_rgbf32;

use piston_window::*;

const ROM: &'static [u8] = include_bytes!("data/Super Metroid (Japan, USA) (En,Ja).sfc");

fn main() {
    let creature = DNA::read_from_rom(&ROM, 0xA0E63F);
    let gfx = creature.graphics();

    let rgb_palette: Vec<_> = creature.palette()
        .chunks(2)
        .map(|bgr| bgr555_rgbf32(LittleEndian::read_u16(bgr)))
        .collect();

    let tiles: Vec<_> = Bitplanes::new(gfx).collect();
    let frames: Vec<_> = creature.frames(6).iter().map(|f| f.composited(&tiles)).collect();
    let mut sprite = Sprite::new(frames);

    // write_sprite_to_gif(format!("{}.gif", creature.name()), &frames, &rgb_palette);

    let opengl = OpenGL::V3_2;
    let zoom = 2usize;
    let mut window: PistonWindow =
        WindowSettings::new(creature.name(), [sprite.width() as u32 * zoom as u32, sprite.height() as u32 * zoom as u32])
        // WindowSettings::new(creature.name(), [128 * zoom as u32, (tiles.len()* zoom / 2) as u32])
            .exit_on_esc(true)
            .opengl(opengl)
            .vsync(true)
            .build()
            .unwrap();
    while let Some(event) = window.next() {
        window.draw_2d(&event, |context, graphics| {
            // clear([background.0, background.1, background.2, 1.0], graphics);
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

            // for (i, tile) in tiles.iter().enumerate() {
            //     let (tile_x, tile_y) = (i % 16, i / 16);
            //     for (j, index) in tile.iter().enumerate() {
            //         let (r, g, b) = rgb_palette[*index as usize];
            //         let (x, y) = (tile_x * 8 + j % 8, tile_y * 8 + j / 8);
            //         rectangle(
            //             [r, g, b, 1.0],
            //             [(x * zoom) as f64, (y * zoom) as f64, zoom as f64, zoom as f64],
            //             context.transform,
            //             graphics
            //         );
            //     }
            // }

        });
    }
}
