// http://old.metroidconstruction.com/tilemapediting.php
use snes_bitplanes::Tile;

fn _paint_tile(buffer: &mut [u8], tile: &Tile, width: usize, offset: usize, flip_x: bool, flip_y: bool) {
    let mut index = offset;
    if flip_y {
        for row in tile.chunks(8).rev() {
            _paint_row(buffer, row, index, flip_x);
            index += width as usize;
        }
    } else {
        for row in tile.chunks(8) {
            _paint_row(buffer, row, index, flip_x);
            index += width as usize;
        }
    }
}

fn _paint_row(buffer: &mut [u8], row: &[u8], offset: usize, flip_x: bool) {
    if flip_x {
        for (n, px) in row.iter().rev().enumerate() {
            if *px == 0 {
                continue;
            }
            buffer[offset + n] = *px;
        }
    } else {
        for (n, px) in row.iter().enumerate() {
            if *px == 0 {
                continue;
            }
            buffer[offset + n] = *px;
        }
    }
}

fn offset(width: u16, zx: u16, zy: u16, x: i16, y: i16) -> usize {
    let mut center = (zy * width + zx) as usize;
    if x >= 0 {
        center += x as usize;
    } else {
        center -= (-x) as usize;
    }
    if y >= 0 {
        center += y as usize * width as usize;
    } else {
        center -= (-y) as usize * width as usize;
    }
    center
}

pub fn paint_tile(buffer: &mut [u8], width: u16, (zx, zy): (u16, u16), tile: &Tile, (x, y): (i16, i16), flip_x: bool, flip_y: bool) {
    let offset = offset(width, zx, zy, x, y);
    _paint_tile(buffer, tile, width as usize, offset, flip_x, flip_y);
}

pub fn paint_block(buffer: &mut [u8], width: u16, (zx, zy): (u16, u16), (tile0, tile1, tile2, tile3): (&Tile, &Tile, &Tile, &Tile), (x, y): (i16, i16), flip_x: bool, flip_y: bool) {
    let offset = offset(width, zx, zy, x, y);
    let width = width as usize;
    _paint_tile(buffer, tile0, width, offset, flip_x, flip_y);
    _paint_tile(buffer, tile1, width, offset + 8, flip_x, flip_y);
    _paint_tile(buffer, tile2, width, offset + width*8, flip_x, flip_y);
    _paint_tile(buffer, tile3, width, offset + width*8 + 8, flip_x, flip_y);
}
