// http://metroidconstruction.com/SMMM/samus_animations.txt

use snes::{Rom, PcAddress, SnesAddress};
use byteorder::{ByteOrder, LittleEndian};
use snes_bitplanes::{Bitplanes, Tile};
use frame_map::FrameMap;
use lib_samus::pose::{ControllerInput, Terminator, Transition};

const BASE_TABLES_POINTER: SnesAddress = SnesAddress(0x92808D);
const BOTTOM_HALF_POINTERS: SnesAddress = SnesAddress(0x92945D);
const TOP_HALF_POINTERS: SnesAddress = SnesAddress(0x929263);

const FRAME_MAP_START: SnesAddress = SnesAddress(0x918000);

const FRAME_PROGRESSION_TABLE_LOOKUP: SnesAddress = SnesAddress(0x92D94E);
const FRAME_PROGRESSION_TABLES: SnesAddress = SnesAddress(0x920000);

const FRAME_DURATION_TABLE: SnesAddress = SnesAddress(0x91B010);
const FRAME_DURATION_START: SnesAddress = SnesAddress(0x910000);

const POSE_TRANSITION_TABLE: SnesAddress = SnesAddress(0x919EE2);

const TOP_DMA_LOOKUP: SnesAddress = SnesAddress(0x92D91E);
const BOTTOM_DMA_LOOKUP: SnesAddress = SnesAddress(0x92D938);

pub fn tilemaps(rom: &Rom, state: usize, num_frames: usize) -> Vec<Vec<FrameMap>> {
    let (bottom_pointers, top_pointers) = lookup_tilemap_table(rom, state, num_frames);

    top_pointers.chunks(2).map(LittleEndian::read_u16)
    .zip(bottom_pointers.chunks(2).map(LittleEndian::read_u16))
    .map(|(addr_t, addr_b)| {
        let mut maps = Vec::with_capacity(num_frames * 2); // Assume both top and bottom will have data
        if addr_t != 0 {
            maps.append(&mut FrameMap::from_rom(rom, FRAME_MAP_START, addr_t as usize));
        };
        if addr_b != 0 {
            maps.append(&mut FrameMap::from_rom(rom, FRAME_MAP_START, addr_b as usize));
        };
        maps
    })
    .collect()
}

pub fn graphics(rom: &Rom, state: usize, num_frames: usize) -> Vec<Vec<Tile>> {
    let pointers = lookup_frame_dma_pointers(rom, state, num_frames);
    let data = lookup_graphics_data(rom, pointers);
    data.into_iter().map(|(t, b)| generate_graphics(rom, t, b)).collect()
}

pub struct Sequence<'a>(pub &'a [u8], pub Terminator, pub Vec<Transition>);

pub fn lookup_frame_sequence<'a>(rom: &'a Rom, state: usize) -> Sequence<'a> {
    let addr = LittleEndian::read_u16(&rom.read(FRAME_DURATION_TABLE.to_pc() + state * 2, 2)) as u32;
    let mut len = 0;
    let mut term = Terminator::Loop;
    for bytes in rom[(FRAME_DURATION_START + addr).to_pc()..].windows(2) {
        if bytes[0] >= 0xF0 {
            term = match bytes[0] {
                0xFF => Terminator::Loop,
                0xFE => Terminator::Backtrack(bytes[1]),
                0xFD => Terminator::TransitionTo(bytes[1]), // possibly a second extra byte of data
                0xFB => Terminator::Loop, // wall jump ??
                0xF9 => Terminator::Loop, // unsure, possibly 6 more bytes though
                0xF8 => Terminator::TransitionTo(bytes[1]),
                0xF6 => Terminator::Loop, // heavy breathing ??
                0xF0 => Terminator::Stop,
                0xF1 | 0xF2 | 0xF3 | 0xF4 | 0xF5 | 0xF7 | 0xFA | 0xFC => Terminator::Loop,
                _ => unreachable!(),
            };
            break;
        };
        len += 1;
    };
    let transitions = lookup_pose_transitions(rom, state);
    Sequence(&rom.read((FRAME_DURATION_START + addr).to_pc(), len), term, transitions)
}

pub fn lookup_pose_transitions<'a>(rom: &'a Rom, state: usize) -> Vec<Transition> {
    let offset = LittleEndian::read_u16(&rom.read(POSE_TRANSITION_TABLE.to_pc() + state * 2, 2)) as u32;
    let addr = (FRAME_DURATION_START + offset).to_pc();
    let mut num_transitions = 0;
    for (n, bytes) in rom[addr..].chunks(6).enumerate() {
        if bytes[0] == 0xFF && bytes[1] == 0xFF {
            num_transitions = n;
            break;
        }
    }

    rom[addr..addr + (6 * num_transitions)].chunks(6).map(|slice| {
        let controls_1 = LittleEndian::read_u16(&slice[0..2]);
        let controls_2 = LittleEndian::read_u16(&slice[2..4]);
        let input = ControllerInput::from_bits_truncate(controls_1 | controls_2);
        let to_pose = LittleEndian::read_u16(&slice[4..6]) as u8;

        Transition { input: input, to_pose }
    }).collect()
}

fn lookup_tilemap_table<'a>(rom: &'a Rom, state: usize, num_frames: usize) -> (&'a [u8], &'a [u8]) {
    let bottom_half = BOTTOM_HALF_POINTERS.to_pc() + state * 2;
    let top_half = TOP_HALF_POINTERS.to_pc() + state * 2;
    let base_addr = BASE_TABLES_POINTER.to_pc();
    let b = base_addr + LittleEndian::read_u16(&rom.read(bottom_half, 2)) as usize * 2;
    let t = base_addr + LittleEndian::read_u16(&rom.read(top_half, 2)) as usize * 2;
    (&rom.read(b, num_frames * 2), &rom.read(t, num_frames * 2))
}

fn lookup_frame_dma_pointers<'a>(rom: &'a Rom, state: usize, num_frames: usize) -> &'a [u8] {
    let lookup_addr = FRAME_PROGRESSION_TABLE_LOOKUP.to_pc() + state * 2;
    let offset = LittleEndian::read_u16(&rom.read(lookup_addr, 2)) as usize;
    let addr = FRAME_PROGRESSION_TABLES.to_pc() + offset;
    &rom.read(addr, num_frames * 4)
}

type DmaEntry = (PcAddress, usize, usize);

fn read_dma(rom: &Rom, table_pointer: PcAddress, entry: u8) -> DmaEntry {
    let dma_offset = LittleEndian::read_u16(&rom.read(table_pointer, 2)) as usize;
    let entry_offset = FRAME_PROGRESSION_TABLES.to_pc() + dma_offset + entry as usize * 7;
    let slice = &rom.read(entry_offset, 7);
    let snes_graphics_addr = LittleEndian::read_u24(&slice[0..3]);
    let graphics_addr = SnesAddress(snes_graphics_addr).to_pc();
    let part_1_bytes = LittleEndian::read_u16(&slice[3..5]) as usize;
    let part_2_bytes = LittleEndian::read_u16(&slice[5..7]) as usize;
    // println!("({:06X}, {:04X}, {:04X})", snes_graphics_addr, part_1_bytes, part_2_bytes);
    (graphics_addr, part_1_bytes, part_2_bytes)
}

fn read_top_dma(rom: &Rom, index: u8, entry: u8) -> DmaEntry {
    assert!(index <= 0xC, "Frame's top DMA table exceeds 0x0C");
    // println!("Top DMA lookup: {} {}", index, entry);
    let base = TOP_DMA_LOOKUP.to_pc() + index as usize * 2;
    read_dma(rom, base, entry)
}

fn read_bottom_dma(rom: &Rom, index: u8, entry: u8) -> DmaEntry {
    assert!(index <= 0xA, "Frame's bottom DMA table exceeds 0x0A");
    // println!("Bottom DMA lookup: {} {}", index, entry);
    let base = BOTTOM_DMA_LOOKUP.to_pc() + index as usize * 2;
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
fn generate_graphics(rom: &Rom, top_frame: DmaEntry, bottom_frame: DmaEntry) -> Vec<Tile> {
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

    let top_part1_padding = (0..((HALF_ROW - top_frame.1) / 32)).map(|_| Tile::default());
    let top_part2_padding = (0..((HALF_ROW - top_frame.2) / 32)).map(|_| Tile::default());
    let bottom_part1_padding = (0..((HALF_ROW - bottom_frame.1) / 32)).map(|_| Tile::default());
    let bottom_part2_padding = (0..((HALF_ROW - bottom_frame.2) / 32)).map(|_| Tile::default());


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
