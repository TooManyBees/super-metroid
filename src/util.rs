// const SNES_HEADER: bool = false;

// fn snespc(addrlo: usize, addrhi: usize, bank: usize) -> usize {
//     (addrlo & 255) + ((addrhi & 255) << 8) + ((bank & 127) << 15)
//       - (if SNES_HEADER {0} else {512}) - 32256
// }

// https://www.smwcentral.net/?p=viewthread&t=13167

#[inline(always)]
pub fn snespc(bank: u8, addr: u16) -> usize {
    (((bank & 127) as usize) << 15) + (addr as usize) - 512 - 32256
}

#[inline(always)]
pub fn snespc2(addr: u32) -> usize {
    (((addr & 0x7F0000) >> 1) + (addr & 0xFFFF)) as usize - 512 - 32256
}

pub fn snes_string(rom: &[u8], addr: usize) -> String {
    let mut v = Vec::new();
    for c in rom[addr..].iter().take_while(|c| **c != 0x20) {
        v.push(*c);
    }
    String::from_utf8(v).expect("Couldn't convert ascii to String")
}

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

pub fn bgr555_rgb888(bgr: u16) -> RGBu8 {
    let r = (bgr & 0b11111) * 8;
    let g = ((bgr & 0b1111100000) >> 5) * 8;
    let b = ((bgr & 0b111110000000000) >> 10) * 8;
    (r as u8, g as u8, b as u8)
}

pub fn bgr555_rgbf32(bgr: u16) -> RGBf32 {
    let r = (bgr & 0b11111) as f32 / 31.0;
    let g = ((bgr & 0b1111100000) >> 5) as f32 / 31.0;
    let b = ((bgr & 0b111110000000000) >> 10) as f32 / 31.0;
    (r, g, b)
}

pub fn bgr555_rgb565(bgr: u16) -> u16 {
    // Used by some oled screens
    let r = (bgr & 0b11111) << 11;
    let g = ((bgr & 0b1111100000) >> 5) << 6;
    let b = ((bgr & 0b111110000000000) >> 10);
    r | g | b
}
