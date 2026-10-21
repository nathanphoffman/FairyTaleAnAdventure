# Game data files

All files are in `faery-tale-amiga/game/`. Everything is big-endian.

## Loose files

| File | Size | What it is | Loaded by |
|------|------|-----------|-----------|
| `page0`, `winpic` | 30K, 26K | IFF ILBM pictures, 320x200, 5 planes, ByteRun1 compressed | `unpackbrush()` in `iffsubs.c` |
| `hiscreen` | 4.7K | IFF ILBM, 640x57, 4 planes. The text panel background | `unpackbrush()` |
| `p1a`/`p1b`, `p2a`/`p2b`, `p3a`/`p3b` | 5-10K each | IFF ILBM story intro pictures (5 planes). `copypage()` blends pairs onto the intro screen | `unpackbrush()` |
| `songs` | 6K | Music scores. Repeated records of a 4 byte length (in words) followed by that many words of track data. Up to 28 tracks (4 voices x 7 songs) | `read_score()` in `fmain2.c` |
| `v6` | 4.6K | Instrument data (waveforms and volume envelopes) for the music driver | `fmain.c` around line 931 |
| `fonts/Amber/9` | | Amiga bitmap font (a Hunk executable-style file, loaded with `LoadSeg`), `amber.font` is its descriptor | `fmain.c` line 774 |
| `*.faery` (`C.faery`, `E.faery`) | 1.2K | Save game slots. Raw dump of game variables | `fmain2.c` around line 1392 |
| `fmain`, `FMain.info`, `game.info` | | The original Amiga executable and icons. Not needed. |

IFF reading is already simple: check `FORM`, `ILBM`, then chunks `BMHD`, `CMAP`, `GRAB`, `CAMG`,
`CRNG`, `BODY`. Pixel data is stored one bitplane per row, each row ByteRun1 compressed. There are
standard IFF libraries, but the 100-line version in `iffsubs.c` is enough to port.

## The `image` file

A 901,120 byte dump of the original floppy: 1760 sectors of 512 bytes (880K). It's created by
`copyimage` (`Install-FTA`). Nearly all game assets live in here and are read by block number
through `load_track_range(block, count, buffer, slot)`. Offset in the file is `block * 512`.

Known block ranges so far (from `fmain.c`, `fmain2.c`):

| Blocks | Count | Contents | Loaded to |
|--------|-------|----------|-----------|
| 149 + `terra1`/`terra2` | 1 + 1 | Terrain tables, 512 bytes each (`TERRA_BLOCK` = 149) | `terra_mem`, `terra_mem+512` |
| `sector` field of region table | 64 | Sector data (the 256-char "sector" tiles, 128 bytes each) | `sector_mem` |
| `region` field of region table | 8 | Map: region layout, one byte per sector | `map_mem` |
| `image[0..3]` + 0/8/16/24/32 | 8 each, 5 planes | Landscape tile graphics, 5 planes, 4 images (blocks listed per region) | `image_mem` |
| 880 | 1 | Used for disk-identification / save-disk check (`fmain2.c:1430`) | |
| 896 | 24 | Shadow / mask memory | `shadow_mem` |
| 920 | 11 | Sound effect samples | `sample_mem` |
| `cfiles[n].file_id` | `numblocks` | Character and object sprite sheets (player, enemies, raft, etc.) | `seq_list[...].location` |

### Region table (`file_index[10]` in `fmain.c:615`)

Each of 10 regions has `{ image[4], terra1, terra2, sector, region, setchar }`:

| Region | image blocks | terra | sector | region | note |
|--------|--------------|-------|--------|--------|------|
| F1 snowy | 320,480,520,560 | 0,1 | 32 | 160 | |
| F2 witch wood | 320,360,400,440 | 2,3 | 32 | 160 | |
| F3 swampy | 320,360,520,560 | 2,1 | 32 | 168 | |
| F4 plains/rocks | 320,360,400,440 | 2,3 | 32 | 168 | |
| F5 desert | 320,480,520,600 | 0,4 | 32 | 176 | |
| F6 bay/city/farms | 320,280,240,200 | 5,6 | 32 | 176 | |
| F7 volcanic | 320,640,520,600 | 7,4 | 32 | 184 | |
| F8 forest | 320,280,240,200 | 5,6 | 32 | 184 | |
| F9 building interiors | 680,720,800,840 | 8,9 | 96 | 192 | |
| F10 dungeons/caves | 680,760,800,840 | 10,9 | 96 | 192 | |

Regions 0-7 are the outdoor world (a grid), 8 and 9 are inside and underground.

## Map data layout

World coordinates are 15-bit (0 to 0x7FFF) and wrap around. One map tile ("image" or "char") is
16 pixels wide and 32 tall, so `imx = x/16` and `imy = y/32`.

**Sectors.** A sector is 16 x 8 tiles (128 bytes, one byte per tile = tile number 0-255). Up to
256 sectors are loaded into `sector_mem` (32K, from 64 blocks at `need.sector`).

**Region map.** `map_mem` (4096 bytes, from 8 blocks at `need.region`) is a 128 x 32 grid of
sector numbers, one byte each. `xreg` and `yreg` give the grid origin of the loaded region in
sectors. For a tile (imx, imy):

```
secx = imx / 16 - xreg          (wraps; see px_to_im for the odd wrap trick)
secy = imy / 8  - yreg          (clamped to 0..31)
sec_num   = map_mem[secy * 128 + secx + xreg]
tile      = sector_mem[sec_num * 128 + (imy & 7) * 16 + (imx & 15)]
```

**Terrain table** (`terra_mem`, 1024 bytes = 256 tiles x 4 bytes). Built by `terrain.c`, so the
four bytes are `maptag`, `terrain`, `tiles`, `big_colors`:

| Byte | Meaning |
|------|---------|
| 0 | `maptag`: image characteristics (not yet traced) |
| 1 | `terrain`: high nibble = terrain type (0-15), low nibble = a second value (not yet traced) |
| 2 | `tiles`: 8 bit mask saying which of the tile's 8 sub-areas are solid. `px_to_im` picks the bit from pixel bits x:3, y:3, y:4 |
| 3 | `big_colors`: colour (0-31) this tile gets on the overview map, used by `bigdraw` |

Terrain types seen in code: 0 open, 1 blocked, 2-7 and up to 9 passable with effects (water,
etc., not yet traced), `prox` treats 1 and >= 10 (and in `px_to_im`-style checks >= 8) as blocked.
Two blocks are loaded per region (`TERRA_BLOCK + terra1` into bytes 0-511 for tiles 0-127,
`TERRA_BLOCK + terra2` into bytes 512-1023 for tiles 128-255).

**Tile graphics** (`image_mem`, 5 planes x 16384 bytes). Each plane holds 256 tiles x 64 bytes
(16 pixels = 2 bytes wide, 32 lines). Tiles are in four groups of 64. Each group `i` comes from
40 blocks at `need.image[i]`: 5 chunks of 8 blocks (4096 bytes), one chunk per bitplane, placed at
`image_mem + plane * 16384 + group * 4096`. So tile `t` plane `p` is at `p * 16384 + t * 64`.

## Sprite sheets (`cfiles[]` in `fmain2.c:643`)

Table fields are `{ width_in_words, height_lines, frame_count, numblocks, slot, start_block }`.
One frame is 5 bitplanes stored back to back, each plane `width * height` words. Total bytes
loaded = `numblocks * 512` = frames x 5 x (width x height x 2). After loading, `make_mask` builds a
one-plane mask per frame (AND of all 5 planes, inverted, so colour 31 is transparent) placed right
after the pixel data.

| # | Name | w (words) | h | frames | blocks | slot | start block |
|---|------|-----------|---|--------|--------|------|-------------|
| 0 | julian | 1 | 32 | 67 | 42 | PHIL | 1376 |
| 1 | phillip | 1 | 32 | 67 | 42 | PHIL | 1418 |
| 2 | kevin | 1 | 32 | 67 | 42 | PHIL | 1460 |
| 3 | objects | 1 | 16 | 116 | 36 | OBJECTS | 1312 |
| 4 | raft | 2 | 32 | 2 | 3 | RAFT | 1348 |
| 5 | turtle | 2 | 32 | 16 | 20 | CARRIER | 1351 |
| 6 | ogre | 1 | 32 | 64 | 40 | ENEMY | 960 |
| 7 | ghost | 1 | 32 | 64 | 40 | ENEMY | 1080 |
| 8 | dknight/spiders | 1 | 32 | 64 | 40 | ENEMY | 1000 |
| 9 | necromancer/farmer | 1 | 32 | 64 | 40 | ENEMY | 1040 |
| 10 | dragon | 3 | 40 | 5 | 12 | DRAGON | 1160 |
| 11 | bird | 4 | 64 | 8 | 40 | CARRIER | 1120 |
| 12 | snake/salamander | 1 | 32 | 64 | 40 | ENEMY | 1376 (same block as julian; looks like a placeholder, check) |
| 13 | wizard/priest | 1 | 32 | 8 | 5 | SETFIG | 936 |
| 14 | royal set | 1 | 32 | 8 | 5 | SETFIG | 931 |
| 15 | bartender | 1 | 32 | 8 | 5 | SETFIG | 941 |
| 16 | witch | 1 | 32 | 8 | 5 | SETFIG | 946 |
| 17 | ranger/beggar | 1 | 32 | 8 | 5 | SETFIG | 951 |

The brother's sheet is `cfiles[brother-1]`. Enemy sheet is `cfiles[actor_file]`, the set-figure
sheet is `cfiles[set_file]`. All sprites are 16 pixels wide except the raft, turtle, dragon and bird.

## Music data

**`songs`:** up to 28 tracks (4 voices x 7 songs). Each is a 4 byte length in words, then that
many words of events. Each event is 2 bytes `(command, value)`:

| Command | Meaning |
|---------|---------|
| 0-127 | A note. Command = note number (index into `ptable`, 6 notes per row of 12 half steps...). Value bits 0-5 = duration (index into `notevals`, 64 entries). Value bit 7 = chorded (ignored), bit 6 = tied (kept but unused) |
| 128 | Rest, value as above |
| 129 | Set instrument, value & 15 indexes `new_wave[]` in `fmain.c` to pick a waveform number |
| 0x90 | Set tempo to value (non-standard SMUS extension) |
| 255 | End of track. Value 0 = stop the voice, non-zero = loop to start |

This is the Electronic Arts SMUS score format, flattened.

**Timing:** each vertical blank adds `tempo` (initially 150) to a clock. A note's length in clock
units comes from `notevals`; its sound is cut off 300 units before the end (a small gap) unless
shorter.

**`v6`:** `open_all` reads 1024 bytes (8 waveforms of 128 bytes, signed 8-bit samples) from
offset 0, then calls `Seek(file, 1024, 0)`. Mode 0 means "from the current position", so this
skips another 1024 bytes, and the volume envelopes (10 x 256 bytes) come from offset 2048. Each
envelope: byte 0 is the starting volume, the following bytes are the volume on each tick, and a
negative byte means "stop changing". 2048 + 2560 = 4608 of the file's 4628 bytes are used.

Waveform trick: each 128-byte waveform holds one octave-stack. The `ptable` second column is a
start offset (0, 16, 24, 28 in units of 4 bytes): high notes play a shorter copy of the cycle
stored later in the 128 bytes, so one table covers several octaves. The first column is the
Amiga hardware period. Frequency = 3546895 / period (PAL clock) or 3579545 / period (NTSC), then
divided by the number of samples in the cycle used.

## Save games (`savegame` in `fmain2.c:1474`)

Saved as raw memory dumps of game variables in a fixed order (80 bytes starting at `map_x` and the
variables that follow it in memory, `region_num`, 6 bytes at `anix`, then `anim_list`, extents,
global objects, the per-region object tables). This depends on C variable layout in memory, so the
port must pick an explicit order and field list. We do not need to read the original saves.

## Copy protection

`cpytest()`, `prot2()`, `IsHardDrive()` and the block 880 read check that the original disk is
present. Not needed. Remove when porting.

## Memory budget in the original

`IMAGE_SZ` is 81K (5 planes x 256 chars x 64 bytes). `SHAPE_SZ` and others are in `fmain.c`. We
don't have the 512K limit, so we can simply allocate what we need.

## Endianness

All multi-byte values in these files are big-endian (the 68000 reads them directly). In Rust,
use `u16::from_be_bytes` and friends when reading. Struct dumps like the save file need the
struct layout figured out and read field by field, not cast.
