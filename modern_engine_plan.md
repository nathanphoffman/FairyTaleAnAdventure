# Modern Engine Plan

Goal: four games from one project, on two engines.

| Game | Engine | What it is |
|---|---|---|
| original | classic | The game as it is now. Frozen except for bug fixes. |
| improved | classic | Original plus quality of life: free mouse, better key bindings. |
| remastered | modern | New UI, bigger view (zoomed out), most logic in Rust. |
| reimagined | modern | Remastered plus new content: spells, items, tweaks. |

Safety: git tag `original-baseline` marks the classic C game as it was before this work. The old
history of the port (C to Rust, test scenarios, file renames) is in `plan.md` and in git.

Core idea: split the game into **rules** (what happens) and **presentation** (drawing and input).
Both engines share the rules. They differ only in how they draw and read input.

## Where we are

The whole game is ported to Rust and plays like the original. No C is compiled; `csrc/` is the
original, kept as a reference. Checked by 229 unit tests, 13 pixel scenarios (`tests/regress.sh`), 21
state scenarios (`tests/state_regress.sh`) and a save/load round trip.

**How the code is organised**
- One `Game` value (`data/state.rs`) holds the rules' state: hero, world, actors, ui, encounter,
  witch, rng, menus, extents, object lists, the map data and the page colours. There is no
  `static mut` and no `unsafe` in the code.
- `Video` holds the drawing state (`screen`, `graphics`: tile images, shadow masks and sprite sheets,
  `blitter`). `run_game` creates both and passes `&mut Game` and `&mut Video` down.
- The rules talk to the screen only through `Output` (`src/game/output.rs`); `Video` implements it
  (`src/game/render/output.rs`). It covers printing into the text panel, the text position and font,
  story pages (`clear_placard`, `draw_placard_border`), menu entries, page and panel palettes,
  swapping and showing pages, waiting a frame, loading sprite sheets and image banks, and a few whole
  screens (compass, item list, overview map, message page, intro zoom). The rules do not see ports,
  pens or rectangles. `actors`, `world`, `ui`, `input` and `files` never name `Video`; they take
  `out: &mut dyn Output`. Drawing code lives in `render/` and `startup/`.
- `game/recorder.rs` has a second `Output`, `Recorder`, which draws nothing and records every call,
  so rules can be tested without a screen.
- The platform does not depend on the game: `platform/host.rs` is the interface (`Host`: key and
  mouse input, joystick fire, test commands), `game/host.rs` implements it and `main` installs it.
- Functions take only what they use where that was possible (`&Hero`, `&mut Rng`, ...). About 260
  orchestrating functions still take the whole `&mut Game` because they pass it on; narrowing them
  further would make the signatures worse. No function has more than seven parameters.
- Actors and missiles are `ActorRef` / `MissileRef` (slot numbers); the big memory areas are owned
  buffers (`image`, `sector`, `terrain`, `shadow`, sprite sheets); the sprite blitter works on
  `Loc { area, pos }` places in them. Drawing uses the safe `gfx` module (owned bitmaps, pens,
  fonts, a `Scene` per frame).

**Not done**
- `Output` still passes 12 bit palettes (`load_page_colors`), menu box colours and `FontKind`;
  a renderer with its own look would replace those.
- Steps 5 to 7 below (classic/modern engines, remastered, reimagined) have not been started.

**Next:** write the modern renderer behind `Output` (it can read `Game` directly), then steps 5 to 7.

Each step keeps the rule: tests pass before each commit, behaviour is unchanged unless the step says
so. Checks: `cargo build`, `cargo clippy`, `cargo test`, `tests/regress.sh`,
`tests/state_regress.sh` (every scenario must say `ok`).

## Layout (target)

```
crates/
  fta-data/             load original files (image, songs, fonts, IFF) into plain types
  fta-core/             game rules and state, no drawing, no SDL
  fta-engine-classic/   320x200 planar renderer (original, improved)
  fta-engine-modern/    RGBA renderer, big viewport, new UI (remastered, reimagined)
games/  original  improved  remastered  reimagined
```

## Steps

### 5. Classic engine: original and improved
Move the planar drawing code (`render/`, `gfx/`) into `fta-engine-classic`. Build `games/original`.
Add `games/improved` options (free mouse, rebinds) on top. Cheap, and only touches the input layer.

### 6. Modern engine: remastered
Convert tiles and sprites from planar to RGBA atlases. Draw from the state, not from the planar
bitmaps. Camera with zoom, larger visible area, new UI layer. Check that actors spawn and act across
the larger visible area, since the original only simulates around the 320x200 view.

### 7. Reimagined
Move tables (items, enemies, spells, text) from code into data files. Add new content on top of the
remastered engine.

## Risks
- Bigger view means the map and actor update code must cover more area.
- Screenshots only test the classic engine; the modern one will look different on purpose, so state
  tests matter more.
