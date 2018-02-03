use snes::{Rom, SnesAddress};
use snes_bitplanes::Tile;
use std::{fmt, mem};
use byteorder::{ByteOrder, LittleEndian};
use centered_canvas;

pub struct CompositedFrame {
    pub buffer: Vec<u8>,
    pub width: u16,
    pub height: u16,
    pub zero_x: u16,
    pub zero_y: u16,
    pub duration: u16,
}

#[derive(Clone)]
pub struct FrameMap {
    x: i8,
    priority_a: u8,
    y: i8,
    tile: u8,
    priority_b: u8,
}

impl FrameMap {
    #[inline(always)]
    pub fn is_double(&self) -> bool {
        self.priority_a & (1 << 7) > 0
    }

    #[inline(always)]
    pub fn x(&self) -> i16 {
        self.x as i16
    }

    #[inline(always)]
    pub fn y(&self) -> i16 {
        self.y as i16
    }

    #[inline(always)]
    pub fn flip_horizontal(&self) -> bool {
        self.priority_b & (1 << 6) > 0
    }

    #[inline(always)]
    pub fn flip_vertical(&self) -> bool {
        self.priority_b & (1 << 7) > 0
    }

    #[inline(always)]
    pub fn load_next_page(&self) -> bool {
        self.priority_b & 1 > 0
    }

    pub fn from_rom(rom: &Rom, snes_addr: SnesAddress, offset: usize) -> Vec<Self> {
        // println!("snes addr: {:?}, offset: {:X}", snes_addr, offset);
        let addr = snes_addr.to_pc() + offset;
        let num_parts = LittleEndian::read_u16(&rom.read(addr, 2)) as usize;
        rom.read(addr+2, 5*num_parts)
            .chunks(5)
            .map(FrameMap::from_slice)
            .collect()
    }

    pub fn from_slice(slice: &[u8]) -> Self {
        FrameMap {
            x: slice[0] as i8,
            priority_a: slice[1],
            y: slice[2] as i8,
            tile: slice[3],
            priority_b: slice[4],
        }
    }

    pub fn composite(frame_maps: &[FrameMap], tiles: &[Tile], duration: u16) -> CompositedFrame {
        let (zx, zy, width, height) = dimensions(frame_maps);

        let mut buffer = vec![0; width as usize * height as usize];

        for part in frame_maps.iter().rev() {
            if part.is_double() {
                let n = part.tile as usize;
                for i in [n, n + 1, n + 16, n + 17].iter() {
                    if *i >= tiles.len() {
                        panic!("Frame part wants tile {} but we only have {}. Try a lower number of frames.", i, tiles.len());
                    }
                }
                // let n = if part.load_next_page() { n - 32 } else { n };
                let mut tile0 = &tiles[n];
                let mut tile1 = &tiles[n + 1];
                let mut tile2 = &tiles[n + 16];
                let mut tile3 = &tiles[n + 17];
                if part.flip_horizontal() {
                    mem::swap(&mut tile0, &mut tile1);
                    mem::swap(&mut tile2, &mut tile3);
                }
                if part.flip_vertical() {
                    mem::swap(&mut tile0, &mut tile2);
                    mem::swap(&mut tile1, &mut tile3);
                }
                centered_canvas::paint_block(&mut buffer, width, (zx, zy), (tile0, tile1, tile2, tile3), (part.x(), part.y()), part.flip_horizontal(), part.flip_vertical());
            } else {
                if part.tile as usize >= tiles.len() {
                    panic!("Frame part wants tile {} but we only have {}. Try a lower number of frames.", part.tile, tiles.len());
                }
                let tile = &tiles[part.tile as usize];
                centered_canvas::paint_tile(&mut buffer, width, (zx, zy), tile, (part.x(), part.y()), part.flip_horizontal(), part.flip_vertical());
            }
        }
        CompositedFrame {
            buffer: buffer,
            width: width,
            height: height,
            zero_x: zx,
            zero_y: zy,
            duration: duration,
        }
    }
}

impl fmt::Debug for FrameMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
            "FrameMap {{ x: {:02}, y: {:02}, tile: {:02X}, is_double: {} flip_x: {}, flip_y: {}, next_page: {} }}",
            self.x, self.y, self.tile, self.is_double(), self.flip_horizontal(), self.flip_vertical(), self.load_next_page()
        )
    }
}

fn dimensions(frame_maps: &[FrameMap]) -> (u16, u16, u16, u16) {
    let mut top = 0i16;
    let mut bottom = 0i16;
    let mut left = 0i16;
    let mut right = 0i16;
    for map in frame_maps.iter() {
        let size = if map.is_double() { 16 } else { 8 };
        if map.x() < left { left = map.x() };
        if map.x() + size > right { right = map.x() + size };
        if map.y() < top { top = map.y() };
        if map.y() + size > bottom { bottom = map.y() + size }
    }
    (-left as u16, -top as u16, (right - left) as u16, (bottom - top) as u16)
}
