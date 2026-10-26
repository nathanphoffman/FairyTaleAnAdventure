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
- [ ] Still without scenarios: swan/turtle, key doors. (`combat_graveyard` and `door_cave` were added later.)
- [x] Record and replay: `FTA_RECORD=file` logs keys and mouse; `FTA_REPLAY=file` repeats them.
      Checked: replay gives an identical state file.
- [x] `regress.sh` golden pixel hashes were stale (they failed on the previous commit too); screens checked by eye, hashes updated. `soak.sh` passes.

### 2. Cut `main()` into functions (still C, no behaviour change)  (done)
`main()` was 1,400 lines. It is now cut into functions in `csrc/fmain.c`, all tests unchanged
(commits `5153af6`, `f1a9bd7`, `e264a72`, plus the earlier `handle_key` and `update_hero`):
- [x] Input and hero: `handle_key`, `update_hero`.
- [x] Before the actor loop: `raft_proximity`. The actor loop itself: `move_actor(i, cycle, &j)`
      (the whole body moved as one piece; its `goto`s cross the actor kinds, so it is not split further).
- [x] After it: `check_bed`, `check_door`, `undo_sprites`.
- [x] World update (once per tick): `day_night_tick`, `spawn_encounters`, `speak_nearby`, `actor_ai`,
      `survival_tick`.
- [x] Combat and drawing: `melee_hits`, `move_missiles`, `queue_missiles`, `sort_actors`,
      `repair_map_and_witch`, `draw_sprites`.
- Start-up (before the loop) and shutdown (`quit_all:`) stay in `main()`.
Things the cuts turned up (all handled, keep them in mind for the Rust versions):
- `cycle` (frame counter) was never initialised, so it started as stack garbage; now `short cycle = 0`.
- A dead actor reads `j`, `xtest` and `ytest` in the actor loop without setting them, so they carry
  over from the previous actor. `move_actor` takes `j` by pointer and keeps `xtest`/`ytest` static;
  `main()` sets `j = anix2` after `draw_sprites` because the drawing loop used to leave it there.
- New scenarios were added: `combat_graveyard`, `door_cave`.

How to do one cut (kept for reference; the method that worked):
1. Move the block into a `static` function above `main()`, marked `/* PORT: ... */`.
2. Locals of `main()` become parameters or locals. If anything later reads a variable without
   assigning it first, it carries a value from this block: pass it by pointer or make it static.
3. `continue`, `break`, `goto`, `return` that jump out of the block stay in `main()`.
4. Do not change logic, order or names.
5. Run both test scripts. Several cuts may share one test run. (Commits are made by the user, see the handoff.)
   If a state test goes `DIFF` and the run is not deterministic (two runs of the same binary differ),
   the cause is an uninitialised variable. Bisect by applying the cuts one at a time.

### 3. Gather the game state into structs
Hero, world, actors, inventory, time. Clear split between rules state and drawing state.
Give C a small API: `tick(input)` and `get_state()`.

### 4. Port the rules to Rust (`fta-core`)
Small leaves first: RNG, collision, text, save/load. Then AI, combat, inventory, shops. The old
`main()` loop goes last. Compare against the C using the state tests.

### 4a. `rsrc/`: clean Rust copy of `csrc/` (decided, in progress)
Decision: `csrc/` is the reference and is not edited, except `#ifndef RSRC_*` guards (preprocessor
only) and the step-2 `main()` cuts. Each C file is ported 1:1 into `rsrc/` (same behaviour, same
function names, bugs included), written to the cleanup rules (short functions, named constants,
small files, tests). `rsrc/` is a module of the main crate (`#[path = "../rsrc/mod.rs"]` in
`src/main.rs`), not a separate crate.

**Status.** Every C file is ported except `fmain.c` (about 60% of it is done), and `layout.c`,
`statedump.c` and `narr_data.c`, which stay C on purpose (`layout.c` checks the Rust struct sizes
against the C compiler, `statedump.c` reads game globals, `narr_data.c` is generated text).
Done files: `random`, `iff_helpers`, `input`, `iffsubs`, `hdrive`, `asm_blit`, `asm_text`,
`asm_map`, `asm_stubs`, `messages`, `savegame`, `objects`, `loading`, `actors`, `screen_fx`,
`witch`, `tables`, `types`, and from `fmain.c`: `regions`, `menus`, `display`, `doors`, `places`,
`encounters`, `revive`, `mood`, `menus` (with `propt`), `options`, `items_menu`, `magic`, `keys`. All have explanatory comments.

#### HANDOFF: how to port the next group (read this first)
**Method (what every group so far followed).**
1. Pick a group of related functions in `csrc/fmain.c`. Read the C carefully, including the globals it
   touches. Write the Rust in `rsrc/<name>.rs` (add `pub mod` to `rsrc/mod.rs` with a one-line comment),
   with `#[unsafe(no_mangle)] pub extern "C" fn <same C name>`. Doc comments say what the rules are.
2. In `csrc/fmain.c` wrap the old C in `#ifndef RSRC_<GROUP> /* PORT: ported to rsrc/<name>.rs */ ...
   #endif /* RSRC_<GROUP> */` and add `.define("RSRC_<GROUP>", None)` in `build.rs`. The C stays as the
   reference. If other C still uses a variable that was defined inside the guard (like `bumped`), add
   `#else extern ...;` inside the guard. If the function was `static` in C and `main()` calls it, add a
   non-static prototype in the `#else` branch.
3. `cargo build` (this is the real check: `cargo test` does not link the C; a missing guard shows up as
   "duplicate symbol", a missing function as an undefined symbol). Then `cargo test`, then
   `tests/regress.sh` and `tests/state_regress.sh`. Run them in the background, about 10 minutes together.
   All 10 pixel tests and all 25 state scenarios must say `ok`; any `DIFF` means fix or undo.
4. Update this plan. **Do not commit: the user makes the commits.** Leave the changes in the working
   tree, say what changed, and suggest a one-line commit message (one commit per group is the idea).

**Rules of thumb that avoided bugs.**
- C `long` is 64 bit, `LONG` is `i32`, `short` is `i16`, `char` is signed (`i8`). `USHORT` arithmetic
  wraps at 16 bits: use `wrapping_add/sub`. Watch int promotion: `x - 16` on a `u16` can go negative in C.
- C has file-level globals `LONG i; SHORT j,k,n;`. Functions that use `i`/`j` without declaring them write
  these globals. In Rust import them with `#[link_name = "i"] static mut c_i: i32;` (a plain `i` would
  clash with local variables). Only mirror the write if something could read it.
- Globals defined in C are read in Rust through `unsafe extern "C" { static ...; }`. Several `rsrc` files
  declare the same C global; that is fine. Keep the C name for any global that moves. Use `#[link_name]`
  when the C name collides with a Rust local (for example `kind`).
- Struct mirrors go in `rsrc/types.rs` with a size check in its test (`Need`, `Menu`, `Bro` were added).
- Uninitialised C variables: treat as 0 only if the value is never really used; otherwise note it in a
  comment (see step 2 for the ones already found).
- A state test that differs between two runs of the same binary means an uninitialised variable.

**Tests and scenarios.** `tests/state_scenarios.txt` has the scenarios (name | tick | keys | clicks |
`tp:x:y` / `kill:luck` commands); a new scenario's golden hash is appended to `tests/state_golden.txt`
after confirming two runs give the same state. `tests/regress.sh` compares screen hashes. Not covered
yet: swan/turtle, key doors, `win_colors`, `rescue`, `witch_fx`, the trackdisk path in `hdrive`.

**Playing it.** `cargo run` (skips the intro; `-- --intro` plays it). Needs the original game data in
`faery-tale-amiga/game/` (git-ignored). See README.

#### Roadmap: what is left in `csrc/fmain.c`
About 2,300 lines of functions plus the data tables at the top. Six groups, in this order.
Sizes are the C line counts.
- [x] **A. Menus and keys (~600).** Done: `propt` in `menus.rs`; `do_option` in `options.rs`
      (talk, buy, game, use, keys, save, give) with the Items menu in `items_menu.rs` and the Magic
      menu in `magic.rs`; `handle_key` and `update_hero` in `keys.rs` (guards `RSRC_MENUS`,
      `RSRC_OPTIONS`, `RSRC_KEYS`). The two `static` functions have non-static prototypes in the C
      `#else` branch. `secx`/`secy` (always 0) stay defined in C.
- [ ] **B. World update (~460).** The small pieces cut from `main()`: `raft_proximity` (12),
      `check_bed` (22), `check_door` (68), `undo_sprites` (17), `day_night_tick` (36),
      `spawn_encounters` (47), `speak_nearby` (13), `actor_ai` (89), `survival_tick` (34),
      `melee_hits` (31), `move_missiles` (38), `queue_missiles` (21), `sort_actors` (32). Suggested
      files: `world.rs` (time, spawning, hunger), `combat.rs` (melee, missiles), `ai.rs` (`actor_ai`).
      All are `static` in C, called only from `main()`.
- [ ] **C. `move_actor` (~400).** The riskiest: moves and animates every actor (hero, raft, carriers,
      people, enemies), with many `goto` labels (`statc`, `raise`, `cpx`, `newloc`, `blocked`,
      `checkdev1/2`). Port it as one function first, using a loop plus `match` for the jumps, then
      split by actor kind only after it passes. Remember the stale-`j`/`xtest`/`ytest` rule (step 2).
      Add scenarios for swan/turtle and key doors before this one, since it covers those paths.
- [ ] **D. Drawing (~260).** `draw_sprites` (~230: clipping, weapon pass, masking against scenery,
      background save) and `repair_map_and_witch` (~20). Uses the blitter already in `asm_blit.rs`.
      `main()` sets `j = anix2` after the call (see step 2); keep that.
- [ ] **E. Start-up and shutdown (~600).** `open_all` (~220: opens libraries, allocates memory, loads
      fonts and pictures), `close_all` (~55), `load_actors` (~10), `print_options` (~20), and last
      `main()` itself (~290: intro, start-up, the main loop that calls everything above, quit). When
      `main()` is Rust, remove `-Dmain=fta_main` from `build.rs` and call it from `src/main.rs`.
- [ ] **F. Data tables, then drop `fmain.c` from the build.** Move the tables at the top of `fmain.c`
      to `rsrc/tables.rs` (or a new file): `setfig_table`, `doorlist`, `extent_list`, `inv_list`, the
      `menus` array and its labels, `blist`, `file_index`, `open_list`-style tables, `introcolors`,
      `new_wave`, `encounter_chart`, and so on. Keep each variable's C name and layout. Then remove
      `fmain.c` from the file list in `build.rs`. The file stays in the repo as the reference.
      `cargo build` will report any symbol the remaining C (`layout.c`, `statedump.c`) still needs.

When F is done the port is finished. What remains afterwards (optional, no firm end) is the cleanup pass:
- Give the Rust functions readable names with thin wrappers that keep the C names for any C still
  left (`layout.c`, `statedump.c`), then delete the wrappers.
- Replace the `static mut` globals and raw pointers with proper structs (this is step 3).
- Add scenarios for `win_colors`, `rescue`, `witch_fx`, the trackdisk path, swan/turtle, key doors.
- See the note below on keeping an all-C "original" build possible while doing this.

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
