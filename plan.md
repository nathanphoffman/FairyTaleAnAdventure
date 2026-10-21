# Porting Plan

Goal: run The Faery Tale Adventure natively on Linux using Rust, with the original C logic reused
at first and ported to Rust gradually. Original source (reference) is in `faery-tale-amiga/`.

## What we're porting

- About 7,000 lines of Aztec C and 3,000 lines of 68000 assembly.
- Game data is included in `faery-tale-amiga/game/` (disk `image`, songs, fonts, palettes, pictures).
- Amiga-specific parts that must be replaced:
  - Graphics: custom chips, bitplanes, `BltBitMap`, hardware sprites, copper/viewports, palettes.
  - Disk: raw floppy reads via trackdisk (`hdrive.c` already has a file-based mode using `image`).
  - Sound: Amiga music driver (`gdriver.asm`).
  - Input and timing: Amiga input handling and vertical-blank interrupt.
- Data files are big-endian and the code assumes 32-bit pointers.

## Phases

Each phase should end with something that builds and runs.

### Phase 0 - Project setup
- [ ] Install Rust, gcc, `libsdl2-dev`.
- [ ] `cargo init`, add `sdl2`, `cc` and `bindgen` (if needed) dependencies.
- [ ] Open an SDL window and draw a test pattern.
- [ ] Write a small tool that dumps the layout of `game/image` so we understand the data.

### Phase 1 - Understand the code and data
- [x] Read `fmain.c` and list what it calls from the Amiga libraries (the main ones: `AllocMem`,
      `SetAPen`, `Read`, `Open`, `Text`, `SetRGB4`, `DoIO`, `BltBitMap`, `SetFont`, `LoadRGB4`).
- [x] Document the data file formats (`image`, `p1a`..`p3b`, `page0`, `songs`, `v6`, `winpic`,
      `hiscreen`, fonts).
- [x] Document the assembly routines in `fsubs.asm`, `fsupp.asm`, `MakeBitMap.asm` and
      `gdriver.asm`: what each one does and who calls it.
- [x] Write findings into `docs/` (first pass done; open questions listed at the end of each file).

### Phase 2 - Get the C compiling with gcc
- [ ] Copy the C files into `csrc/` (leave the originals untouched).
- [ ] Add stand-in Amiga headers and types (`stubs/amiga.h`) so the code compiles.
- [ ] Fix old-style C (K&R declarations, implicit int, prototypes).
- [ ] Fix 32-bit assumptions: use fixed-size types (`uint16_t`, `uint32_t`), no pointers stored in `int`.
- [ ] Goal: everything compiles and links with stubbed Amiga calls, even if nothing works yet.

### Phase 3 - Rust platform layer
Rust implements the Amiga calls the C code makes, exposed with `extern "C"`.
- [ ] Memory: `AllocMem`/`FreeMem` as plain allocation.
- [ ] Files: `Open`/`Read`/`Write`/`Seek`/`Close` on normal files, with case-insensitive paths.
- [ ] Disk: route all trackdisk reads to the `image` file (reuse the `hdrive.c` approach).
- [ ] Bitmaps and rastports: planar bitmaps in memory, `InitBitMap`, `AllocRaster`.
- [ ] Drawing: `BltBitMap`, `SetAPen`, `Move`, `Draw`, `Text`, `SetFont`, `ScrollRaster`.
- [ ] Palette: `SetRGB4`, `LoadRGB4`.
- [ ] Display: convert planar bitmaps to a texture and present each frame with SDL2.
- [ ] Sprites: `GetSprite`, `ChangeSprite`.
- [ ] Input and timing: keyboard and joystick via SDL; 50/60 Hz tick standing in for the vertical blank.

### Phase 4 - Rewrite the assembly
Rewrite in C first (easy to compare), then Rust later with the rest of the game.
- [ ] `MakeBitMap.asm`
- [ ] `fsupp.asm`
- [ ] `fsubs.asm` (largest: map and rendering routines)
- [ ] `narr.asm` (story text data, likely just becomes a data table)
- [ ] `gdriver.asm` (music driver, see Phase 5)

### Phase 5 - Sound
- [ ] Work out the song and sample format in `game/songs`.
- [ ] Write a player in Rust that mixes the channels and plays through SDL audio.
- [ ] Add sound effects.

### Phase 6 - First playable build
- [ ] Title screen shows correctly.
- [ ] Game starts, the map renders and scrolling works.
- [ ] Player moves, input works.
- [ ] Combat, items, dialogue, save/load.
- [ ] Music and effects.
- [ ] Fix byte-order bugs as they show up.

### Phase 7 - Port C to Rust
Move modules one at a time, with the game runnable after each one. Rough order, easiest first:
- [ ] `text.c`, `terrain.c`, `iffsubs.c`
- [ ] `hdrive.c`, `rtrack.c`, `mtrack.c`
- [ ] `fmain2.c`
- [ ] `fmain.c` (largest, split into modules as it moves)

### Phase 8 - Polish
- [ ] Window scaling and fullscreen, correct aspect ratio.
- [ ] Config file and key remapping.
- [ ] Remove leftover Amiga stand-ins.
- [ ] Packaging and how to supply game data.

## Risks

- The graphics code is tied to Amiga bitplanes and hardware tricks (split screens, sprites, copper
  colour changes). Some of it may need a redesign instead of a straight translation.
- The assembly has few comments and the author doesn't remember details, so we'll be reading it
  closely.
- Hidden 32-bit and big-endian assumptions will cause crashes and corrupt data.
- The legal status of the game data is unclear, so we shouldn't distribute it.

## Reference

- Original source: https://github.com/viridia/faery-tale-amiga
- SDL port attempt: https://github.com/XarkLabs/faery-tale-amiga
- Rust and research branches: https://github.com/bacomatic/faery-tale-rs (`original-bugfix`, `research`)
