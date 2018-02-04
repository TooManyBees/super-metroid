// const SNES_HEADER: bool = false;

// fn snespc(addrlo: usize, addrhi: usize, bank: usize) -> usize {
//     (addrlo & 255) + ((addrhi & 255) << 8) + ((bank & 127) << 15)
//       - (if SNES_HEADER {0} else {512}) - 32256
// }

// https://www.smwcentral.net/?p=viewthread&t=13167

use quote::{Tokens, ToTokens};
use proc_macro2::{TokenTree, TokenNode, Span, Spacing};

use std::cmp;

#[allow(dead_code)]
pub fn print_hex(arr: &[u8]) {
    print!("[");
    for byte in arr.iter().take(arr.len() - 1) {
        print!("{:02X} ", byte);

    }
    print!("{:02X}", arr[arr.len() - 1]);
    println!("]");
}

pub type RGBu8 = (u8, u8, u8);
pub type RGBf32 = (f32, f32, f32);

pub fn bgr555_rgb888(bgr: &u16) -> RGBu8 {
    let r = (bgr & 0b11111) * 8;
    let g = ((bgr & 0b1111100000) >> 5) * 8;
    let b = ((bgr & 0b111110000000000) >> 10) * 8;
    (r as u8, g as u8, b as u8)
}

pub fn bgr555_rgbf32(bgr: &u16) -> RGBf32 {
    let r = (bgr & 0b11111) as f32 / 31.0;
    let g = ((bgr & 0b1111100000) >> 5) as f32 / 31.0;
    let b = ((bgr & 0b111110000000000) >> 10) as f32 / 31.0;
    (r, g, b)
}

#[allow(dead_code)]
pub fn bgr555_rgb565(bgr: &u16) -> u16 {
    // Used by some oled screens
    let r = (bgr & 0b11111) << 11;
    let g = ((bgr & 0b1111100000) >> 5) << 6;
    let b = (bgr & 0b111110000000000) >> 10;
    r | g | b
}

pub struct Zip3<T, U, V> {
    t: T,
    u: U,
    v: V,
}

impl<T, U, V> Iterator for Zip3<T, U, V>
    where T: Iterator, U: Iterator, V: Iterator {
    type Item = (T::Item, U::Item, V::Item);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.t.next().and_then(|t| {
            self.u.next().and_then(|u| {
                self.v.next().and_then(|v| {
                    Some((t, u, v))
                })
            })
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (t_l, t_u) = self.t.size_hint();
        let (u_l, u_u) = self.u.size_hint();
        let (v_l, v_u) = self.v.size_hint();

        let lower = cmp::min(cmp::min(t_l, u_l), v_l);

        let upper = match (t_u, u_u, v_u) {
            (Some(x), Some(y), Some(z)) => Some(cmp::min(cmp::min(x, y), z)),
            (Some(x), Some(y),    None) => Some(cmp::min(x, y)),
            (Some(x),    None, Some(z)) => Some(cmp::min(x, z)),
            (   None, Some(y), Some(z)) => Some(cmp::min(y, z)),
            (Some(x),    None,    None) => Some(x),
            (   None, Some(y),    None) => Some(y),
            (   None,    None, Some(z)) => Some(z),
            (   None,    None,    None) => None,
        };

        (lower, upper)
    }
}

pub fn zip3<T, U, V>(t: T, u: U, v: V) -> Zip3<T::IntoIter, U::IntoIter, V::IntoIter>
    where T: IntoIterator, U: IntoIterator, V: IntoIterator {
    Zip3 { t: t.into_iter(), u: u.into_iter(), v: v.into_iter() }
}

pub fn tt(kind: TokenNode) -> TokenTree {
    TokenTree {
        span: Span::def_site(),
        kind: kind,
    }
}

pub fn slice_as_tokens<T: ToTokens>(slice: &[T]) -> Tokens {
    let mut tokens = quote!();
    tokens.append(tt(TokenNode::Op('[', Spacing::Joint)));
    for item in slice {
        item.to_tokens(&mut tokens);
        tokens.append(tt(TokenNode::Op(',', Spacing::Alone)));
    }
    tokens.append(tt(TokenNode::Op(']', Spacing::Joint)));
    tokens
}

pub fn tuple3_as_tokens<T: ToTokens>(tuple: &(T, T, T)) -> Tokens {
    let mut tokens = quote!();
    tokens.append(tt(TokenNode::Op('(', Spacing::Joint)));
    tuple.0.to_tokens(&mut tokens);
    tokens.append(tt(TokenNode::Op(',', Spacing::Alone)));
    tuple.1.to_tokens(&mut tokens);
    tokens.append(tt(TokenNode::Op(',', Spacing::Alone)));
    tuple.2.to_tokens(&mut tokens);
    tokens.append(tt(TokenNode::Op(')', Spacing::Joint)));
    tokens
}
