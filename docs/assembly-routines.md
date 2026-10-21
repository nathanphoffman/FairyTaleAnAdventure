# Assembly routines

Files: `fsubs.asm` (2050 lines), `gdriver.asm` (512), `narr.asm` (519), `MakeBitMap.asm` (123),
`fsupp.asm` (42). Names starting with `_` are callable from the C code. Arguments are pushed on the
stack, last argument first (Aztec C convention), results in `d0`.

## fsupp.asm

| Routine | What it does | Port to |
|---------|--------------|---------|
| `colorplay` | Fills a 32-entry palette with random colours 32 times, 1 tick apart (screen flash effect) | C/Rust: loop calling `rand`, `LoadRGB4`, `Delay` |
| `stillscreen` | Zeros the scroll offsets of the drawing page and flips pages | Rust |
| `skipint` | Reads a key; if it is space sets `skipp` and returns true | Rust |

## fsubs.asm

### Input

| Routine | What it does |
|---------|--------------|
| `HandlerInterface` | Amiga input handler: raw keys into a ring buffer through `keytrans`, ignores key repeats, tracks mouse position (clamped x 5-315, y 147-195), left click in the menu column (x 215-265) becomes a menu letter (`a` + row), a timer event counter that injects a fake "shift up" every 16 ticks. Moves the pointer sprite. |
| `getkey` | Pops one code from the key ring buffer |
| `trapper` | Debug trap handler, drop it |

### Random numbers

`rand` is a simple linear congruential generator: `seed = (seed & 0xFFFF) * 45821 + 1`, result
`ror(seed, 6) & 0x7FFFFFFF`. Note `mulu.w` only multiplies the low 16 bits of the seed. Reproduce
it exactly if we want the same game behaviour. Wrappers: `bitrand(mask)`, `rand2/4/8/64/256`
(mask), `rnd(n)` (low 16 bits of rand mod n). Easy to port.

### Text and drawing

| Routine | What it does |
|---------|--------------|
| `prdec(value, len)` | Convert number to right-aligned decimal text in `numbuf` and print it with `Text` |
| `ion` | Same conversion, no printing |
| `move(x,y)`, `text(str,len)` | Thin wrappers over `Move` and `Text` on the current rastport `rp` |
| `ssp(str)` | Print a string that can contain `XY` escape (byte 128 followed by x/2 and y) to move the cursor. Strings end at a zero byte or a byte >= 128 |
| `placard` | Draws the decorative border pattern on `rp_map` with `Move`/`Draw`, mirrored four ways. Uses `xmod`/`ymod` offset tables. Straightforward to port to C |

### Map and tiles (the performance critical parts)

| Routine | What it does |
|---------|--------------|
| `px_to_im(x,y)` | Given a pixel position in the world, returns the terrain type (0-15) under it. Looks up the tile in the sector map (`map_mem`, `sector_mem`) then the terrain table (`terra_mem`, 4 bytes per tile) and checks a quarter-tile mask bit. Used for all collision and terrain effects. **Port first, it's small and defines how the map works.** |
| `map_draw` | Copies the 19x6 (114) tile characters in `minimap` into the 5 bitplanes. Each character is 16x32 pixels and is stored as 5 planes in `image_mem` (plane stride 16384 bytes, 64 bytes per char per plane). Writes into screen planes with 40 bytes per scanline |
| `strip_draw`, `row_draw` | Same idea for drawing one new column or row of tiles when scrolling |
| `bigdraw`, `maskit`, `mapxy` | Draw big tiles and build sprite masks from the background. **Not yet read in detail.** |
| `ReadHeader`, `ReadLength` | Read 4-byte IFF chunk id / length. Replace with Rust byte reading |
| `MoveSprite` call | Pointer sprite move, see the input handler |

## MakeBitMap.asm

`MakeBitMap(b, depth, width, height)` allocates the bitplanes for a `BitMap`, `UnMakeBitMap(b)`
frees them. Becomes the planar bitmap allocator in Rust.

## narr.asm

Almost entirely data: the story text and messages.

- `event_msg`, `place_msg`, `inside_msg`, `place_tbl`, `inside_tbl`, `speeches`, `placard_text`, `question`.
- Strings are text with embedded control codes (see `ssp`). The tables map map locations to messages.
- Port as a data module (C arrays or a Rust `const`). Easiest done with a script that converts the
  `dc.b` lines to Rust/C strings so we don't retype anything.

## gdriver.asm

The music and sound driver. Four voices on the Amiga's four audio channels.

| Routine | What it does |
|---------|--------------|
| `init_music`, `wrap_music` | Install and remove the vblank and audio interrupt handlers |
| `setscore(t1..t4)`, `playscore(t1..t4)`, `stopscore` | Select and start/stop four track pointers |
| `set_tempo(n)` | Set playback speed |
| `playsample(addr, len, rate)`, `stopsample` | One-shot sound effect on voice 2 (hardware channel 2) |
| `vblank_server` | Runs 50/60 times a second, steps through each voice's note list, applies volume envelopes |

Per voice state (28 bytes): waveform number, envelope number, volume delay, status flags
(`TIE`, `CHORD`, `REST`, `ENDTK`), event start/stop times, volume list pointer, track pointer, track
start and loop stack. Notes are looked up in a period table (`ptable`).

Plan: write the sequencer in Rust as a function called once per tick from the audio callback that
fills a mix buffer from the instrument waveforms in `v6`. Needs the track and instrument formats
fully documented first (see `data-formats.md`).

## Suggested port order

1. `rand` and friends (tiny, must match exactly)
2. `px_to_im` (small, needed for gameplay)
3. `prdec`, `ion`, `ssp`, `placard`, `move`, `text`, `colorplay`, `stillscreen`, `skipint`
4. `MakeBitMap`
5. `narr.asm` data conversion
6. `map_draw`, `strip_draw`, `row_draw`
7. `bigdraw`, `maskit`, `mapxy` (after reading them)
8. Input handler (reimplemented in terms of SDL events)
9. `gdriver.asm`
