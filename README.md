# "Soup e'er met," Roy'd

Ripping assets out of the finest videogame<sup>[1](#videogame)</sup> ever
created by humans. This project consists of several crates in a workspace:

* **`sm`**<br>
  The main library for ripping assets out of the ROM.

* **`snes-bitplanes`**<br>
  Library for decoding SNES graphics stored in bitplane formats.
  [published to crates.io](https://crates.io/crates/snes-bitplanes)

* **`proc-samus`**<br>
  Contains procedural macros that precompute Samus assets and palettes using
  the `sm` crate.

* **`viewer`**<br>
  A command line utility that uses `piston_window` to view creature and Samus
  assets, either as a sprite sheet or as an animated sprite. Because it's mostly
  a debugging/exploratory tool, the binary `include_bytes!`s the entire 3MB ROM
  for convenience.

* **`static_viewer`**<br>
  Similar to `viewer`, but is built with `proc-samus` macros to embed the
  precomputed assets into the binary, so that is has no dependency on the ROM
  or on the `sm` library.

Using this project depends on having the Super Metroid ROM named
`Super Metroid (Japan, USA) (En,Ja).sfc` (not included, duh) in a folder
called `data` relative to the base of the workspace.

*<sup>1</sup><a name="videogame">fight me</a>*
