# The Faery Tale Adventure - Linux Port

A native Linux port of *The Faery Tale Adventure* (MicroIllusions, 1987, Amiga), written in Rust.

The original Amiga source by Talin is at https://github.com/viridia/faery-tale-amiga. Clone it
into `faery-tale-amiga/` in this directory to use as a reference. It is git-ignored and is not part
of this project.

```
git clone https://github.com/viridia/faery-tale-amiga
```

## Status

The game is fully ported to Rust and playable: intro, map, fighting, shops, doors, magic, music,
save and load. No C is compiled any more; `csrc/` is the original C kept as a reference. The code has
been cleaned up (short functions, named constants, unit tests) and the Amiga structs and raw pointers
are gone. What is left is the shared game state (`static mut`), then the optional "modern engine"
steps. The detailed status and plan are in [modern_engine_plan.md](modern_engine_plan.md)
(start at "Where we are"); the history of the port is in [plan.md](plan.md).

## Approach

- `src/game/` is the game, a clean Rust version of the C with the same behaviour (bugs included).
- `src/gfx/` is safe drawing code: owned bitmaps, pens, fonts, and a scene that becomes one frame.
- `src/platform/` is SDL2: window, audio, input, file paths.
- `csrc/` (original C) is not built and not edited. It is the reference for what the game should do.

## Layout

| Path | Purpose |
|------|---------|
| `faery-tale-amiga/` | Original source, reference only (git-ignored) |
| `src/game/` | The game: `actors`, `world`, `ui`, `input`, `render`, `files`, `data`, `startup` |
| `src/gfx/` | Safe bitmap, font and scene drawing |
| `src/platform/` | SDL2 layer: `display`, `audio`, `input`, `paths` |
| `csrc/` | Original C, reference only |
| `docs/` | Notes on the data formats, Amiga calls and assembly routines |
| `tests/` | Headless regression scripts and their golden hashes |
| `tools/` | Helper scripts (`narr2rs.py` regenerates the story text, `package.sh`) |
| `plan.md`, `modern_engine_plan.md` | History and plan |

## Building

Needs Rust (stable) and SDL2 (`libsdl2-dev`). No C compiler is needed.

```
cargo run                                   # runs the game, skipping the intro (Esc quits)
cargo run -- --intro                        # plays the story-book intro first
cargo test                                  # unit tests
cargo run --bin dump_image -- faery-tale-amiga/game/image   # map of the floppy image
```

## Controls

- **Walk:** arrow keys (two together for diagonals), keypad digits, or hold the left mouse button over the compass.
- **Fight:** right mouse button, Ctrl, or keypad 0.
- **Menus:** click them, or use the keys the original game had:
  Items menu: `I` List, `T` Take, `?` Look, `U` Use, `G` Give.
  Talk: `Y` Yell, `S` Say, `A` Ask. Game: Space Pause, `M` Music, `F` Sound, `Q` Quit, `L` Load.
  Buy: `O R 8 C W B E`. Save menu: `V` Save, `X` Exit. Magic: F1-F7. Use: `1`-`7`, `K` keys.
- **Gamepad:** left stick or D-pad walks, A fights, X Take, Y List, B Look, LB Use, RB Give, Start Pause.
- **Fullscreen:** F11 or Alt+Enter. The window can also be resized.
- **Remap keys:** copy `controls.cfg.example` to `controls.cfg` and edit it (`W = Up`).
- **Esc** closes the window.

## Testing

All headless (no window opens, no sound):
- `tests/regress.sh` replays scripted sessions (walking, fighting, menus, save/load) and compares screen hashes with `tests/golden.txt`. `--update` accepts new output.
- `tests/state_regress.sh` replays scenarios and compares a dump of the game state with `tests/state_golden.txt`.
- `tests/soak.sh [runs] [ticks]` feeds random keys and clicks to an AddressSanitizer build (written when
  C was part of the build; not re-checked since).

Set `SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy` to run the game itself headless, `FTA_FAST=1` to skip the 50 Hz pacing.

## Packaging

`tools/package.sh` builds a release binary and packs it (no game data) into `dist/`.

## Saving

Saves go in `saves/` (git-ignored) as `A.faery` ... `H.faery`. Use Game, Quit, Save.

## Legal

The original source is released by its author under an MIT-style license, but the copyright status
of the game and its data files is unclear. Do not distribute game data from this repo.
