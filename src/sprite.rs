// use std::cmp;
use util::{bgr555_rgb888, bgr555_rgbf32};

pub struct CompositedFrame {
    pub buffer: Vec<u8>,
    pub width: u16,
    pub height: u16,
    pub zero_x: u16,
    pub zero_y: u16,
    pub duration: u16,
}

pub struct Sprite<'a> {
    frames: Vec<CompositedFrame>,
    palette: &'a [u16],
}

impl<'a> Sprite<'a> {
    pub fn new(frames: Vec<CompositedFrame>, palette: &'a [u16]) -> Self {
        Sprite {
            frames: frames,
            palette: palette,
        }
    }

    // pub fn width(&self) -> u16 {
    //     let (l, r) = self.frames.iter().fold((0, 0), |(l, r), f| {
    //         (cmp::max(l, f.zero_x), cmp::max(r, f.width - f.zero_x)) // SHOW ME YOUR MOVES
    //     });
    //     l + r
    // }

    // pub fn height(&self) -> u16 {
    //     let (t, b) = self.frames.iter().fold((0, 0), |(t, b), f| {
    //         (cmp::max(t, f.zero_y), cmp::max(b, f.height - f.zero_y))
    //     });
    //     t + b
    // }

    // pub fn zero(&self) -> (u16, u16) {
    //     self.frames.iter().fold((0, 0), |(x, y), f| {
    //         (cmp::max(x, f.zero_x), cmp::max(y, f.zero_y))
    //     })
    // }

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
