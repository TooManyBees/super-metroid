use std::cmp;
use util::{bgr555_rgb888, bgr555_rgbf32};

pub struct CompositedFrame {
    pub buffer: Vec<u8>,
    pub width: u16,
    pub height: u16,
    pub duration: u16,
}

pub struct Sprite {
    frames: Vec<CompositedFrame>,
    palette: Vec<u16>,
}

impl Sprite {
    pub fn new(frames: Vec<CompositedFrame>, palette: Vec<u16>) -> Self {
        Sprite {
            frames: frames,
            palette: palette,
        }
    }

    pub fn width(&self) -> u16 {
        self.frames.iter().fold(0, |width, f| cmp::max(width, f.width))
    }

    pub fn height(&self) -> u16 {
        self.frames.iter().fold(0, |height, f| cmp::max(height, f.height))
    }

    pub fn frames(&self) -> &[CompositedFrame] {
        &self.frames
    }

    pub fn palette888(&self) -> Vec<(u8, u8, u8)> {
        self.palette.iter().map(bgr555_rgb888).collect()
    }

    pub fn palettef32(&self) -> Vec<(f32, f32, f32)> {
        self.palette.iter().map(bgr555_rgbf32).collect()
    }
}

pub struct SpriteView<'a> {
    frames: &'a [CompositedFrame],
    index: usize,
}

impl<'a> SpriteView<'a> {
    pub fn new(sprite: &'a Sprite) -> Self {
        SpriteView {
            frames: sprite.frames(),
            index: 0,
        }
    }

    pub fn frame(&mut self) -> &'a CompositedFrame {
        let f = &self.frames[self.index as usize];
        self.index = (self.index + 1) % self.frames.len();
        f
    }
}
