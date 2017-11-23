// http://fdwr.tripod.com/docs/snesgfx.txt
use std::iter::Map;
use std::slice::Chunks;

pub struct Bitplanes<'a> {
    chunks: Chunks<'a, u8>,
}

impl<'a> Iterator for Bitplanes<'a> {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        self.chunks.next()
            .map(|chunk| {
                let (planes01, planes23) = chunk.split_at(16);
                let mut result = Vec::with_capacity(64);
                for (bytes01, bytes23) in planes01.chunks(2).zip(planes23.chunks(2)) {
                    for n in (0..8).rev() {
                        let mask = 1 << n;
                        let mut px = 0;

                        if bytes23[1] & mask > 0 {
                            px += 1;
                        }
                        px <<= 1;
                        if bytes23[0] & mask > 0 {
                            px += 1;
                        }
                        px <<= 1;
                        if bytes01[1] & mask > 0 {
                            px += 1;
                        }
                        px <<= 1;
                        if bytes01[0] & mask > 0 {
                            px += 1;
                        }
                        result.push(px);
                    }
                }
                result
            })
    }
}

impl<'a> Bitplanes<'a> {
    pub fn new(bytes: &'a [u8]) -> Bitplanes<'a> {
        assert!(bytes.len() % 32 == 0, "Byte slice doesn't fit into 32-byte tiles");
        Bitplanes {
            chunks: bytes.chunks(32),
        }
    }
}
