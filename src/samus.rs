// http://metroidconstruction.com/SMMM/samus_animations.txt

use byteorder::{ByteOrder, LittleEndian};
use util::{snespc2, print_hex};

const BASE_TABLES_POINTER: u32 = 0x92808D;
const BOTTOM_HALF_POINTERS: u32 = 0x92945D;
const TOP_HALF_POINTERS: u32 = 0x929263;

const FRAME_PROGRESSION_TABLE_LOOKUP: u32 = 0x92D94E;
const FRAME_PROGRESSION_TABLES: u32 = 0x920000;

const TOP_DMA_LOOKUP: u32 = 0x92D91E;
const BOTTOM_DMA_LOOKUP: u32 = 0x92D938;

pub fn lookup_tilemap_table(rom: &[u8], state: usize, num_frames: usize) -> (&[u8], &[u8]) {
    let bottom_half = snespc2(BOTTOM_HALF_POINTERS) + state * 2;
    let top_half = snespc2(TOP_HALF_POINTERS) + state * 2;
    let base_addr = snespc2(BASE_TABLES_POINTER);
    let b = base_addr + LittleEndian::read_u16(&rom[bottom_half..bottom_half+2]) as usize * 2;
    let t = base_addr + LittleEndian::read_u16(&rom[top_half..top_half+2]) as usize * 2;
    (&rom[b..b + num_frames * 2], &rom[t..t + num_frames * 2])
}

pub fn lookup_frame_progression(rom: &[u8], state: usize, num_frames: usize) -> &[u8] {
    let lookup_addr = snespc2(FRAME_PROGRESSION_TABLE_LOOKUP) + state * 2;
    let offset = LittleEndian::read_u16(&rom[lookup_addr..lookup_addr+2]) as usize;
    let addr = snespc2(FRAME_PROGRESSION_TABLES) + offset;
    &rom[addr..addr + num_frames * 4]
}

type DMA_ENTRY = (usize, u16, u16);

fn read_dma(rom: &[u8], table_pointer: usize, entry: u8) -> DMA_ENTRY {
    let dma_offset = LittleEndian::read_u16(&rom[table_pointer..table_pointer+2]) as usize;
    let entry_offset = snespc2(FRAME_PROGRESSION_TABLES) + dma_offset + entry as usize * 7;
    let slice = &rom[entry_offset..entry_offset + 7];
    // It's really a 24-bit LE number in 3 bytes
    let snes_graphics_addr = LittleEndian::read_u32(&slice[0..4]) & 0x00FFFFFF;
    let graphics_addr = snespc2(snes_graphics_addr);
    let part_1_bytes = LittleEndian::read_u16(&slice[3..5]);
    let part_2_bytes = LittleEndian::read_u16(&slice[5..7]);
    (graphics_addr, part_1_bytes, part_2_bytes)
}

fn read_top_dma(rom: &[u8], index: u8, entry: u8) -> DMA_ENTRY {
    assert!(index <= 0xC, "Frame's top DMA table exceeds 0x0C");
    let base = snespc2(TOP_DMA_LOOKUP) + index as usize * 2;
    read_dma(rom, base, entry)
}

fn read_bottom_dma(rom: &[u8], index: u8, entry: u8) -> DMA_ENTRY {
    assert!(index <= 0xA, "Frame's bottom DMA table exceeds 0x0A");
    let base = snespc2(BOTTOM_DMA_LOOKUP) + index as usize * 2;
    read_dma(rom, base, entry)
}

pub fn lookup_frames(rom: &[u8], frame_gfx: &[u8]) {
    assert!(frame_gfx.len() % 4 == 0, "Frame progression is not evenly divisible by 4 bytes");
    for (n, frame) in frame_gfx.chunks(4).enumerate() {
        let top_dma_table = frame[0];
        let top_dma_entry = frame[1];
        let bottom_dma_table = frame[2];
        let bottom_dma_entry = frame[3];
        read_top_dma(rom, top_dma_table, top_dma_entry);
        read_bottom_dma(rom, bottom_dma_table, bottom_dma_entry);
    }
}
