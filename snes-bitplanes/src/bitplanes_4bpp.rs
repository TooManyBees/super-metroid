use core::slice::Chunks;

/// An iterator over 4-bits-per-pixel bitplanes.
///
/// Accepts a slice of bytes encoded in bitplanes and decoded `u8`s.
/// The length of input bytes must be evenly divisible by 32. For every
/// 32 bytes consumed, this iterator yields 64 decoded bytes.
/// (Conceptually, it's an 8x8 tile.)
///
/// The 4 most significan bits of each decoded byte will always be 0.
#[derive(Debug)]
pub struct Bitplanes<'a> {
    chunks: Chunks<'a, u8>,
}

impl<'a> Iterator for Bitplanes<'a> {
    type Item = [u8; 64];

    fn next(&mut self) -> Option<Self::Item> {
        self.chunks.next()
            .map(|chunk| {
                let (planes01, planes23) = chunk.split_at(16);
                let mut result = [0; 64];
                let mut cursor = 0;
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
                        result[cursor] = px;
                        cursor += 1;
                    }
                }
                result
            })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.chunks.size_hint()
    }
}

impl<'a> Bitplanes<'a> {
    /// Constructs a new `Bitplanes` from a slice of bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use snes_bitplanes::Bitplanes;
    /// # fn main() {
    ///
    /// let bytes = [0u8; 128];
    ///
    /// let tiles: Vec<_> = Bitplanes::new(&bytes[0..64]).collect();
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the length of the slice is not evenly divisible by 32.
    pub fn new(bytes: &'a [u8]) -> Bitplanes<'a> {
        assert!(bytes.len() % 32 == 0, "Byte slice doesn't fit into 32-byte tiles");
        Bitplanes {
            chunks: bytes.chunks(32),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Bitplanes;

    #[test]
    fn snes_mode_4_4bpp() {
        let encoded: &[u8] = &[
            0b00101110, // Bitplane 0
            0b00100000, // Bitplane 1
            0b11101001, // Bitplane 0
            0b10010101, // Bitplane 1
            0b11111101, // ...
            0b10101011,
            0b01000111,
            0b00010101,
            0b00110000,
            0b01011100,
            0b10011111,
            0b00011101,
            0b10010110,
            0b01011010,
            0b00010101,
            0b00101110,

            0b01000000, // Bitplane 2
            0b00001111, // Bitplane 3
            0b00010001, // Bitplane 2
            0b11111011, // Bitplane 3
            0b01000001, // ...
            0b10011100,
            0b10000100,
            0b11110110,
            0b10110000,
            0b00011000,
            0b00100111,
            0b00001001,
            0b11101000,
            0b00101010,
            0b00010001,
            0b10000011,
        ];

        let expected = [
            0b0000, 0b0100, 0b0011, 0b0000, 0b1001, 0b1001, 0b1001, 0b1000,
            0b1011, 0b1001, 0b1001, 0b1110, 0b1001, 0b0010, 0b1000, 0b1111,
            0b1011, 0b0101, 0b0011, 0b1001, 0b1011, 0b1001, 0b0010, 0b0111,
            0b1100, 0b1001, 0b1000, 0b1010, 0b0000, 0b1111, 0b1001, 0b0011,
            0b0100, 0b0010, 0b0101, 0b1111, 0b1010, 0b0010, 0b0000, 0b0000,
            0b0001, 0b0000, 0b0100, 0b0011, 0b1011, 0b0111, 0b0101, 0b1111,
            0b0101, 0b0110, 0b1100, 0b0011, 0b1110, 0b0001, 0b1011, 0b0000,
            0b1000, 0b0000, 0b0010, 0b0101, 0b0010, 0b0011, 0b1010, 0b1101,                                     
        ];

        let mut decoded = Bitplanes::new(encoded);

        // // The delights of working with fixed length arrays :|

        let some_decoded_tile = decoded.next();
        assert!(some_decoded_tile.is_some());
        assert_eq!(&expected[..], &some_decoded_tile.unwrap()[..]);
        assert!(decoded.next().is_none());
    }
}
