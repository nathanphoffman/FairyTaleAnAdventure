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

### Still to work out

- Where `cfiles[]` is defined and the full list of sprite sheets (file_id, width, height, count).
- Exact layout of the sector, map and terrain blocks. `px_to_im` in `fsubs.asm` shows how the
  terrain table is indexed: 4 bytes per tile, byte 1 high nibble = terrain type, byte 2 = tile
  mask bits.
- Layout of the instrument file `v6` and the track encoding in `songs`.
- Format of sprite sheets: planar, 5 planes, with a separate mask built at load time by `make_mask`.

## Memory budget in the original

`IMAGE_SZ` is 81K (5 planes x 256 chars x 64 bytes). `SHAPE_SZ` and others are in `fmain.c`. We
don't have the 512K limit, so we can simply allocate what we need.

## Endianness

All multi-byte values in these files are big-endian (the 68000 reads them directly). In Rust,
use `u16::from_be_bytes` and friends when reading. Struct dumps like the save file need the
struct layout figured out and read field by field, not cast.
