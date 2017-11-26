use std::{fmt, mem};
use byteorder::{ByteOrder, LittleEndian};
use centered_canvas::CenteredCanvas;
use sprite::CompositedFrame;
use util::snespc2;

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
        // if self.priority_a & 0x01 > 0 {
        //     self.x as i16 + 0xFF
        // } else {
            self.x as i16
        // }
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

    pub fn from_rom(rom: &[u8], snes_addr: u32, offset: usize) -> Vec<Self> {
        let addr = snespc2(snes_addr) + offset;
        let num_parts = LittleEndian::read_u16(&rom[addr..addr+2]) as usize;
        rom[addr+2..addr+2+5*num_parts]
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

    pub fn composite(frame_maps: &[FrameMap], tiles: &[[u8; 64]], duration: u16) -> CompositedFrame {
        let (zx, zy, width, height) = dimensions(frame_maps);

        let mut canvas = CenteredCanvas::new(width, height, (zx, zy));

        for part in frame_maps.iter().rev() {
            if part.is_double() {
                let n = part.tile as usize;
                for i in [n, n + 1, n + 16, n + 17].iter() {
                    if *i >= tiles.len() {
                        panic!("Frame part wants tile {} but we only have {}. Try a lower number of frames.", i, tiles.len());
                    }
                }
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
                canvas.paint_block(tile0, tile1, tile2, tile3, part.x(), part.y(), part.flip_horizontal(), part.flip_vertical());
            } else {
                if part.tile as usize >= tiles.len() {
                    panic!("Frame part wants tile {} but we only have {}. Try a lower number of frames.", part.tile, tiles.len());
                }
                let tile = &tiles[part.tile as usize];
                canvas.paint_tile(tile, part.x(), part.y(), part.flip_horizontal(), part.flip_vertical());
            }
        }
        CompositedFrame {
            buffer: canvas.buffer,
            width: canvas.width,
            height: canvas.height,
            duration: duration,
        }
    }
}

impl fmt::Debug for FrameMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
            "FrameMap {{ x: {:02}, priority_a: {:08b}, y: {:02}, tile: {:03}, priority_b: {:08b} }}",
            self.x, self.priority_a, self.y, self.tile, self.priority_b
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
