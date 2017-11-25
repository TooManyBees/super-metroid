pub struct Sprite {
    frames: Vec<CompositedFrame>,
    index: usize,
    time: u16,
}

impl Sprite {
    pub fn new(frames: Vec<CompositedFrame>) -> Self{
        Sprite {
            frames: frames,
            index: 0,
            time: 0,
        }
    }

    pub fn frame(&mut self) -> &CompositedFrame {
        if self.time >= self.frames[self.index].duration {
            self.time = 1;
            self.index = (self.index + 1) % self.frames.len();
            &self.frames[self.index]
        } else {
            self.time += 1;
            &self.frames[self.index as usize]
        }
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
