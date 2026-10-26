# Modern Engine Plan

Goal: four games from one project, on two engines.

| Game | Engine | What it is |
|---|---|---|
| original | classic | The game as it is now. Frozen except for bug fixes. |
| improved | classic | Original plus quality of life: free mouse, better key bindings. |
| remastered | modern | New UI, bigger view (zoomed out), most logic in Rust. |
| reimagined | modern | Remastered plus new content: spells, items, tweaks. |

Safety: git tag `original-baseline` marks the classic C game as it was before this work. See what
changed in the C code since then with `git diff original-baseline -- csrc/`.

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
- [ ] Still without scenarios: swan/turtle, key doors. (`combat_graveyard`, `door_cave` added.)
- [x] Record and replay: `FTA_RECORD=file` logs keys and mouse; `FTA_REPLAY=file` repeats them.
      Checked: replay gives an identical state file.
- [x] `regress.sh` golden pixel hashes were stale (they failed on the previous commit too); screens checked by eye, hashes updated. `soak.sh` passes.

### 2. Cut `main()` into functions (still C, no behaviour change)
Input, hero update, actor update, combat, spawning, render, menus. Run tests after each cut.
- [x] `handle_key` (key and menu handling at the top of the loop). State and pixel tests unchanged.
- [x] `update_hero` (timers, dead/fairy, shoot/ride/fight/walk state). Tests unchanged.
Remaining blocks inside the `main()` loop of `csrc/fmain.c` (line numbers as of commit 184cc37,
they move after each cut; the `/* ... */` comments in the loop mark most boundaries):
- [x] Batch 1 (one commit, one test run): `raft_proximity`, `check_bed`, `check_door`, `undo_sprites`,
      and the world update split into `day_night_tick`, `spawn_encounters`, `speak_nearby`,
      `actor_ai`, `survival_tick`. State and pixel tests unchanged.
      Found: `cycle` (frame counter) was never initialised, so its start value was stack garbage and the
      cuts changed it. Now `short cycle = 0` at the top of `main()` (golden state still matches).
- [x] Batch 3: the per-actor motion loop is now `move_actor(i, cycle, &j)`, the whole loop body moved
      as one piece (its `goto`s cross the actor kinds, so it was not split further yet). `j` is passed by
      pointer and `xtest`/`ytest` are static, because a dead actor reads them without setting them.
      State and pixel tests unchanged. Splitting `move_actor` by actor kind can be done in the Rust port.
- [x] Batch 2 (one commit, one test run): `melee_hits`, `move_missiles`, `queue_missiles`, `sort_actors`,
      `repair_map_and_witch`, `draw_sprites`. New scenarios `combat_graveyard` (kills) and `door_cave`
      (outdoor door, cave, dying). Still without scenarios: swan/turtle, key doors.
      `draw_sprites` used to leave `j = anix2`, which the next frame's actor loop reads for dead
      actors, so `main()` sets `j = anix2` after the call.
- Not planned: start-up (before the loop) and shutdown (`quit_all:`).

How to do one cut (the method used for `handle_key` and `update_hero`):
1. Move the block into a `static` function above `main()`. Mark it with a `/* PORT: ... */` comment.
2. Variables that are locals of `main()` or of the loop become parameters or locals of the new
   function. Before making one a plain local, grep for later uses: if anything later reads it
   without assigning it first, it carries a value from this block and must be passed by pointer
   (or made a file-level static) instead.
3. `continue`, `break`, `goto` and `return` that jump out of the block stay in `main()`: cut the
   block just before or after them, or return a flag and let `main()` do the jump.
4. Do not change logic, order or names. Uninitialised locals in the original are initialised to 0
   only if the value is provably never used.
5. Test: `tests/state_regress.sh` and `tests/regress.sh` (both about 5-7 minutes; run them in the
   background and wait for the result). Every scenario must say `ok`. Any `DIFF` means undo the cut.
6. One commit per cut, and tick it off here.

### 3. Gather the game state into structs
Hero, world, actors, inventory, time. Clear split between rules state and drawing state.
Give C a small API: `tick(input)` and `get_state()`.

### 4. Port the rules to Rust (`fta-core`)
Small leaves first: RNG, collision, text, save/load. Then AI, combat, inventory, shops. The old
`main()` loop goes last. Compare against the C using the state tests.

### 4a. `rsrc/`: clean Rust copy of `csrc/` (decided, in progress)
Decision: `csrc/` is never edited again. It stays as the reference. Each C file is ported 1:1 into
`rsrc/` (same behaviour, same function names, bugs included), written to the cleanup rules (short
functions, named constants, small files, tests). `rsrc/` is a module of the main crate
(`#[path = "../rsrc/mod.rs"]` in `src/main.rs`), not a separate crate.
How a file moves: write the Rust with the same `#[unsafe(no_mangle)] extern "C"` names, remove the
file from the list in `build.rs`, check with `cargo test`, `tests/regress.sh` and
`tests/state_regress.sh`. Globals still defined in C are read through `extern` statics; globals
that move must keep their C name. Leaf files first, `fmain.c` / `fmain2.c` last.
- [x] `asm_port.c` -> `rsrc/random.rs` (RNG, checked against values from the C) and
      `rsrc/iff_helpers.rs` (`ReadHeader`, `ReadLength`, `unpack_line`).
- [x] `input.c` -> `rsrc/input.rs` (ring buffer, mouse, menu clicks, compass; pure parts unit tested).
- [x] `iffsubs.c` -> `rsrc/iffsubs.rs` (`unpackbrush` and the shared IFF state; the unused `DEST` check and dead code were dropped).
- [x] `hdrive.c` -> `rsrc/hdrive.rs` (image-file reads plus the original trackdisk path).
- [x] `asm_blit.c`, `asm_text.c`, `asm_map.c`, `asm_stubs.c` -> `rsrc/asm_blit.rs`, `asm_text.rs`,
      `asm_map.rs`, `asm_stubs.rs` (pure parts unit tested: minterms, steering, camera, steps).
- [ ] Then `statedump.c`, `layout.c` (both stay C until the end: layout checks Rust against the C
      compiler's struct sizes, statedump reads game globals), `narr_data.c` (keep as generated data),
      `fmain2.c`, `fmain.c`.
Left off: `fmain2.c` is fully ported and out of the build. `fmain.c` is being ported in groups (see "`fmain.c` port" below).
#### `fmain2.c` split (done)
All functions are ported as groups, each into its own `rsrc/` file. The old C stays in `fmain2.c`
inside `#ifndef RSRC_<GROUP>` (the macros are defined in `build.rs`), so the original is still
readable there and a group can be switched back to C for a bisect.
- [x] `messages.rs`: `print`, `print_cont`, `extract`, `announce_*`, `name`.
- [x] `savegame.rs`: `savegame`, `saveload`, `sl_*`, `locktest`, `waitnewdisk`.
- [x] `objects.rs`: `do_objects`, `set_objects`, `change_object`, `leave_item`.
- [x] `loading.rs`: `cfiles` table, `shape_read`, `read_shapes`, `prep`, `read_score`, the page turn
      (`copypage`, `flipscan`), `skipint`, `into_chip`, `cpytest`, `copy_protect_junk`.
- [x] `actors.rs`: `dohit`, `aftermath`, `proxcheck`, `calc_dist`, `move_figure`, `do_tactic`,
      `rescue`, `eat`, `set_loc`.
- [x] `screen_fx.rs`: compass, `fade_page`, `ppick`, `map_message`, `win_colors`, `day_fade`.
- [x] `witch.rs`: `witch_fx` (only about 40 lines; the first estimate counted the data tables).
- [x] The data tables moved to `tables.rs` (colour tables, object lists and `ob_table`, `witchpoints`,
      bow and loot tables, `minimap`, print queue). `fmain2.c` is no longer compiled; the file stays
      as the reference (with its `RSRC_*` guards, which are off, so it reads as the original).
      `cargo build` is the check that catches a symbol the remaining C still needs (it found `s1`
      and `s2`); `cargo test` alone does not link the C.
#### `fmain.c` port (in progress)
`main()` is already cut into functions (step 2, done). Groups are ported the same way as `fmain2.c`: the C
stays in `fmain.c` inside `#ifndef RSRC_<GROUP>` (macro defined in `build.rs`).
- [x] `regions.rs`: `gen_mini`, `load_all`, `load_new_region` (+ `Need` struct in `types.rs`).
- [x] Small leftovers: `menus.rs` (`set_options`, `gomenu`), `loading.rs` (`read_sample`, `effect`),
      `actors.rs` (`get_turtle`), `savegame.rs` (`mod1save`), `display.rs` (`pagechange`,
      `add_device`, `wrap_device`). `Menu` and `Bro` structs added to `types.rs`.
- [x] Mid-sized functions: `doors.rs` (`doorfind`, `open_list`, `bumped`), `places.rs` (`xfer`,
      `find_place`), `encounters.rs` (`set_encounter`, `checkdead`, `load_carrier`), `revive.rs`
      (`revive`), `mood.rs` (`setmood`), `screen_fx.rs` (`screen_size`).
- [ ] Still to port from the earlier missed list: `propt`, `do_option`, `handle_key`, `update_hero`.
- [ ] `open_all`, `close_all`, `load_actors`, `print_options`.
- [ ] The `main()` pieces: `move_actor`, `draw_sprites`, world update, combat, and so on.
Not covered by the regression scenarios yet: `win_colors`, `rescue`, `witch_fx`, the trackdisk
path in `hdrive`. They were ported line by line; a scenario for each would be worth adding.

Status (summary): ported to `rsrc/` and out of the build: every C file except `fmain.c`,
`layout.c`, `statedump.c` and `narr_data.c` (generated data). All `rsrc/` files have explanatory
comments (file summaries, function docs, game rules); that pass changed no code (checked by comparing
the code with comments stripped against the previous commit).
Exceptions to "csrc is never edited": `#ifndef RSRC_*` guards in `csrc/fmain2.c` (preprocessor only,
off now, so the file reads as the original) and the `main()` cuts planned for `fmain.c`.
After the port (cleanup pass): give the Rust functions readable names with thin wrappers that keep
the C names for any C that is left, then delete the wrappers; replace the `static mut` globals and
raw pointers with proper structs; add test scenarios for `win_colors`, `rescue`, `witch_fx` and the
trackdisk path.
Decision for `fmain.c`: cut `main()` into functions in `csrc/` first (step 2, the method above), then
port the pieces to `rsrc/`. This is the one exception to "csrc is never edited": only step-2 cuts, with
no behaviour change. Do the cuts before starting `fmain.c` in `rsrc/`; `fmain2.c` can be ported earlier.

### Keeping an all-C "original" build possible
Idea, not scheduled: a launcher that offers "Original" (the C game logic) next to the Rust-ported game.
Nothing in `csrc/` is deleted: the ported functions stay in the C files inside `#ifndef RSRC_*`, so
turning the `RSRC_*` defines off compiles the original C again. Rust and C define the same function
names, so both cannot be in one binary; the launcher would start one of two builds (for example a
`cargo` feature that compiles every C file and leaves out the Rust game modules).
Limits: the C still uses the Rust platform layer (SDL window, sound, files), so it is the original
game logic on the new platform, not an Amiga build. The `main()` cuts and `cycle = 0` are the only
edits to the C beyond the guards.
To stop it drifting: when a file is ported, keep its C inside the guard (do not delete it), and once
in a while build with the `RSRC_*` defines off and run the tests on that build.

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
