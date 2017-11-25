use gif::{Frame, Encoder, Repeat, SetParameter};
use std::borrow::Cow;
use std::fs::File;
use std::io;
use util::RGBu8;
use sprite::CompositedFrame;

pub fn write_sprite_to_gif(name: &str, frames: &[CompositedFrame], palette: &[RGBu8]) -> Result<(), io::Error> {
    let rgb_palette = palette.iter().fold(vec![], |mut v, color| {
        v.push(color.0); v.push(color.1); v.push(color.2);
        v
    });
    let mut image = File::create(format!("{}.gif", name))?;
    let width = frames.iter().max_by_key(|f| f.width).map(|f| f.width).unwrap_or(128);
    let height = frames.iter().max_by_key(|f| f.height).map(|f| f.height).unwrap_or(128);
    let mut encoder = Encoder::new(&mut image, width, height, &rgb_palette)?;
    encoder.set(Repeat::Infinite)?;
    for frame in frames {
        let mut f = Frame::default();
        f.transparent = Some(0);
        f.delay = (1.0f32 / 60f32 * 1000f32 / 10f32 * frame.duration as f32) as u16;
        f.width = width;
        f.height = height;
        f.buffer = Cow::Borrowed(&frame.buffer);
        encoder.write_frame(&f)?;
    }
    Ok(())
}
