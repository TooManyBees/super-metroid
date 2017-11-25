use gif::{Frame as GifFrame, Encoder, Repeat, SetParameter};
use std::borrow::Cow;
use std::fs::File;
use std::io;
use super::{CompositedFrame, RGBu8};

pub fn write_sprite_to_gif(name: &str, frames: &[CompositedFrame], palette: &[RGBu8]) -> Result<(), io::Error> {
    let rgb_palette = palette.iter().fold(vec![], |mut v, color| {
        v.push(color.0); v.push(color.1); v.push(color.2);
        v
    });
    let mut image = File::create("ebi.gif")?;
    let mut encoder = Encoder::new(&mut image, 32, 53, &rgb_palette)?;
    encoder.set(Repeat::Infinite);
    for frame in frames {
        let mut f = GifFrame::default();
        f.transparent = Some(0);
        f.delay = (1.0f32 / 60f32 * 1000f32 / 10f32 * frame.duration as f32) as u16;
        f.width = frame.width;
        f.height = frame.height;
        f.buffer = Cow::Borrowed(&*frame.buffer);
        encoder.write_frame(&f)?;
    }
    Ok(())
}
