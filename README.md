# The Faery Tale Adventure - Linux Port

A native Linux port of *The Faery Tale Adventure* (MicroIllusions, 1987, Amiga), written in Rust
with the original game logic kept in C at first and moved to Rust over time.

The original Amiga source by Talin is at https://github.com/viridia/faery-tale-amiga. Clone it
into `faery-tale-amiga/` in this directory to use as a reference. It is git-ignored and is not part
of this project.

```
git clone https://github.com/viridia/faery-tale-amiga
```

## Status

Phases 0-3 done: the original C game runs inside a Rust program on SDL2. The intro, menus and text panel work, with keyboard and mouse. The map is not drawn yet (Phase 4), so it is not playable. See [plan.md](plan.md).

## Approach

- Rust provides the new platform layer: SDL2 window, graphics, input, audio, timing, file loading.
- The original C game logic is compiled into the Rust build (via the `cc` crate) and calls the
  platform layer through `extern "C"` functions.
- The 68000 assembly is rewritten by hand.
- Game modules are ported from C to Rust one at a time, with the game runnable after each step.

## Layout

| Path | Purpose |
|------|---------|
| `faery-tale-amiga/` | Original source, reference only (git-ignored) |
| `plan.md` | Porting plan and progress |
| `src/` | Rust code (to be created) |
| `csrc/` | Original C game code with minimal changes, plus Amiga stand-in headers and stubs |
| `tools/` | Helper scripts (`genprotos.sh` regenerates `csrc/protos.h`) |

## Building

Needs Rust (stable), a C compiler (gcc) and SDL2 (`libsdl2-dev`).

```
cd csrc && make                             # builds ../build/fta_c (C part only, stubbed)
cargo run                                   # runs the game, skipping the intro (Esc quits)
cargo run -- --intro                        # plays the story-book intro first
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
- `tests/soak.sh [runs] [ticks]` feeds random keys and clicks to an AddressSanitizer build.

Set `SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy` to run the game itself headless, `FTA_FAST=1` to skip the 50 Hz pacing.

## Packaging

`tools/package.sh` builds a release binary and packs it (no game data) into `dist/`.

## Saving

Saves go in `saves/` (git-ignored) as `A.faery` ... `H.faery`. Use Game, Quit, Save.

## Legal

The original source is released by its author under an MIT-style license, but the copyright status
of the game and its data files is unclear. Do not distribute game data from this repo.
