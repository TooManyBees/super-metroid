// http://fdwr.tripod.com/docs/snesgfx.txt

pub struct Bitplanes<'a> {
    v: &'a [u8],
    bytes: &'a [u8],
    mask: u8,
}

impl<'a> Iterator for Bitplanes<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        // mask value of 0 signals indicates that we need to
        // fetch the next bytes and start the cycle over
        if self.mask == 0 {
            if self.v.is_empty() {
                return None;
            }

            let (bytes, rest) = self.v.split_at(4);
            self.bytes = bytes;
            self.v = rest;
            self.mask = 0b10000000;
        }

        let mut result = 0u8;
        for (n, byte) in self.bytes.iter().enumerate() {
            if (byte & self.mask) > 0 {
                result += 1 << (3 - n);
            }
        }

        self.mask >>= 1;
        Some(result)
    }
}

static nothing: [u8; 0] = [];

impl<'a> Bitplanes<'a> {
    pub fn new(bytes: &'a [u8]) -> Bitplanes<'a> {
        Bitplanes {
            v: bytes,
            bytes: &nothing,
            mask: 0,
        }
    }
}
