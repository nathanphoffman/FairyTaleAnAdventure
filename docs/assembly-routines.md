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
| `genmini(img_x, img_y)` | Fills `minimap[19*6]` (19 columns x 6 rows of tile numbers = what's on screen) from the sector and region map, with wrapping. Also sets `hero_sector`, adding 256 when `region_num > 7` (inside / underground). C-friendly, port it |
| `mapxy(imx, imy)` | Like `px_to_im` but takes tile coordinates and returns a pointer to the tile byte in `sector_mem` |
| `bigdraw(map_x, map_y)` | Draws the full-screen overview map: 18 x 9 sectors, each sector drawn as 16 x 8 coloured pixels using the `big_colors` byte (terra byte 3), written to the 5 planes and doubled vertically. Sets `secx` and `secy` |
| `maskit(x, y, mod, char)` | Copies a tile's 32-line "shadow" mask (`shadow_mem`, 64 bytes per tile) into the sprite's mask buffer so sprites can pass behind scenery (trees, buildings) |
| `make_mask(src, dest, words, scans, chars)` | For each sprite frame, ANDs all 5 planes together and inverts it, giving a 1-plane mask where colour 31 is transparent. Easy C |
| `prox(x, y)` | Collision test with two probes. Probe 1 is terrain at (x+4, y+2): blocked if the type is 1 or >= 10. Probe 2 is (x-4, y+2): blocked if 1 or >= 8. Returns the blocking type, or 0 if clear |
| `newx(x, dir, speed)`, `newy(y, dir, speed)` | Move a coordinate in one of 8 directions. `xdir`/`ydir` tables hold -2..3 and the result is `(table * speed) >> 1`, masked to 15 bits. `newy` keeps bit 15 of y |
| `wrap(x)` | If bit 14 set, set bit 15, else clear bit 15 (wraps the world coordinate into a signed range) |
| `map_adjust(x, y)` | Camera follow: moves `map_x`/`map_y` toward the hero. If the hero is 70 pixels (24 vertically) from the camera it jumps, if over 20 (10 vertically) it moves 1 pixel per call |
| `page_det(x)` | Looks up which of the page-flip/wipe steps a column belongs to (small table) |
| `decode_mouse` | Decides the walking direction (0-7, 9 = none) from the mouse over the on-screen compass, a joystick, or a keyboard direction key (keys 20-29). Calls `drawcompass` when it changes. Needs a rewrite over SDL input; joystick bits come from `JOY1DAT` |
| `unpack_line(dest)` | IFF ByteRun1 decoder for one scanline (plain copy if uncompressed). Port as is |
| `do_error(n)` | Prints "ERROR n" at (200, 90) |
| `ReadHeader`, `ReadLength` | Read 4-byte IFF chunk id / length. Replace with Rust byte reading |

### Blitter code (hardest to port)

These use the Amiga blitter directly with hand-built register values:

| Routine | What it does | Port approach |
|---------|--------------|---------------|
| `scrollmap(dir)` | Scrolls the 5-plane map one tile step in one of 8 directions using blits with a different offset per direction. `fmain.c` calls it with `dif_x`/`dif_y` of -1, 0, 1 and redraws the new edge with `strip_draw`/`row_draw` | Copy the bitmap with an offset in software, or ignore the scroll trick and just redraw the whole map each frame with `map_draw`. Because the machine is fast, a full redraw is simplest |
| `clear_blit(mem, size)` | Clears a chunk of memory with the blitter | `memset` |
| `save_blit(backsave)` | Before drawing a sprite, copies the background behind it (5 planes) into a save buffer | Not needed if we redraw the full frame every time |
| `mask_blit()` | Builds the combined background mask for the sprite | Software version of the same |
| `shape_blit()` | Draws a sprite through a mask (cookie-cut blit: `D = (A & mask) | (B & ~mask)`) with sub-word horizontal shifting | Software sprite draw per pixel |
| `rest_blit(backsave)` | Restores the saved background after the frame | Not needed with full redraw |

Design note: the original double-buffers two pages and erases and redraws only moved sprites for
speed. A full redraw of 288 x 143 pixels each frame is trivial on modern hardware, which removes the
need for `save_blit`, `rest_blit`, `clear_blit` and most of `scrollmap`. The `sshape` queue and
`backsave` in `fmain.c` can go with it, but the draw order and masking must still be matched so
sprites keep passing behind scenery correctly (`maskit` logic).

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

The track and instrument formats are now documented in `data-formats.md` (Music data). Summary of
the driver: every vblank, add `tempo` to a clock; for each voice with a track, if the clock has
reached the next event time read the next `(command, value)` pair; a note sets the Paula
registers (waveform pointer, length, period, volume); between notes the volume envelope bytes are
read one per tick; there is a 300-unit gap before the next note. `playsample` takes over voice 2
for sound effects. Samples in `image` blocks 920-930 (11 blocks, `sample_mem`).

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
