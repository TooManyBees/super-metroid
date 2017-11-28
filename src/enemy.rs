// http://old.metroidconstruction.com/docs/framedelays.TXT

use snes::{Rom};
use std::{fmt};
use byteorder::{ByteOrder, LittleEndian};
use frame_map::FrameMap;
use sprite::CompositedFrame;
use bitplanes::Bitplanes;
#[allow(unused_imports)]
use util::{snespc, snespc2, snes_string, print_hex};

pub struct DNA<'a> {
    sizeb: u16,
    palet: u16,
    piece: u16,
    ename: u16,
    graphadr: u32,
    rom: &'a Rom<'a>,
    mb: u8,
}

impl<'a> DNA<'a> {
    pub fn read_from_rom(rom: &'a Rom, snes_addr: u32) -> Self {
        let addr = snespc2(snes_addr);
        let dna = &rom.read(addr, 64);

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
        &self.rom.read(addr, 32)
    }

    fn frame_indices(&self, n: usize) -> Vec<FrameIndex> {
        let addr = snespc(self.mb, self.palet) + 0x20;
        self.rom.read(addr, n * 4)
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

    pub fn frames(&self, n: usize) -> Vec<Frame> {
        if n == 0 {
            return Vec::new();
        }
        let indices = self.frame_indices(n);
        indices.into_iter()
            .map(|fi| {
                let full_addr = ((self.mb as u32) << 16) + fi.snes_addr as u32;
                Frame {
                    duration: fi.duration,
                    parts: FrameMap::from_rom(self.rom, full_addr as u32, 0),
                }
            })
            .collect()
    }

    pub fn graphics(&self) -> Vec<[u8; 64]> {
        let addr = snespc2(self.graphadr);
        let data = &self.rom.read(addr, self.sizeb as usize);
        Bitplanes::new(data).collect()
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

pub struct Frame {
    parts: Vec<FrameMap>,
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
    pub fn composited(&self, tiles: &[[u8; 64]]) -> CompositedFrame {
        FrameMap::composite(&self.parts, tiles, self.duration)
    }
}
