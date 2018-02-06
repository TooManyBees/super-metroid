#![feature(proc_macro)]
#![recursion_limit="128"]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use] extern crate syn;
extern crate sm;
#[macro_use] extern crate quote;
extern crate byteorder;

use proc_macro::TokenStream;
use std::str::FromStr;
use syn::{Ident, Expr};
use syn::synom::Synom;
use quote::ToTokens;
use byteorder::{ByteOrder, LittleEndian};

use sm::{snes, samus, pose, frame_map, util};
use snes::{Rom, PcAddress};
use pose::{Pose, Frame};
use frame_map::FrameMap;
use util::{zip3, bgr555_rgb888};

const ROM_DATA: &'static [u8] = include_bytes!("../../data/Super Metroid (Japan, USA) (En,Ja).sfc");
const ROM: Rom = Rom(ROM_DATA);

struct PoseInfo {
    name: Ident,
    state: Expr,
}

impl Synom for PoseInfo {
    named!(parse -> Self, do_parse!(
        name: syn!(Ident) >>
        punct!(,) >>
        state: syn!(Expr) >>
        (PoseInfo { name, state })
    ));
}

fn parse_pose_id(input: TokenStream) -> (Ident, usize) {
    let PoseInfo { name, state } = syn::parse(input).expect("proc-samus::samus_pose: falied to parse input");

    let id = if let syn::Expr::Lit(expr_lit) = state {
        if let syn::Lit::Int(int) = expr_lit.lit {
            let s = int.into_tokens().to_string();
            usize::from_str(&s).unwrap_or_else(|_| usize::from_str_radix(&s.trim_left_matches("0x"), 16).expect("proc-samus::samus_pose: `state` must be a positive hex or decimal number"))
        } else {
            panic!("proc-samus::samus_pose: `state` must be a positive hex or decimal number");
        }
    } else {
        unreachable!("proc-samus::samus_pose: `state` is not a syn::Expr::Lit; PoseInfo::parse should have required this.");
    };

    ( name, id )
}

#[proc_macro]
pub fn samus_pose(input: TokenStream) -> TokenStream {
    let (name, state) = parse_pose_id(input);

    println!("We parsed samus_pose({}, {})", name, state);

    let name_str = name.into_tokens().to_string();
    let sequence = Pose::lookup_frame_sequence(&ROM, state);
    let durations = sequence.0;
    let sequence_terminator = sequence.1;
    let sequence_len = durations.len();

    let tile_maps = samus::tilemaps(&ROM, state, durations.len());
    let tile_sets = samus::graphics(&ROM, state, durations.len());
    let frames: Vec<_> = zip3(tile_maps, &tile_sets, durations)
        .map(|(tm, ts, ds)| FrameMap::composite(&tm, &ts, *ds as u16)).collect();
    let borrow_frames: Vec<_> = frames.iter().map(|f| Frame {
        buffer: &f.buffer,
        width: f.width,
        height: f.height,
        zero_x: f.zero_x,
        zero_y: f.zero_y,
    }).collect();

    TokenStream::from(quote!{
        mod #name {
            use sm::pose::{Pose, Frame, Terminator};
            static DURATIONS: [u8; #sequence_len] = [#(#durations),*];
            static FRAMES: [Frame; #sequence_len] = [#(#borrow_frames),*];

            pub fn pose<'a>() -> Pose<'a> {
                Pose {
                    name: #name_str,
                    terminator: #sequence_terminator,
                    durations: &DURATIONS,
                    length: #sequence_len,
                    cursor: 0,
                    frames: &FRAMES,
                }
            }
        }
    })
}

#[proc_macro]
pub fn samus_palettes(_input: TokenStream) -> TokenStream {
    let addr = 0xD9400;

    let p = PcAddress(addr);
    let palette = ROM.read(p, 32)
        .chunks(2)
        .map(LittleEndian::read_u16)
        .map(|c| bgr555_rgb888(&c));

    let palette_tokens: Vec<_> = palette.map(|(r, g, b)| quote!{(#r, #g, #b)}).collect();
    let palette_len = palette_tokens.len();

    TokenStream::from(quote!{
        mod palette {
            use sm::util::RGBu8;
            pub static PALETTE: [RGBu8; #palette_len] = [#(#palette_tokens),*];
        }
    })
}
