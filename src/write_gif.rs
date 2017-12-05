use gif::{Frame, Encoder, Repeat, SetParameter};
use std::borrow::Cow;
use std::fs::File;
use std::io;
use util::RGBu8;
use sprite::CompositedFrame;

fn center_buffer_on_square(size: usize, frame: &CompositedFrame) -> Vec<u8> {
    let offset_x = size / 2 - frame.zero_x as usize;
    let offset_y = size / 2 - frame.zero_y as usize;
    let mut v = vec![0; size*size];
    for (i, p) in frame.buffer.iter().enumerate() {
        if *p == 0 {
            continue;
        }
        let (px, py) = (offset_x + i % frame.width as usize, offset_y + i / frame.width as usize);
        v[py*size + px] = *p;
    }
    v
}

fn scale_square(buffer: &[u8], size: usize, scale: usize) -> Vec<u8> {
    let mut new_buffer = Vec::with_capacity(size * size * scale * scale);
    for row in 0..size as usize {
        for _ in 0..scale {
            for px in &buffer[row*size..(row+1)*size] {
                for _ in 0..scale {
                    new_buffer.push(*px);
                }
            }
        }
    }
    new_buffer
}

pub fn write_sprite_to_gif(name: &str, frames: &[CompositedFrame], palette: &[RGBu8]) -> Result<(), io::Error> {
    let rgb_palette = palette.iter().fold(vec![], |mut v, color| {
        v.push(color.0); v.push(color.1); v.push(color.2);
        v
    });

    let dimension = 64u16;
    let scale = 4u16;

    let mut image = File::create(format!("{}.gif", name))?;
    let width = dimension * scale;
    let height = dimension * scale;
    let mut encoder = Encoder::new(&mut image, width, height, &rgb_palette)?;
    encoder.set(Repeat::Infinite)?;
    for frame in frames {
        let mut buffer = center_buffer_on_square(dimension as usize, frame);
        if scale > 1 {
            buffer = scale_square(&buffer, dimension as usize, scale as usize);
        }
        let mut f = Frame::default();
        // f.transparent = Some(0);
        f.delay = (1.0f32 / 60f32 * 1000f32 / 10f32 * frame.duration as f32) as u16;
        f.width = width;
        f.height = height;
        f.buffer = Cow::Borrowed(&buffer);
        encoder.write_frame(&f)?;
    }
    Ok(())
}
