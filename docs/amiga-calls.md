# Amiga system calls used by the game

Counts come from grepping the C source in `faery-tale-amiga/`. "Provide" means the Rust platform
layer (Phase 3) has to supply it, or the call gets removed.

## Memory

| Call | Used in | Plan |
|------|---------|------|
| `AllocMem`, `FreeMem` | fmain, fmain2, iffsubs, text, form, mtrack, rtrack | Provide as plain malloc/free (zeroed when `MEMF_CLEAR`). Chip vs fast RAM no longer matters. |
| `AllocRaster`, `FreeRaster` | fmain, fmain2 | Provide: allocate one bitplane of w x h bits. |

## Files

| Call | Plan |
|------|------|
| `Open`, `Close`, `Read`, `Write`, `Seek` | Provide on normal files. Mode `1005` is read, `1006` is write. |
| `Lock`, `UnLock` | Used in `hdrive.c` to check whether `image` exists and in `fmain2.c` to check for `df0:`/`df1:` (save disks). Replace with plain file checks. |
| `LoadSeg`, `UnLoadSeg` | Used to load the font file (`fonts/Amber/9`). Replace with our own font loader. |
| `CurrentDir` | Drop. |

## Disk (trackdisk)

| Call | Plan |
|------|------|
| `OpenDevice`, `CloseDevice`, `CreatePort`, `CreateExtIO`, `DoIO`, `SendIO`, `WaitIO`, `CheckIO` | Replace. All game assets are read as 512-byte blocks via `load_track_range(block, count, buffer, slot)`. `hdrive.c` already has a mode that reads these blocks from a file called `image` instead. We make that the only mode and read synchronously (or on a thread). |
| `TD_MOTOR` | Drop. |
| Save games to `df1:A.faery` | Replace with a file in a save directory. |

## Graphics

| Call | Used for | Plan |
|------|----------|------|
| `InitBitMap` | Set up planar bitmaps (5 planes game view, 4 planes text view) | Provide with a planar bitmap struct. |
| `BltBitMap` | Main copy routine (minterm `0xC0` = plain copy) | Provide. Handle planes, minterm and mask. |
| `SetRast`, `RectFill` | Clear / fill | Provide. |
| `SetAPen`, `SetBPen`, `SetDrMd` | Pen colours and draw mode | Provide. |
| `Move`, `Draw` | Lines (used by `placard` borders in `fsubs.asm`) | Provide. |
| `Text`, `SetFont`, `OpenFont`, `OpenDiskFont`, `CloseFont` | Text rendering | Provide with our own bitmap font loader. |
| `ScrollRaster` | Scrolling the text area | Provide. |
| `AreaMove`, `AreaDraw`, `AreaEnd`, `InitArea`, `InitTmpRas` | Filled polygons (fmain2.c) | Provide a polygon fill. |
| `SetRGB4`, `LoadRGB4`, `GetRGB4` | Palette (12-bit colours, 4 bits per channel) | Provide. Expand 4-bit to 8-bit by multiplying by 17. |
| `OwnBlitter`, `DisownBlitter`, `WaitBlit` | Direct blitter access | Provide as no-ops. |
| `InitRastPort`, `InitView`, `InitVPort`, `MakeVPort`, `MrgCop`, `LoadView`, `WaitBOVP` | Display setup: the screen is built from two viewports (play field on top, text panel below) | Replace. This is the biggest design change. We keep two bitmaps and compose them into one window texture. |
| `GetSprite`, `ChangeSprite`, `FreeSprite`, `MoveSprite` | Hardware mouse-pointer sprite | Replace with a software cursor drawn on top, or the system cursor. |
| `OpenScreen`, `OpenWindow`, `CloseScreen`, `CloseWindow`, `SetMenuStrip`, `GetMsg`, `ReplyMsg`, `Wait` | Used in `text.c` and `form.c` only. | These look like tool programs and the startup requester, not the game loop. Check before porting. |

## Timing and system

| Call | Plan |
|------|------|
| `Delay(ticks)` | Called ~26 times. 1 tick = 1/50 s. Provide as sleep, but it must keep the window responsive and allow skip with space (see `skipint`). |
| `OpenLibrary`, `CloseLibrary` | Provide as no-ops that return a dummy pointer. |
| `FindTask` | Used for process priority. Drop. |
| `rand` | Game replaces it with its own `_rand` in `fsubs.asm` (see assembly notes). |

## Input

The game installs an input handler (`_HandlerInterface` in `fsubs.asm`) that receives raw keyboard,
mouse and timer events. It translates keys to ASCII-ish codes through the `keytrans` table, puts
them in a 128 byte ring buffer (`keybuf`), tracks the mouse position, and turns left clicks on the
menu area into menu key codes. The Rust layer replaces it with SDL events feeding the same ring
buffer and mouse position.

## Hardware registers touched directly

- Audio: `$dff0xx` registers in `gdriver.asm` (`playsample`, `playscore`).
- Interrupts: `AddIntServer` vblank server plus an audio interrupt.
- Copper lists: through the graphics library (`MrgCop`).
