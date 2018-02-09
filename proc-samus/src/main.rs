#![feature(proc_macro)]
extern crate proc_samus;
extern crate sm;
extern crate core;

use sm::pose::*;

proc_samus::samus_poses!([0, 1, 2]);

proc_samus::samus_palettes!();

fn main() {
    let pose = poses::lookup(0).clone();
    let palette = &palette::PALETTE;
    println!("{:?}", pose.name);
    println!("{:?}", palette);
}
