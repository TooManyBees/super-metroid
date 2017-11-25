extern crate piston_window;
extern crate byteorder;

mod bitplanes;
use bitplanes::*;

mod centered_canvas;
use centered_canvas::CenteredCanvas;

use piston_window::*;
use byteorder::{ByteOrder, LittleEndian};
// use std::fs::File;
// use std::io::BufReader;
// use std::io::prelude::*;

const ROM: &'static [u8] = include_bytes!("data/Super Metroid (Japan, USA) (En,Ja).sfc");

// const SNES_HEADER: bool = false;

// fn snespc(addrlo: usize, addrhi: usize, bank: usize) -> usize {
//     (addrlo & 255) + ((addrhi & 255) << 8) + ((bank & 127) << 15)
//       - (if SNES_HEADER {0} else {512}) - 32256
// }

// fn print_hex(arr: &[u8]) {
//     print!("[");
//     for byte in arr.iter().take(arr.len() - 1) {
//         print!("{:02X} ", byte);

//     }
//     print!("{:02X}", arr[arr.len() - 1]);
//     println!("]");
// }

// https://www.smwcentral.net/?p=viewthread&t=13167

#[inline(always)]
fn snespc(bank: u8, addr: u16) -> usize {
    (((bank & 127) as usize) << 15) + (addr as usize) - 512 - 32256
}

#[inline(always)]
fn snespc2(addr: u32) -> usize {
    (((addr & 0x7F0000) >> 1) + (addr & 0xFFFF)) as usize - 512 - 32256
}

struct FrameIndex {
    duration: u16,
    snes_addr: u16,
}

impl std::fmt::Debug for FrameIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,
            "FrameIndex {{ duration: {:02X}, snes_addr: {:02X} }}",
            self.duration, self.snes_addr
        )
    }
}

struct FramePart {
    xx: i8,
    priority_b: u8,
    yy: i8,
    tl: u8,
    priority_a: u8,
}

impl FramePart {
    #[inline(always)]
    fn is_double(&self) -> bool {
        (self.priority_a & 0b10000000) > 0
    }

    // fn x_offset(&self) -> i16 {
    //     // let add_FF = (self.priority_a & 0b01) > 0;
    //     if self.priority_a & 0b01 > 0 {
    //         self.xx as i16 + 0xFF
    //     } else {
    //         self.xx as i16
    //     }
    // }

    // #[inline(always)]
    // fn flip_vertical(&self) -> bool {
    //     (self.priority_b & 0b01000000) > 0
    // }

    // #[inline(always)]
    // fn flip_horizontal(&self) -> bool {
    //     (self.priority_b & 0b00100000) > 0
    // }
}

impl std::fmt::Debug for FramePart {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,
            "FramePart {{ xx: {:02X}, priority_a: {:08b}, yy: {:02X}, tl: {:02X}, priority_b: {:08b} }}",
            self.xx, self.priority_a, self.yy, self.tl, self.priority_b
        )
    }
}

struct Frame {
    parts: Vec<FramePart>,
    duration: u16,
}

impl std::fmt::Debug for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,
            "Frame {{ duration: {:02X}, parts: {:?} }}",
            self.duration, self.parts
        )
    }
}

impl Frame {
    fn dimensions(&self) -> (u16, u16, u16, u16) {
        let mut top = 0i8;
        let mut bottom = 0i8;
        let mut left = 0i8;
        let mut right = 0i8;
        for part in self.parts.iter() {
            let size = if part.is_double() { 16 } else { 8 };
            if part.xx < left { left = part.xx };
            if part.xx + size > right { right = part.xx + size };
            if part.yy < top { top = part.yy };
            if part.yy + size > bottom { bottom = part.yy + size }
        }
        (-left as u16, -top as u16, (right - left) as u16, (bottom - top) as u16)
    }

    fn composited(&self, tiles: &[Vec<u8>]) -> CompositedFrame {
        let (zx, zy, width, height) = self.dimensions();

        let mut canvas = CenteredCanvas::new(width, height, (zx, zy));

        for part in self.parts.iter().rev() {
            if part.is_double() {
                let n = part.tl as usize;
                let tile0 = &tiles[n];
                let tile1 = &tiles[n+1];
                let tile2 = &tiles[n + 16];
                let tile3 = &tiles[n + 17];
                canvas.paint_block(tile0, tile1, tile2, tile3, part.xx as i16, part.yy as i16);
            } else {
                let tile = &tiles[part.tl as usize];
                canvas.paint_tile(tile, part.xx as i16, part.yy as i16);
            }
        }
        CompositedFrame {
            buffer: canvas.buffer,
            width: canvas.width,
            height: canvas.height,
            duration: self.duration,
        }
    }
}

pub struct CompositedFrame {
    buffer: Vec<u8>,
    width: u16,
    height: u16,
    duration: u16,
}

pub struct Sprite {
    frames: Vec<CompositedFrame>,
    index: usize,
    time: u16,
}

impl Sprite {
    pub fn new(frames: Vec<CompositedFrame>) -> Self{
        Sprite {
            frames: frames,
            index: 0,
            time: 0,
        }
    }

    pub fn frame(&mut self) -> &CompositedFrame {
        if self.time >= self.frames[self.index].duration {
            self.time = 1;
            self.index = (self.index + 1) % self.frames.len();
            &self.frames[self.index]
        } else {
            self.time += 1;
            &self.frames[self.index as usize]
        }
    }

    pub fn width(&self) -> u16 {
        self.frames.iter().max_by_key(|f| f.width).unwrap().width
    }

    pub fn height(&self) -> u16 {
        self.frames.iter().max_by_key(|f| f.height).unwrap().height
    }
}

struct DNA<'a> {
    sizeb: u16,
    palet: u16,
    piece: u16,
    ename: u16,
    graphadr: u32,
    rom: &'a [u8],
    mb: u8,
}

fn snes_string(rom: &[u8], addr: usize) -> String {
    let mut v = Vec::new();
    for c in rom[addr..].iter().take_while(|c| **c != 0x20) {
        v.push(*c);
    }
    String::from_utf8(v).expect("Couldn't convert ascii to String")
}

impl<'a> DNA<'a> {
    fn read_from_rom(rom: &'a [u8], snes_addr: u32) -> Self {
        let addr = snespc2(snes_addr);
        let dna = &rom[addr..addr+64];

        DNA {
            sizeb: LittleEndian::read_u16(&dna[0..2]),
            palet: LittleEndian::read_u16(&dna[2..4]),
            mb: dna[12],
            piece: LittleEndian::read_u16(&dna[20..22]),
            graphadr: LittleEndian::read_u32(&dna[54..58]) & 0x00FFFFFF,
            ename: LittleEndian::read_u16(&dna[62..64]),
            rom: rom,
        }
    }

    fn name(&self) -> String {
        let addr = snespc(0x34, self.ename);
        snes_string(self.rom, addr)
    }

    fn palette(&self) -> &[u8] {
        let addr = snespc(self.mb, self.palet);
        &self.rom[addr..addr + 32]
    }

    fn frame_indices(&self, n: usize) -> Vec<FrameIndex> {
        let addr = snespc(self.mb, self.palet) + 0x20;
        self.rom[addr..addr + n * 4]
            .chunks(4)
            .map(|slice| {
                let duration = LittleEndian::read_u16(&slice[0..2]);
                let addr = LittleEndian::read_u16(&slice[2..4]);
                FrameIndex {
                    duration: duration,
                    snes_addr: addr,
                }
            })
            .collect()
    }

    fn frame_parts(&self, snes_addr: u16) -> Vec<FramePart> {
        let addr = snespc(self.mb, snes_addr);
        let num_parts = LittleEndian::read_u16(&self.rom[addr..addr+2]) as usize;
        self.rom[addr+2..addr+2+5*num_parts]
            .chunks(5)
            .map(|slice| FramePart {
                xx: slice[0] as i8,
                priority_a: slice[1],
                yy: slice[2] as i8,
                tl: slice[3],
                priority_b: slice[4],
            })
            .collect()
    }

    fn frames(&self, n: usize) -> Vec<Frame> {
        if n == 0 {
            return Vec::new();
        }
        let indices = self.frame_indices(n);
        indices.into_iter()
            .map(|fi| Frame {
                duration: fi.duration,
                parts: self.frame_parts(fi.snes_addr),
            })
            .collect()
    }

    fn graphics(&self) -> &[u8] {
        let addr = snespc2(self.graphadr);
        &self.rom[addr..addr + self.sizeb as usize]
    }
}

impl<'a> std::fmt::Debug for DNA<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,
            "DNA {{ sizeb: {:04X}, palet: {:04X}, mb: {:02X}, piece: {:04X}, graphadr: {:06X}, ename: {:02X} }}",
            self.sizeb, self.palet, self.mb, self.piece, self.graphadr, self.ename
        )
    }
}

// type RGBu8 = (u8, u8, u8);
type RGBf32 = (f32, f32, f32);

// fn bgr555_rgb888(bgr: u16) -> RGBu8 {
//     let r = (bgr & 0b11111) * 8;
//     let g = ((bgr & 0b1111100000) >> 5) * 8;
//     let b = ((bgr & 0b111110000000000) >> 10) * 8;
//     (r as u8, g as u8, b as u8)
// }

fn bgr555_rgbf32(bgr: u16) -> RGBf32 {
    let r = (bgr & 0b11111) as f32 / 31.0;
    let g = ((bgr & 0b1111100000) >> 5) as f32 / 31.0;
    let b = ((bgr & 0b111110000000000) >> 10) as f32 / 31.0;
    (r, g, b)
}

// fn bgr555_rgb565(bgr: u16) -> u16 {
//     // Used by some oled screens
//     let r = (bgr & 0b11111) << 11;
//     let g = ((bgr & 0b1111100000) >> 5) << 6;
//     let b = ((bgr & 0b111110000000000) >> 10);
//     r | g | b
// }

// fn lookup(bytes: &[u8], palette: &[RGBu8]) -> Vec<RGBu8> {
//     let mut v = Vec::with_capacity(bytes.len() * 2);
//     for byte in bytes {
//         v.push(palette[((byte >> 4) & 0xFu8) as usize]);
//         v.push(palette[(byte & 0xF) as usize]);
//     }
//     v
// }

fn main() {
    let ebi = DNA::read_from_rom(&ROM, 0xA0E63F);
    let gfx = ebi.graphics();

    let rgb_palette: Vec<_> = ebi.palette()
        .chunks(2)
        .map(|bgr| bgr555_rgbf32(LittleEndian::read_u16(bgr)))
        .collect();

    let tiles: Vec<_> = Bitplanes::new(gfx).collect();
    let frames: Vec<_> = ebi.frames(6).iter().map(|f| f.composited(&tiles)).collect();
    let mut sprite = Sprite::new(frames);

    let opengl = OpenGL::V3_2;
    let zoom = 8usize;
    let mut window: PistonWindow =
        WindowSettings::new(ebi.name(), [sprite.width() as u32 * zoom as u32, sprite.height() as u32 * zoom as u32])
            .exit_on_esc(true)
            .opengl(opengl)
            .vsync(true)
            .build()
            .unwrap();
    while let Some(event) = window.next() {
        window.draw_2d(&event, |context, graphics| {
            clear([1.0; 4], graphics);

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
