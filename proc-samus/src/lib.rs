#![feature(proc_macro)]
#![recursion_limit="128"]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use] extern crate syn;
#[macro_use] extern crate quote;
extern crate byteorder;

extern crate sm;
extern crate lib_samus;

mod poses_list;

use proc_macro::TokenStream;
use std::str::FromStr;
use std::collections::HashSet;
use syn::{Ident, Expr};
use syn::punctuated::Punctuated;
use syn::synom::Synom;
use quote::{Tokens, ToTokens};
use byteorder::{ByteOrder, LittleEndian};

use sm::{snes, samus, frame_map, util};
use snes::{Rom, PcAddress};
use lib_samus::pose::Frame;
use frame_map::FrameMap;
use util::{zip3, bgr555_rgb888};

const ROM_DATA: &'static [u8] = include_bytes!("../../data/Super Metroid (Japan, USA) (En,Ja).sfc");
const ROM: Rom = Rom(ROM_DATA);

fn parse_pose_state(state: syn::Expr) -> usize {
    if let syn::Expr::Lit(expr_lit) = state {
        if let syn::Lit::Int(int) = expr_lit.lit {
            let s = int.into_tokens().to_string();
            usize::from_str(&s).unwrap_or_else(|_| {
                usize::from_str_radix(&s.trim_left_matches("0x"), 16)
                .expect("proc-samus::samus_pose: `state` must be a positive hex or decimal number")
            })
        } else {
            panic!("proc-samus::samus_pose: `state` must be a positive hex or decimal number");
        }
    } else {
        unreachable!("proc-samus::samus_pose: `state` is not a syn::Expr::Lit; PoseInfo::parse should have required this.");
    }
}

fn samus_pose_struct_tokens(name: Ident, state: usize) -> Tokens {
    let name_str = name.into_tokens().to_string();
    let sequence = samus::lookup_frame_sequence(&ROM, state);
    let durations = sequence.0;
    let sequence_terminator = sequence.1;
    let transitions = sequence.2;
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

    quote!{
        Pose {
            name: #name_str,
            id: #state,
            terminator: #sequence_terminator,
            durations: &[#(#durations),*],
            transitions: &[#(#transitions),*],
            length: #sequence_len,
            cursor: 0,
            frames: &[#(#borrow_frames),*],
        }
    }
}

struct Chosen {
    ids: HashSet<Expr>,
}

impl Synom for Chosen {
    named!(parse -> Self, map!(
        brackets!(Punctuated::<Expr, Token![,]>::parse_terminated),
        |(_parens, vars)| Chosen {
            ids: vars.into_iter().collect(),
        }
    ));
}

fn parse_chosen_poses(input: TokenStream) -> Vec<(Ident, usize)> {
    let Chosen { ids } = syn::parse(input).expect("eep, hi there");
    let chosen: HashSet<_> = ids.into_iter().map(|state| parse_pose_state(state)).collect();

    poses_list::ALL.iter()
        .filter_map(|&(state, name_str, _v_offset)| {
            if chosen.is_empty() || chosen.contains(&state) {
                let name = Ident::from(name_str);
                Some((name, state))
            } else {
                None
            }
        })
        .collect()
}

const NUM_POSES: usize = 256;

#[proc_macro]
pub fn samus_poses(input: TokenStream) -> TokenStream {
    let poses = parse_chosen_poses(input);
    let poses_tokens: Vec<_> = poses.iter().map(|&(name, state)| samus_pose_struct_tokens(name, state)).collect();

    let mut arr = vec![255u8; NUM_POSES];
    for (n, &(_name, state)) in poses.iter().enumerate() {
        arr[state] = n as u8;
    }

    TokenStream::from(quote!{
        mod poses {
            static POSES: &[Pose] = &[#(#poses_tokens),*];
            static LOOKUP: &[u8] = &[#(#arr), *];

            pub fn lookup(n: usize) -> Option<&'static Pose<'static>> {
                let index = LOOKUP[n];
                if index == 255 {
                    None
                } else {
                    Some(&POSES[index as usize])
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
            pub static PALETTE: [(u8, u8, u8); #palette_len] = [#(#palette_tokens),*];
        }
    })
}
