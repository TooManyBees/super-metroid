// http://metroidconstruction.com/SMMM/samus_animations.txt

use snes::{Rom, PcAddress};
use byteorder::{ByteOrder, LittleEndian};
use bitplanes::Bitplanes;
use frame_map::FrameMap;
use util::{snespc2, print_hex};

const BASE_TABLES_POINTER: u32 = 0x92808D;
const BOTTOM_HALF_POINTERS: u32 = 0x92945D;
const TOP_HALF_POINTERS: u32 = 0x929263;

const FRAME_MAP_START: u32 = 0x918000;

const FRAME_PROGRESSION_TABLE_LOOKUP: u32 = 0x92D94E;
const FRAME_PROGRESSION_TABLES: u32 = 0x920000;

const FRAME_DURATION_TABLE: u32 = 0x91B010;

const TOP_DMA_LOOKUP: u32 = 0x92D91E;
const BOTTOM_DMA_LOOKUP: u32 = 0x92D938;

pub fn tilemaps(rom: &Rom, state: usize, num_frames: usize) -> Vec<Vec<FrameMap>> {
    let (bottom_pointers, top_pointers) = lookup_tilemap_table(rom, state, num_frames);

    top_pointers.chunks(2).map(LittleEndian::read_u16)
    .zip(bottom_pointers.chunks(2).map(LittleEndian::read_u16))
    .map(|(addr_t, addr_b)| {
        let mut v = FrameMap::from_rom(rom, FRAME_MAP_START, addr_t as usize);
        v.append(&mut FrameMap::from_rom(rom, FRAME_MAP_START, addr_b as usize));
        v
    })
    .collect()
}

pub fn graphics(rom: &Rom, state: usize, num_frames: usize) -> Vec<Vec<[u8; 64]>> {
    let pointers = lookup_frame_dma_pointers(rom, state, num_frames);
    let data = lookup_graphics_data(rom, pointers);
    data.into_iter().map(|(t, b)| generate_graphics(rom, t, b)).collect()
}

pub fn lookup_frame_durations<'a>(rom: &'a Rom, state: usize, num_frames: usize) -> &'a [u8] {
    let addr = snespc2(FRAME_DURATION_TABLE) + state * 2;
    &rom.read(addr, num_frames)
}

fn lookup_tilemap_table<'a>(rom: &'a Rom, state: usize, num_frames: usize) -> (&'a [u8], &'a [u8]) {
    let bottom_half = snespc2(BOTTOM_HALF_POINTERS) + state * 2;
    let top_half = snespc2(TOP_HALF_POINTERS) + state * 2;
    let base_addr = snespc2(BASE_TABLES_POINTER);
    let b = base_addr + LittleEndian::read_u16(&rom.read(bottom_half, 2)) as usize * 2;
    let t = base_addr + LittleEndian::read_u16(&rom.read(top_half, 2)) as usize * 2;
    (&rom.read(b, num_frames * 2), &rom.read(t, num_frames * 2))
}

fn lookup_frame_dma_pointers<'a>(rom: &'a Rom, state: usize, num_frames: usize) -> &'a [u8] {
    let lookup_addr = snespc2(FRAME_PROGRESSION_TABLE_LOOKUP) + state * 2;
    let offset = LittleEndian::read_u16(&rom.read(lookup_addr, 2)) as usize;
    let addr = snespc2(FRAME_PROGRESSION_TABLES) + offset;
    &rom.read(addr, num_frames * 4)
}

type DmaEntry = (PcAddress, usize, usize);

fn read_dma(rom: &Rom, table_pointer: PcAddress, entry: u8) -> DmaEntry {
    let dma_offset = LittleEndian::read_u16(&rom.read(table_pointer, 2)) as usize;
    let entry_offset = snespc2(FRAME_PROGRESSION_TABLES) + dma_offset + entry as usize * 7;
    let slice = &rom.read(entry_offset, 7);
    // It's really a 24-bit LE number in 3 bytes
    let snes_graphics_addr = LittleEndian::read_u32(&slice[0..4]) & 0x00FFFFFF;
    let graphics_addr = snespc2(snes_graphics_addr);
    let part_1_bytes = LittleEndian::read_u16(&slice[3..5]) as usize;
    let part_2_bytes = LittleEndian::read_u16(&slice[5..7]) as usize;
    // println!("({:06X}, {:04X}, {:04X})", snes_graphics_addr, part_1_bytes, part_2_bytes);
    (graphics_addr, part_1_bytes, part_2_bytes)
}

fn read_top_dma(rom: &Rom, index: u8, entry: u8) -> DmaEntry {
    assert!(index <= 0xC, "Frame's top DMA table exceeds 0x0C");
    // println!("Top DMA lookup: {} {}", index, entry);
    let base = snespc2(TOP_DMA_LOOKUP) + index as usize * 2;
    read_dma(rom, base, entry)
}

fn read_bottom_dma(rom: &Rom, index: u8, entry: u8) -> DmaEntry {
    assert!(index <= 0xA, "Frame's bottom DMA table exceeds 0x0A");
    // println!("Bottom DMA lookup: {} {}", index, entry);
    let base = snespc2(BOTTOM_DMA_LOOKUP) + index as usize * 2;
    read_dma(rom, base, entry)
}

fn lookup_graphics_data(rom: &Rom, pointer_entries: &[u8]) -> Vec<(DmaEntry, DmaEntry)> {
    assert!(pointer_entries.len() % 4 == 0, "Frame progression is not evenly divisible by 4 bytes");
    pointer_entries.chunks(4).map(|frame| {
        let top_dma_table = frame[0];
        let top_dma_entry = frame[1];
        let bottom_dma_table = frame[2];
        let bottom_dma_entry = frame[3];
        (
            read_top_dma(rom, top_dma_table, top_dma_entry),
            read_bottom_dma(rom, bottom_dma_table, bottom_dma_entry),
        )
    }).collect()
}

static HALF_ROW: usize = 0x0100;
fn generate_graphics(rom: &Rom, top_frame: DmaEntry, bottom_frame: DmaEntry) -> Vec<[u8; 64]> {
    debug_assert!(top_frame.1 <= HALF_ROW);
    debug_assert!(top_frame.2 <= HALF_ROW);
    debug_assert!(bottom_frame.1 <= HALF_ROW);
    debug_assert!(bottom_frame.2 <= HALF_ROW);

    /*
    Oh jeez
    http://old.metroidconstruction.com/images/crashtour99_VRAMview.png

    For both the top and bottom parts, we read `frame.1 + frame.2` number
    of bytes, starting at the address `frame.0`. Both parts will be at most
    0x0100 bytes, which is 8 tiles.

    The final tile map must be arranged in two rows of 16 tiles, in which
    the left 8 tiles of each row belong to the top of the frame, and the right
    8 tiles belong to the bottom of the frame.

    Since the parts can be less 8 tiles (an even half of a row) we need to pad.

    If the numbers look funny, remember we count the needed frames by counting
    how many tiles would fit in the rest of the half row (at 4bpp) and mapping
    that to tiles of decoded pixels (at 8bpp) hence the discrepancy in numbers:
    32 -> 64
    */

    let top_part1_padding = (0..((HALF_ROW - top_frame.1) / 32)).map(|_| [0; 64]);
    let top_part2_padding = (0..((HALF_ROW - top_frame.2) / 32)).map(|_| [0; 64]);
    let bottom_part1_padding = (0..((HALF_ROW - bottom_frame.1) / 32)).map(|_| [0; 64]);
    let bottom_part2_padding = (0..((HALF_ROW - bottom_frame.2) / 32)).map(|_| [0; 64]);


    Bitplanes::new(&rom.read(top_frame.0, top_frame.1))
    .chain(top_part1_padding)
    .chain(Bitplanes::new(&rom.read(bottom_frame.0, bottom_frame.1)))
    .chain(bottom_part1_padding)
    .chain(Bitplanes::new(&rom.read(top_frame.0 + top_frame.1, top_frame.2)))
    .chain(top_part2_padding)
    .chain(Bitplanes::new(&rom.read(bottom_frame.0 + bottom_frame.1, bottom_frame.2)))
    .chain(bottom_part2_padding)
    .collect()
}
