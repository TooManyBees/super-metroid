pub struct Sprite {
    frames: Vec<CompositedFrame>,
    index: usize,
}

impl Sprite {
    pub fn new(frames: Vec<CompositedFrame>) -> Self {
        Sprite {
            frames: frames,
            index: 0,
        }
    }

    pub fn frame(&mut self) -> &CompositedFrame {
        let f = &self.frames[self.index as usize];
        self.index = (self.index + 1) % self.frames.len();
        f
    }

    pub fn width(&self) -> u16 {
        self.frames.iter().max_by_key(|f| f.width).unwrap().width
    }

    pub fn height(&self) -> u16 {
        self.frames.iter().max_by_key(|f| f.height).unwrap().height
    }
}

pub struct CompositedFrame {
    pub buffer: Vec<u8>,
    pub width: u16,
    pub height: u16,
    pub duration: u16,
}
