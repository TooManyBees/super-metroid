#![recursion_limit="128"]

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
extern crate sm;
#[macro_use] extern crate quote;
extern crate byteorder;

use proc_macro::TokenStream;
use std::str::FromStr;
use syn::DeriveInput;
use byteorder::{ByteOrder, LittleEndian};

use sm::{snes, samus, pose, frame_map, util};
use snes::{Rom, PcAddress};
use pose::{Pose, Frame};
use frame_map::FrameMap;
use util::{zip3, bgr555_rgb888};

const ROM_DATA: &'static [u8] = include_bytes!("../../data/Super Metroid (Japan, USA) (En,Ja).sfc");
const ROM: Rom = Rom(ROM_DATA);

fn parse_pose_id(ast: DeriveInput) -> (String, usize) {
    let mut name: Option<String> = None;
    let mut state: Option<usize> = None;
    for attr in ast.attrs.iter().filter_map(|attr| attr.interpret_meta()) {
        if let syn::Meta::NameValue(nv) = attr {
            let ident = nv.ident.to_string();
            if ident == "Name" {
                if let syn::Lit::Str(lit) = nv.lit {
                    name = Some(lit.value());
                }
            } else if ident == "State" {
                if let syn::Lit::Str(lit) = nv.lit {
                    state = Some(usize::from_str(&lit.value())
                            .expect("proc-samus::derive(SamusPose) `State` annotation must be parsable as usize"));
                }
            }
        }
    }
    (
        name.expect("proc-samus::derive(SamusPose) mising `#[Name = \"pose_name\"]` annotation"),
        state.expect("proc-samus::drive(SamusPose) missing `#[State = \"state_id\"]` annotation"),
    )
}

#[proc_macro_derive(SamusPose, attributes(State, Name))]
pub fn samus_pose(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input)
        .expect("proc-samus::derive(SamusPose) failed to parse input");

    let (name, state) = parse_pose_id(ast);

    println!("We parsed #[Name=\"{}\"] #[State=\"{}\"]", name, state);

    let sequence = Pose::lookup_frame_sequence(&ROM, state);
    let durations = sequence.0;
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
        duration: f.duration,
    }).collect();

    let sequence_terminator = sequence.1;

    TokenStream::from(quote!{
        use sm::pose::{Pose, Frame, Terminator};
        static DURATIONS: [u8; #sequence_len] = [#(#durations),*];
        static FRAMES: [Frame; #sequence_len] = [#(#borrow_frames),*];

        pub fn pose<'a>() -> Pose<'a> {
            Pose {
                name: #name,
                terminator: #sequence_terminator,
                durations: &DURATIONS,
                length: #sequence_len,
                cursor: 0,
                frames: &FRAMES,
            }
        }
    })
}

fn parse_address(ast: DeriveInput) -> usize {
    let mut addr: Option<usize> = None;
    for attr in ast.attrs.iter().filter_map(|attr| attr.interpret_meta()) {
        if let syn::Meta::NameValue(nv) = attr {
            let ident = nv.ident.to_string();
            if ident == "Addr" {
                if let syn::Lit::Str(lit) = nv.lit {
                    addr = Some(usize::from_str_radix(&lit.value(), 16)
                        .expect("proc-samus::derive(SamusPalette) `Addr` annotation must be hex-parsable as usize"));
                }
            }
        }
    }
    addr.expect("proc-samus::derive(SamusPalette) mising `#[Addr = \"SOME_ADDR\"]` annotation")
}

#[proc_macro_derive(SamusPalette, attributes(Addr))]
pub fn samus_palette(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input)
        .expect("proc-samus::derive(SamusPalette) failed to parse input");

    let addr = parse_address(ast);

    println!("We parsed palette #[Addr=\"{}\"]", addr);

    let p = PcAddress(addr);
    let palette = ROM.read(p, 32)
        .chunks(2)
        .map(LittleEndian::read_u16)
        .map(|c| bgr555_rgb888(&c));

    let palette_tokens: Vec<_> = palette.map(|(r, g, b)| quote!{(#r, #g, #b)}).collect();
    let palette_len = palette_tokens.len();

    TokenStream::from(quote!{
        use sm::util::RGBu8;
        pub static PALETTE: [RGBu8; #palette_len] = [#(#palette_tokens),*];
    })
}
