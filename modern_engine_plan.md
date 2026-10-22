# Modern Engine Plan

Goal: four games from one project, on two engines.

| Game | Engine | What it is |
|---|---|---|
| original | classic | The game as it is now. Frozen except for bug fixes. |
| improved | classic | Original plus quality of life: free mouse, better key bindings. |
| remastered | modern | New UI, bigger view (zoomed out), most logic in Rust. |
| reimagined | modern | Remastered plus new content: spells, items, tweaks. |

Core idea: split the game into **rules** (what happens) and **presentation** (drawing and input).
Both engines share the rules. They differ only in how they draw and read input.

## Layout (target)

```
crates/
  fta-data/             load original files (image, songs, fonts, IFF) into plain types
  fta-core/             game rules and state, no drawing, no SDL
  fta-engine-classic/   320x200 planar renderer (original, improved)
  fta-engine-modern/    RGBA renderer, big viewport, new UI (remastered, reimagined)
  fta-c/                leftover C, built once and shared
games/  original  improved  remastered  reimagined
```

## Rules for every step
1. One piece at a time. Never port two things in one step.
2. The game works after every step. `tests/regress.sh` and `tests/soak.sh` pass before each commit.
3. Port behaviour exactly, bugs included. Clean up later, with tests around it.
4. Keep the C to Rust seam thin (`#[unsafe(no_mangle)] extern "C"`, same name as the C function).
5. Use `i16`/`i32` with `wrapping_*` to match C integer behaviour. Check every signed/unsigned
   comparison (the ones silenced by `-Wno-sign-compare`).

## Steps

### 1. Safety net  (done)
- [x] State dump: `FTA_STATE=file FTA_STATE_AFTER=tick` writes hero, stats, all 20 actors, inventory
      and RNG seed as text (`csrc/statedump.c`). Same inputs give identical output.
- [x] `tests/state_regress.sh` + `tests/state_golden.txt`: compares state instead of pixels
      (takes about 7 minutes; run `--update` to accept new output).
- [x] Extra scenarios in `tests/state_scenarios.txt`: 8 seeded random sessions (`fuzz_N`). One
      reaches region 8 (indoors). They stand in for shops, talking, doors, caves and dying.
- [x] Targeted scenarios (`FTA_CMD="tick:tp:x:y,tick:kill:luck"` teleports like a door or kills the
      hero; applied by `fta_test_hook()` at the top of the main loop): `talk_beggar`, `shop_buy`
      (inn, bartender), `inn_inside` (region 8), `die_fairy` (revived), `die_brother` (brother 2).
- [ ] Still without scenarios: caves/underground, swan/turtle, key doors, combat kills.
- [x] Record and replay: `FTA_RECORD=file` logs keys and mouse; `FTA_REPLAY=file` repeats them.
      Checked: replay gives an identical state file.
- [x] `regress.sh` golden pixel hashes were stale (they failed on the previous commit too); screens checked by eye, hashes updated. `soak.sh` passes.

### 2. Cut `main()` into functions (still C, no behaviour change)
Input, hero update, actor update, combat, spawning, render, menus. Run tests after each cut.
- [x] `handle_key` (key and menu handling at the top of the loop). State and pixel tests unchanged.
- [ ] controls, monster and hero motion, bed/doors/map, drawing, missiles and scroll.

### 3. Gather the game state into structs
Hero, world, actors, inventory, time. Clear split between rules state and drawing state.
Give C a small API: `tick(input)` and `get_state()`.

### 4. Port the rules to Rust (`fta-core`)
Small leaves first: RNG, collision, text, save/load. Then AI, combat, inventory, shops. The old
`main()` loop goes last. Compare against the C using the state tests.

### 5. Classic engine: original and improved
Move the planar drawing code (`asm_blit.c`, `asm_map.c`) into `fta-engine-classic`. Build
`games/original`. Add `games/improved` options (free mouse, rebinds) on top.
Cheap, and only touches the input layer, so can happen any time after step 1.

### 6. Modern engine: remastered
Convert tiles and sprites from planar to RGBA atlases. Draw from the state, not from C bitmaps.
Camera with zoom, larger visible area, new UI layer. Check that actors spawn and act across the
larger visible area, since the original only simulates around the 320x200 view.

### 7. Reimagined
Move tables (items, enemies, spells, text) from code into data files. Add new content on top
of the remastered engine.

## Risks
- Bigger view means the map and actor update code must cover more area.
- Shared C between engines means no engine-specific edits in it, so port the rules to Rust early.
- Screenshots only test the classic engine; the modern one will look different on purpose, so
  state tests matter more.
