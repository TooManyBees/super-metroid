use std::{fmt, mem};
use byteorder::{ByteOrder, LittleEndian};
use centered_canvas::CenteredCanvas;
use sprite::CompositedFrame;
#[allow(unused_imports)]
use util::{snespc, snespc2, snes_string, print_hex};

pub struct DNA<'a> {
    sizeb: u16,
    palet: u16,
    piece: u16,
    ename: u16,
    graphadr: u32,
    rom: &'a [u8],
    mb: u8,
}

impl<'a> DNA<'a> {
    pub fn read_from_rom(rom: &'a [u8], snes_addr: u32) -> Self {
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

    pub fn name(&self) -> Option<String> {
        let addr = snespc(0x34, self.ename);
        snes_string(self.rom, addr)
    }

    pub fn palette(&self) -> &[u8] {
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
        // print_hex(&self.rom[addr..addr+2+5*num_parts]);
        // print_hex(&self.rom[addr+2+5*num_parts..addr+2+5*num_parts+50]);
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

    pub fn frames(&self, n: usize) -> Vec<Frame> {
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

    pub fn graphics(&self) -> &[u8] {
        let addr = snespc2(self.graphadr);
        &self.rom[addr..addr + self.sizeb as usize]
    }
}

impl<'a> fmt::Debug for DNA<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
            "DNA {{ sizeb: {:04X}, palet: {:04X}, mb: {:02X}, piece: {:04X}, graphadr: {:06X}, ename: {:02X} }}",
            self.sizeb, self.palet, self.mb, self.piece, self.graphadr, self.ename
        )
    }
}

struct FrameIndex {
    duration: u16,
    snes_addr: u16,
}

impl fmt::Debug for FrameIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
        self.priority_a & (1 << 7) > 0
    }

    #[inline(always)]
    fn x(&self) -> i16 {
        // if self.priority_a & 0x01 > 0 {
        //     self.xx as i16 + 0xFF
        // } else {
            self.xx as i16
        // }
    }

    #[inline(always)]
    fn y(&self) -> i16 {
        self.yy as i16
    }

    #[inline(always)]
    fn flip_horizontal(&self) -> bool {
        self.priority_b & (1 << 6) > 0
    }

    #[inline(always)]
    fn flip_vertical(&self) -> bool {
        self.priority_b & (1 << 7) > 0
    }
}

impl fmt::Debug for FramePart {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
            "FramePart {{ xx: {:02}, priority_a: {:08b}, yy: {:02}, tl: {:03}, priority_b: {:08b} }}",
            self.xx, self.priority_a, self.yy, self.tl, self.priority_b
        )
    }
}

pub struct Frame {
    parts: Vec<FramePart>,
    duration: u16,
}

impl fmt::Debug for Frame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
            "Frame {{ duration: {:02X}, parts: {:?} }}",
            self.duration, self.parts
        )
    }
}

impl Frame {
    fn dimensions(&self) -> (u16, u16, u16, u16) {
        let mut top = 0i16;
        let mut bottom = 0i16;
        let mut left = 0i16;
        let mut right = 0i16;
        for part in self.parts.iter() {
            let size = if part.is_double() { 16 } else { 8 };
            if part.x() < left { left = part.x() };
            if part.x() + size > right { right = part.x() + size };
            if part.y() < top { top = part.y() };
            if part.y() + size > bottom { bottom = part.y() + size }
        }
        (-left as u16, -top as u16, (right - left) as u16, (bottom - top) as u16)
    }

    pub fn composited(&self, tiles: &[Vec<u8>]) -> CompositedFrame {
        let (zx, zy, width, height) = self.dimensions();

        let mut canvas = CenteredCanvas::new(width, height, (zx, zy));

        for part in self.parts.iter().rev() {
            if part.is_double() {
                let n = part.tl as usize;
                let mut tile0 = &tiles[n];
                let mut tile1 = &tiles[n+1];
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
                let tile = &tiles[part.tl as usize];
                canvas.paint_tile(tile, part.x(), part.y(), part.flip_horizontal(), part.flip_vertical());
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
