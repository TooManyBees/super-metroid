#[cfg(feature="codegen")] use quote::{Tokens, ToTokens};
pub use controller_input::ControllerInput;

#[derive(Copy, Clone, Debug)]
pub struct Transition {
    pub input: ControllerInput,
    pub to_pose: u8,
}

#[cfg(feature="codegen")]
impl ToTokens for Transition {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let input_bits = self.input.bits();
        let to_pose = self.to_pose;
        tokens.append_all(quote!{
            Transition {
                input: ControllerInput { bits: #input_bits },
                to_pose: #to_pose,
            }
        });
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Terminator {
    Loop,
    Backtrack(u8),
    TransitionTo(u8),
    Stop,
}

#[cfg(feature="codegen")]
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
    Frame(&'a Frame<'a>, u8),
    NewPose(u8),
}

pub struct Frame<'a> {
    pub buffer: &'a [u8],
    pub width: u16,
    pub height: u16,
    pub zero_x: u16,
    pub zero_y: u16,
}

#[cfg(feature="codegen")]
impl<'a> ToTokens for Frame<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let buffer = self.buffer;
        let width = self.width;
        let height = self.height;
        let zero_x = self.zero_x;
        let zero_y = self.zero_y;
        tokens.append_all(quote!{
            Frame {
                buffer: &[#(#buffer),*],
                width: #width,
                height: #height,
                zero_x: #zero_x,
                zero_y: #zero_y,
            }
        })
    }
}

#[derive(Clone)]
pub struct Pose<'a> {
    pub name: &'a str,
    pub id: usize,
    pub terminator: Terminator,
    pub durations: &'a [u8],
    pub frames: &'a [Frame<'a>],
    pub transitions: &'a [Transition],
    pub length: usize,
    pub cursor: usize,
}

impl<'a> Pose<'a> {
    pub fn next(&mut self) -> Next<'a> {
        let next = if self.cursor >= self.length {
            match self.terminator {
                Terminator::Loop => {
                    self.cursor = 0;
                    Next::Frame(&self.frames[0], self.durations[0])
                },
                Terminator::Backtrack(number_of_frames) => {
                    self.cursor -= number_of_frames as usize;
                    Next::Frame(&self.frames[self.cursor], self.durations[self.cursor])
                },
                Terminator::Stop => Next::Frame(&self.frames[self.cursor-1], self.durations[self.cursor-1]), //optimization?
                Terminator::TransitionTo(pose) => Next::NewPose(pose),
            }
        } else {
            Next::Frame(&self.frames[self.cursor], self.durations[self.cursor])
        };
        self.cursor = self.cursor + 1;
        next
    }
}
