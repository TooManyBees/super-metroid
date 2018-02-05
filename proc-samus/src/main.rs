#![feature(proc_macro)]
extern crate proc_samus;
extern crate sm;

proc_samus::samus_pose!(standing, 0x0D);

proc_samus::samus_palettes!();

fn main() {
    let pose = standing::pose();
    let palette = &palette::PALETTE;
    println!("{:?}", pose.name);
    println!("{:?}", palette);
}
