#![feature(proc_macro)]
extern crate proc_samus;
extern crate lib_samus;
extern crate core;

use lib_samus::pose::*;

proc_samus::samus_poses!([0x00, 0x01, 0x02, 0x09, 0x0A]);

proc_samus::samus_palettes!();

fn main() {
    let pose = poses::lookup(0).clone();
    let palette = &palette::PALETTE;
    println!("{:?}", pose.unwrap().name);
    println!("{:?}", palette);
}
