extern crate piston_window;
extern crate byteorder;

mod bitplanes;

use bitplanes::*;

use piston_window::*;
use byteorder::{ByteOrder, LittleEndian};
// use std::fs::File;
// use std::io::BufReader;
// use std::io::prelude::*;

// const BYTES: &'static [u8] = include_bytes!("data/thing.spr");
const ROM: &'static [u8] = include_bytes!("data/Super Metroid (Japan, USA) (En,Ja).sfc");

// const SNES_HEADER: bool = false;

// fn snespc(addrlo: usize, addrhi: usize, bank: usize) -> usize {
//     (addrlo & 255) + ((addrhi & 255) << 8) + ((bank & 127) << 15) - (if SNES_HEADER {0} else {512}) - 32256
// }

fn print_hex(arr: &[u8]) {
    print!("[");
    for byte in arr.iter().take(arr.len() - 1) {
        print!("{:02X} ", byte);

    }
    print!("{:02X}", arr[arr.len() - 1]);
    println!("]");
}

// https://www.smwcentral.net/?p=viewthread&t=13167

#[inline]
fn snespc(bank: u8, addr: u16) -> usize {
    (((bank & 127) as usize) << 15) + (addr as usize) - 512 - 32256
}

#[inline]
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

impl std::fmt::Debug for FramePart {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,
            "FramePart {{ xx: {:02X}, priority_b: {:02X}, yy: {:02X}, tl: {:02X}, priority_a: {:02X} }}",
            self.xx, self.priority_b, self.yy, self.tl, self.priority_a
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
        println!("graphics at {:06X}", addr);
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

type RGB = (u8, u8, u8);

fn bgr555_rgb888(bgr: u16) -> RGB {
    let r = (bgr & 0b11111) * 8;
    let g = ((bgr & 0b1111100000) >> 5) * 8;
    let b = ((bgr & 0b111110000000000) >> 10) * 8;
    (r as u8, g as u8, b as u8)
}

fn bgr555_rgbf32(bgr: u16) -> (f32, f32, f32) {
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

fn lookup(bytes: &[u8], palette: &[RGB]) -> Vec<RGB> {
    let mut v = Vec::with_capacity(bytes.len() * 2);
    for byte in bytes {
        v.push(palette[((byte >> 4) & 0xFu8) as usize]);
        v.push(palette[(byte & 0xF) as usize]);
    }
    v
}

fn main() {
    let ebi = DNA::read_from_rom(&ROM, 0xA0E63F);
    // println!("{:?}", ebi.frames(6));
    let gfx = ebi.graphics();
    println!();
    let rgb_palette: Vec<_> = ebi.palette()
        .chunks(2)
        .map(|bgr| bgr555_rgbf32(LittleEndian::read_u16(bgr)))
        .collect();

    let tiles: Vec<Vec<_>> = Bitplanes::new(gfx).collect();

    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow =
        WindowSettings::new(ebi.name(), [256, 24])
            .exit_on_esc(true)
            .opengl(opengl)
            .build()
            .unwrap();
    while let Some(event) = window.next() {
        window.draw_2d(&event, |context, graphics| {
            clear([1.0; 4], graphics);

            for (i, tile) in tiles.iter().enumerate() {
                let (tile_x, tile_y) = (i % 16, i / 16);
                for (j, index) in tile.iter().enumerate() {
                    let (r, g, b) = rgb_palette[*index as usize];
                    let (x, y) = (tile_x * 8 + j % 8, tile_y * 8 + j / 8);
                    rectangle(
                        [r, g, b, 1.0],
                        [(x * 8) as f64, (y * 8) as f64, 8.0, 8.0],
                        context.transform,
                        graphics
                    );
                }
            }

        });
    }
}
