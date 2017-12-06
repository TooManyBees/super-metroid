This is documentation for the `snes-bitplanes` crate.

The Super NES includes stores its graphics in bitplanes,
a packed format in which the bits representing a specific
pixel are spread across multiple bytes in the same bit
position.

For example, 2-bit-per-pixel data stored as bitplanes
might have the byte representation:
```rust
00101110 //  0, bitplane 1
01100101 //  1, bitplane 2
11101001 //  2, bitplane 1
10010101 //  3, bitplane 2
// ...
00010101 // 14, bitplane 1
00101110 // 15, bitplane 2
```
The initial decoded values are `00`, `10`, `11`, `00`, `01`, `11`, `01`, `10`.
In total, 2bpp data will inflate to 4 times its original size
(because `Bitplanes` iterators yield bytes themselves).

The Super NES is little-endian, so the leftmost bits represent
the earliest decoded bytes. Also note that the second bitplane
is the more significant bit in the output.

# Usage

```rust
let bitplanes_data = vec![0u8; 128]; // Extremely boring data
let decoded: Vec<_> = Bitplanes::new(&bitplanes_data).collect();
```

Currently only 4-bits-per-pixel (16 color) bitplanes are decodable with
this crate.

# Thanks
This crate would not be possible without the research of others,
notably
* FDwR (Frank Dwayne) [/snesgfx.txt](http://fdwr.tripod.com/docs/snesgfx.txt) 1998
* Qwertie (David Piepgrass) [/snesdoc.html](https://emu-docs.org/Super%20NES/General/snesdoc.html#GraphicsFormat) 1998