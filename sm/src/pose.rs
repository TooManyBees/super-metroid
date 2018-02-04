use byteorder::{ByteOrder, LittleEndian};
use snes::{Rom, SnesAddress};
use frame_map::CompositedFrame;

use util::slice_as_tokens;
use quote::{Tokens, ToTokens};

const FRAME_DURATION_TABLE: SnesAddress = SnesAddress(0x91B010);
const FRAME_DURATION_START: SnesAddress = SnesAddress(0x910000);

pub struct Sequence<'a>(pub &'a [u8], pub Terminator);

#[derive(Copy, Clone)]
pub enum Terminator {
    Loop,
    Backtrack(u8),
    TransitionTo(u8),
    Stop,
}

impl ToTokens for Terminator {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let ts = match self {
            &Terminator::Loop => quote!(Terminator::Loop),
            &Terminator::Backtrack(ref b) => quote!(Terminator::Backtrack(#b)),
            &Terminator::TransitionTo(ref t) => quote!(Terminator::TransitionTo(#t)),
            &Terminator::Stop => quote!(Terminator::Stop),
        };
        tokens.append_all(ts);
    }
}

pub enum Next<'a> {
    Frame(&'a Frame<'a>),
    NewPose(u8),
}

pub struct Frame<'a> {
    pub buffer: &'a [u8],
    pub width: u16,
    pub height: u16,
    pub zero_x: u16,
    pub zero_y: u16,
    pub duration: u16,
}

impl<'a> Frame<'a> {
    pub fn new(frame: &'a CompositedFrame) -> Self {
        Frame {
            buffer: &frame.buffer,
            width: frame.width,
            height: frame.height,
            zero_x: frame.zero_x,
            zero_y: frame.zero_y,
            duration: frame.duration,
        }
    }
}

impl<'a> ToTokens for Frame<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let buffer_tokens = slice_as_tokens(self.buffer);
        let width = self.width;
        let height = self.height;
        let zero_x = self.zero_x;
        let zero_y = self.zero_y;
        let duration = self.duration;
        tokens.append_all(quote!{
            Frame {
                buffer: &#buffer_tokens,
                width: #width,
                height: #height,
                zero_x: #zero_x,
                zero_y: #zero_y,
                duration: #duration,
            }
        })
    }
}

pub struct Pose<'a> {
    pub name: &'a str,
    pub terminator: Terminator,
    pub durations: &'a [u8],
    pub frames: &'a [Frame<'a>],
    pub length: usize,
    pub cursor: usize,
}

impl<'a> Pose<'a> {
    pub fn lookup_frame_sequence(rom: &'a Rom, state: usize) -> Sequence<'a> {
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
        Sequence(&rom.read((FRAME_DURATION_START + addr).to_pc(), len), term)
    }

    pub fn new(name: &'a str, frames: &'a [Frame], sequence: &'a Sequence) -> Self {
        Pose {
            name,
            terminator: sequence.1,
            durations: sequence.0,
            length: frames.len(),
            frames,
            cursor: 0,
        }
    }

    pub fn next(&mut self) -> Next {
        if self.cursor >= self.length {
            match self.terminator {
                Terminator::Loop => {
                    self.cursor = 0;
                    Next::Frame(&self.frames[0])
                },
                Terminator::Backtrack(number_of_frames) => {
                    self.cursor -= number_of_frames as usize;
                    Next::Frame(&self.frames[self.cursor])
                },
                Terminator::Stop => Next::Frame(&self.frames[self.cursor-1]),
                Terminator::TransitionTo(pose) => Next::NewPose(pose),
            }
        } else {
            let f = Next::Frame(&self.frames[self.cursor]);
            self.cursor = (self.cursor + 1) % self.length;
            f
        }
    }

    // pub fn jump()

    // pub fn crouch()

    // pub fn morph()

    // pub fn turn_left()

    // pub fn turn_right()

    // pub fn move_left()

    // pub fn move_right()
}
