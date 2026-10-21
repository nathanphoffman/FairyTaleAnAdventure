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

Planning. See [plan.md](plan.md).

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
| `csrc/` | Cleaned-up C code compiled into the build (to be created) |

## Building

Not buildable yet. Will need Rust (stable), a C compiler (gcc) and SDL2 (`libsdl2-dev`).

## Legal

The original source is released by its author under an MIT-style license, but the copyright status
of the game and its data files is unclear. Do not distribute game data from this repo.
