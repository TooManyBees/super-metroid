// http://old.metroidconstruction.com/docs/framedelays.TXT

use snes::{Rom, SnesAddress};
use std::{fmt};
use byteorder::{ByteOrder, LittleEndian};
use frame_map::{CompositedFrame, FrameMap};
use snes_bitplanes::{Bitplanes, Tile};

pub struct DNA<'a> {
    palet: u32,
    graphadr: u32,
    mb: u32,
    sizeb: u16,
    piece: u16,
    ename: u16,
    rom: &'a Rom<'a>,
}

impl<'a> DNA<'a> {
    pub fn read_from_rom(rom: &'a Rom, snes_addr: SnesAddress) -> Self {
        let addr = snes_addr.to_pc();
        let dna = &rom.read(addr, 64);

        DNA {
            sizeb: LittleEndian::read_u16(&dna[0..2]),
            palet: LittleEndian::read_u16(&dna[2..4]) as u32,
            mb: (dna[12] as u32) << 16,
            piece: LittleEndian::read_u16(&dna[20..22]),
            graphadr: LittleEndian::read_u32(&dna[54..58]) & 0x00FFFFFF,
            ename: LittleEndian::read_u16(&dna[62..64]),
            rom: rom,
        }
    }

    pub fn name(&self) -> Option<String> {
        let addr = SnesAddress((0x34 << 16) + self.ename as u32).to_pc();
        self.rom.read_string(addr, 16)
    }

    pub fn palette(&self) -> &[u8] {
        let addr = SnesAddress(self.mb + self.palet).to_pc();
        &self.rom.read(addr, 32)
    }

    fn frame_indices(&self) -> Vec<FrameIndex> {
        let addr = SnesAddress(self.mb + self.palet).to_pc() + 0x20;
        // Animations are followed by the ending bytes ED80 (littleendian)
        // and the LE short address of the *start* of the animation
        self.rom[addr..].chunks(4).take_while(|c| c[0..2] != [0xED, 0x80] && c[2..4] != [0xED, 0x80])
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

    pub fn frames(&self) -> Vec<Frame> {
        let indices = self.frame_indices();
        indices.into_iter()
            .map(|fi| {
                let full_addr = SnesAddress(self.mb + fi.snes_addr as u32);
                Frame {
                    duration: fi.duration,
                    parts: FrameMap::from_rom(self.rom, full_addr, 0),
                }
            })
            .collect()
    }

    pub fn graphics(&self) -> Vec<Tile> {
        let addr = SnesAddress(self.graphadr).to_pc();
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
    pub fn composited(&self, tiles: &[Tile]) -> CompositedFrame {
        FrameMap::composite(&self.parts, tiles, self.duration, 0)
    }
}
