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

## Where we are

**Done (all verified: 224 unit tests, the state and pixel playtests (21 and 13 scenarios ok), save/load round trip).**
1. Safety net, `main()` cut into functions, game state gathered into structs (steps 1-3 below).
2. The whole game ported from C to Rust in `src/game/` (step 4a). `fmain.c` and the rest of `csrc/`
   are no longer compiled.
3. Cleanup: magic numbers named, big files split, long functions split, comments, unit tests instead
   of most playtests. Last codequality score 93.6 / 100 (A).
4. No C in the build (story text is generated Rust data); platform calls are plain Rust.
5. Amiga structs gone: safe `gfx` module, a `Screen` that owns bitmaps and ports, owned audio data,
   save file written field by field, input handler is a normal Rust type.
6. Memory model: owned buffers for map, sectors, terrain, tile images, shadow masks and sprite
   sheets; the sprite blitter works on places in those buffers instead of raw pointers.
7. Text is `&str`; actors and missiles are `ActorRef` / `MissileRef`; item counts, extents and object
   lists are plain data. No `*mut` / `*const` left in the game code (details in "Owned memory instead
   of Amiga structs and raw pointers" at the end).

8. One `Game` value (`data/state.rs`) holds all the game state: `hero`, `world`, `actors`, `ui`,
   `encounter`, `witch`, `memory`, `screen`, `rng`, the menus, extents, object lists, page colours and
   the small work variables. `run_game` creates it and passes `&mut Game` down. There is no
   `static mut` left and no `unsafe` in `src/game/` (one `unsafe` remains in `platform/`). The
   constant tables (`tables.rs`, `object_tables.rs`) are plain `static` data; the ones that change
   (menus, extents, object lists, page colours) are copied into `Game` at the start from
   `initial_*`. `ActorRef` and `MissileRef` are plain slot numbers: read an actor with
   `game.actors.list[r.slot()]`.

9. Cleanup pass after the state move: the loose fields are grouped (`Blitter`, `PrintQueue`, `Witch`,
   `Ui`, `Actors`, ...), actors are read with `game.actor(r)` / `game.actor_mut(r)`, the boilerplate
   bit-mask comments are gone, table lookups that could be out of range use `tables::entry`, and 61
   functions take only the parts of `Game` they use (`&Hero`, `&mut Rng`, `&mut Screen`, ...).
10. Rules and drawing state are two values: `Game` (hero, world, actors, ui, encounter, witch, rng,
   extents, menus, objects, `map`: sector and terrain data) and `Video` (`screen`, `graphics`: tile
   images, shadow masks and sprite sheets, `blitter`, `page_colors`). `run_game` creates both and
   passes `&mut Game` and `&mut Video` down. `platform/display` only takes a `Scene`; the state-dump
   test hook lives in `game/render/display.rs`.

11. The rules talk to the screen through `Output` (`src/game/output.rs`), and `Video` implements it
   (`src/game/render/output.rs`). It covers printing into the text panel, drawing text and lines on
   the three ports (play field, message area, menu area), loading the page and panel colours,
   swapping and showing pages, waiting a frame, loading sprite sheets and image banks, and a few
   whole screens (compass, item list, overview map, message page, intro zoom). The folders `actors`,
   `world`, `ui`, `input` and `files` no longer name `Video` at all; they take `out: &mut dyn Output`.
   The drawing code that used to sit in those folders moved to `render/` (`map_tiles.rs`,
   `overview_map.rs`, `sprite_loading.rs`, `screen_fx/item_screen.rs`) and `startup/story_pages.rs`.
   `view_status` (which screen is showing) moved from `Screen` to `Game.ui`, and the page colours
   from `Video` to `Game`.

**Not done.**
- `Output` is still a thin layer over the Amiga drawing: it takes `PortId` and `FontKind` and has pen
  and cursor calls, so a different renderer would have to imitate those or the story and menu code
  would need higher-level calls (show this placard, draw this menu).
- Unused `out` parameters are left in a few functions; they can be removed.
- `Game` can now be built and driven with any `Output`, but there is no second one yet (for example
  a recording one for tests).
- About 270 functions still take the whole `&mut Game`; 10 have more than six parameters (marked
  `allow(clippy::too_many_arguments)`).
- `platform/display/scripting.rs` still calls `game::statedump::queue_command` for the test commands,
  and `platform/input` feeds keys to `game::input::handler`.
- Steps 5 to 7 below (classic/modern engines, remastered, reimagined) have not been started.

**Plan for what comes next, in order.**
1. *Use the output interface*: add a second `Output` that records what the rules ask for (so rules
   can be tested without a screen), replace the pen and cursor calls with higher-level ones, and
   drop the unused `out` parameters. Then the modern renderer can read `Game` directly.
2. Step 5 (classic engine, improved options), step 6 (modern renderer: RGBA tiles, zoom, bigger view),
   step 7 (data files for items, enemies, spells and text).

Each step keeps the rule: tests pass before each commit, behaviour is unchanged unless the step says
so. Checks: `cargo build`, `cargo test`, `tests/regress.sh`, `tests/state_regress.sh` (every scenario
must say `ok`).

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
- [x] Later additions: `combat_graveyard`, `door_cave`, and 11 scenarios added with ports A and B
      (`sleep_bed`, `take_chest`, `take_sacks`, `loot_graveyard`, `stone_jump`, `spells`, `spells_2`,
      `desert_no_statues`, `desert_statues`, `hunger_events`, `hunger_fruit`). More test commands:
      `give:slot:count`, `stat:0|1|2:value` (hunger, fatigue, wealth), `slay` (kill all enemies out).
      Total: 36 state scenarios plus the 10 pixel tests.
- [ ] Still without scenarios: swan/turtle, key doors. `loot_graveyard` never reached a body close
      enough to search, and in `spells` the freeze and fear spells did not fire (`spells_2` was added
      for them but is not checked either), so body search, freeze and fear are weakly covered.
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
      `repair_edges`, `draw_sprites`.
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

### 4a. `src/game/`: clean Rust copy of `csrc/` (done; the notes below are the history, and they mention the C build, `build.rs` and `RSRC_*` guards that no longer exist)
Decision: `csrc/` is the reference and is not edited, except `#ifndef RSRC_*` guards (preprocessor
only) and the step-2 `main()` cuts. Each C file is ported 1:1 into `src/game/` (same behaviour, same
function names, bugs included), written to the cleanup rules (short functions, named constants,
small files, tests). `src/game/` is a module of the main crate (`#[path = "../src/game/mod.rs"]` in
`src/main.rs`), not a separate crate.

**Status.** Every C file is ported except `fmain.c` (about 85% of its functions are done; groups C to F
below are left), and `layout.c`,
`statedump.c` and `narr_data.c`, which stay C on purpose (`layout.c` checks the Rust struct sizes
against the C compiler, `statedump.c` reads game globals, `narr_data.c` is generated text).
Done files: `random`, `iff_helpers`, `input`, `iffsubs`, `hdrive`, `asm_blit`, `asm_text`,
`asm_map`, `asm_stubs`, `messages`, `savegame`, `objects`, `loading`, `actors`, `screen_fx`,
`witch`, `tables`, `types`, and from `fmain.c`: `regions`, `menus`, `display`, `doors`, `places`,
`encounters`, `revive`, `mood`, and (groups A and B) `options`, `items_menu`, `magic`, `keys`, `world`,
`combat`, `ai`, with `propt` added to `menus`. All have explanatory comments.

#### HANDOFF: how to port the next group (read this first)
**Method (what every group so far followed).**
1. Pick a group of related functions in `csrc/fmain.c`. Read the C carefully, including the globals it
   touches. Write the Rust in `src/game/<name>.rs` (add `pub mod` to `src/game/mod.rs` with a one-line comment),
   with `#[unsafe(no_mangle)] pub extern "C" fn <same C name>`. Doc comments say what the rules are.
2. In `csrc/fmain.c` wrap the old C in `#ifndef RSRC_<GROUP> /* PORT: ported to src/game/<name>.rs */ ...
   #endif /* RSRC_<GROUP> */` and add `.define("RSRC_<GROUP>", None)` in `build.rs`. The C stays as the
   reference. If other C still uses a variable that was defined inside the guard (like `bumped`), add
   `#else extern ...;` inside the guard. If the function was `static` in C and `main()` calls it, add a
   non-static prototype in the `#else` branch.
3. `cargo build` (this is the real check: `cargo test` does not link the C; a missing guard shows up as
   "duplicate symbol", a missing function as an undefined symbol). Then `cargo test`, then
   `tests/regress.sh` and `tests/state_regress.sh`. Run them in the background, about 10 minutes together.
   All 10 pixel tests and all 36 state scenarios must say `ok`; any `DIFF` means fix or undo.
   Extra check that worked well for A and B: for new scenarios, build the commit before the port in a
   `git worktree` (symlink `faery-tale-amiga/` into it, copy over the test-only files), run the same
   scenarios there, and compare the state hashes with the Rust build. Identical hashes mean the port
   matches the original C on those paths, not just that it matches itself.
4. Update this plan. **Do not commit: the user makes the commits.** Leave the changes in the working
   tree, say what changed, and suggest a one-line commit message (one commit per group is the idea).

**Rules of thumb that avoided bugs.**
- C `long` is 64 bit, `LONG` is `i32`, `short` is `i16`, `char` is signed (`i8`). `USHORT` arithmetic
  wraps at 16 bits: use `wrapping_add/sub`. Watch int promotion: `x - 16` on a `u16` can go negative in C.
- C has file-level globals `LONG i; SHORT j,k,n;`. Functions that use `i`/`j` without declaring them write
  these globals. In Rust import them with `#[link_name = "i"] static mut c_i: i32;` (a plain `i` would
  clash with local variables). Only mirror the write if something could read it.
- Globals defined in C are read in Rust through `unsafe extern "C" { static ...; }`. Several `src/game` files
  declare the same C global; that is fine. Keep the C name for any global that moves. Use `#[link_name]`
  when the C name collides with a Rust local (for example `kind`).
- Struct mirrors go in `src/game/data/types.rs` with a size check in its test (`Need`, `Menu`, `Bro` were added).
- Uninitialised C variables: treat as 0 only if the value is never really used; otherwise note it in a
  comment (see step 2 for the ones already found).
- A state test that differs between two runs of the same binary means an uninitialised variable.

**Tests and scenarios.** `tests/state_scenarios.txt` has the scenarios (name | tick | keys | clicks |
test commands: `tp:x:y`, `kill:luck`, `give:slot:count`, `stat:0|1|2:value`, `slay`; only one command
can run per tick); a new scenario's golden hash is appended to `tests/state_golden.txt`
after confirming two runs give the same state. `tests/regress.sh` compares screen hashes. Rows in
`scenarios.txt` may have a fifth column of test commands; `regress.sh` (pixels) and `state_regress.sh`
(state) both run them. Not covered yet: the freeze and fear spells, and the trackdisk path in `hdrive`
(only used when the `image` file is missing; the platform layer has no trackdisk device, so it cannot
run). Covered since the port: swan/turtle, key doors, `win_ending`, `rescue_scene`, `witch_beam`
(the last three were checked against a build of the C versions, identical).

**Playing it.** `cargo run` (skips the intro; `-- --intro` plays it). Needs the original game data in
`faery-tale-amiga/game/` (git-ignored). See README.

#### Roadmap: what is left in `csrc/fmain.c`
About 1,300 lines of functions (A and B are done) plus the data tables at the top. Six groups, in this
order; all six are done. Sizes are the C line counts.
- [x] **A. Menus and keys (~600).** Done: `propt` in `menus.rs`; `do_option` in `options.rs`
      (talk, buy, game, use, keys, save, give) with the Items menu in `items_menu.rs` and the Magic
      menu in `magic.rs`; `handle_key` and `update_hero` in `keys.rs` (guards `RSRC_MENUS`,
      `RSRC_OPTIONS`, `RSRC_KEYS`). The two `static` functions have non-static prototypes in the C
      `#else` branch. `secx`/`secy` (always 0) stay defined in C. Struct mirrors `InvItem`, `Letter`
      added to `types.rs`.
- [x] **B. World update (~460).** Done: `world.rs` (`raft_proximity`, `check_bed`, `check_door`,
      `undo_sprites`, `day_night_tick`, `spawn_encounters`, `speak_nearby`, `survival_tick`,
      `sort_actors`), `combat.rs` (`melee_hits`, `move_missiles`, `queue_missiles`) and `ai.rs`
      (`actor_ai`). One C guard, `RSRC_WORLD`, with prototypes in the `#else`. `doorlist` stays in C
      until group F (`Door` mirror added to `types.rs`). Two original quirks are kept on purpose: the
      missile dodge test uses the missile's own index, and the melee carrier test looks at the attacker.
- [x] **C. `move_actor` (~400).** Done: `move_actor.rs` (guard `RSRC_MOVE_ACTOR`, prototype in the C
      `#else`). Each `goto` label of the C is a `Pc` value and each stretch between labels a method
      (`dragon`, `carrier`, `setfig`, `raft`, `walking`, `new_loc`, `shooting`, `tail`, `raise`, ...),
      so the order of the original is kept. `xtest`/`ytest` are statics and `j` is passed by pointer
      (step 2). Quirks kept: the dragon's image is always 0, the `an->state == STILL` no-op for race 7.
      Scenarios added: `swan_ride`, `swan_no_lasso`, `turtle_shell`, `door_locked`, `door_wood`; their
      state hashes were identical on a worktree build of the C `move_actor`.
- [x] **D. Drawing (~260).** Done: `draw.rs` (`repair_edges`, `draw_sprites`; guard `RSRC_DRAW`,
      prototypes in the C `#else`). `draw_sprites` is split into `weapon_passmode`, `draw_one`,
      `drawn_in_front` and `mask_against_scenery`; an actor with a weapon is drawn in two passes as
      before. It still writes the C globals the blitter reads (`shp`, `shapedata`, `mask_buffer`,
      `shift`, `aoff`, `boff`, `bmod`, `planesize`, `wmask`). `StateEntry` mirror added to `types.rs`.
      `main()` still sets `j = anix2` after the call (step 2). Checked by the 10 pixel tests and all
      41 state scenarios.
- [x] **E. Start-up and shutdown (~600).** Done: `startup.rs` (`open_all`, `close_all`, and the original
      `main` as `fta_main`, split into `start_up`, `play_intro` and `main_loop`; guard `RSRC_STARTUP`),
      `load_actors` in `encounters.rs`, `print_options` in `menus.rs`. `-Dmain=fta_main` stays in
      `build.rs`: the Rust function is simply named `fta_main`, so `src/main.rs` is unchanged. Note that
      `long` is 32 bit in this port (`#define long int` in `csrc/amiga.h`), so C `long` is `i32`
      (`move_actor`'s `i` was changed to `i32` for that). The game and the `--intro` path both run; the
      pixel and state tests cover the main loop, the intro is only checked by running it.
- [x] **F. Data tables, then drop `fmain.c` from the build.** Done: the tables are in
      `game_tables.rs` (`setfig_table`, `encounter_chart`, `trans_list`, `statelist`, `doorlist`,
      `extent_list`, `stone_list`, `inv_list`, `menus`, `letter_list`, `file_index`, colours, labels,
      ...) and every variable `fmain.c` defined is in `globals.rs` (zeroed). Names and layouts are the
      C ones. `fmain.c` is out of the file list in `build.rs` (it stays in the repo as the reference),
      and the `-Dmain=fta_main` define is gone. The table bytes were compared with the old C object
      file (all identical; pointer fields excluded). Moving the variables exposed one old bug: `loading.rs`
      declared `sample_size` as 64 bit entries but C `long` is 32 bit, so it overflowed into the next
      variable (harmless in C's layout); fixed, and `secx`/`secy` in `magic.rs` the same.
      The `RSRC_*` defines in `build.rs` do nothing now; they only matter if `fmain.c` is built again.

The port is finished. What remains afterwards (optional, no firm end) is the cleanup pass:
- Give the Rust functions readable names with thin wrappers that keep the C names for any C still
  left (`layout.c`, `statedump.c`), then delete the wrappers.
- Replace the `static mut` globals with structs (this is step 3). Done for the game state:
  `src/game/data/state.rs` has `Hero`, `World`, `Encounters`, `Actors`, `Screen`, `Memory`, `Ui` and `Witch`
  (statics `the_hero`, `the_world`, ...). `globals.rs` keeps only the Amiga base pointers, fonts,
  scratch variables and the audio ones. `statedump.c` is now `src/game/statedump.rs`, so no C reads Rust
  variables any more. All long functions are split (done).
- Unit tests instead of playtests (done): the rules are plain functions that take values or the state
  structs and return what should happen (hunger, tactics, melee and missile hits, death, revive,
  talking, shops, loot, doors, spells, riding, water, beds, the day clock). The callers do the sounds,
  text and drawing. 192 unit tests. The playtests went from 49 to 21 (13 pixel, 8 state); the ones
  left check drawing, save/load and that the game reaches the rules. `cargo clippy` is clean.
- `FTA_MUSIC_EXPORT=dir` writes every tune as a WAV file and quits (`src/game/ui/music.rs`).
- See the note below on keeping an all-C "original" build possible while doing this.

### Where the Rust files are (after the folder clean-up)
The Rust game lives in `src/game/`, grouped by purpose (it used to be a flat `rsrc/` folder named
after the C files). `csrc/` comments still use the old names; this table maps them.

| Old (`rsrc/`) | Now (`src/game/`) |
|---|---|
| `actors.rs` | `actors/interaction.rs` (hits, blocking, nearest figure, rescue) |
| `ai.rs`, `combat.rs`, `encounters.rs`, `revive.rs` | `actors/ai.rs`, `actors/combat.rs`, `actors/encounters.rs`, `actors/revive.rs` |
| `move_actor.rs` | `actors/movement.rs` |
| `world.rs` | split: `actors/carriers.rs`, `actors/spawning.rs`, `actors/nearby_speech.rs`, `world/bed.rs`, `world/travel_doors.rs`, `world/clock.rs`, `world/survival.rs`, `render/restore.rs`, `render/sprite_order.rs` |
| `asm_text.rs` | `ui/text.rs` (printing) and `actors/steering.rs` |
| `asm_map.rs`, `regions.rs`, `places.rs`, `objects.rs`, `doors.rs` | `world/map.rs`, `world/regions.rs`, `world/places.rs`, `world/objects.rs`, `world/key_doors.rs` |
| `asm_blit.rs`, `draw.rs`, `display.rs`, `screen_fx.rs`, `witch.rs` | `render/blit.rs`, `render/draw.rs`, `render/display.rs`, `render/screen_fx.rs`, `render/witch.rs` |
| `menus.rs`, `options.rs`, `items_menu.rs`, `magic.rs`, `messages.rs`, `mood.rs` | `ui/menus.rs`, `ui/options.rs`, `ui/items_menu.rs`, `ui/magic.rs`, `ui/messages.rs`, `ui/music.rs` |
| `keys.rs`, `input.rs` | `input/keys.rs`, `input/handler.rs` |
| `hdrive.rs`, `iffsubs.rs`, `iff_helpers.rs`, `loading.rs`, `savegame.rs` | `files/disk.rs`, `files/iff_picture.rs`, `files/iff_chunks.rs`, `files/loading.rs`, `files/savegame.rs` |
| `types.rs`, `state.rs`, `globals.rs`, `random.rs` | `data/types.rs`, `data/state.rs`, `data/globals.rs`, `data/random.rs` |
| `game_tables.rs`, `tables.rs` | `data/tables.rs` (people, monsters, doors, items, menus, regions), `data/object_tables.rs` (item ids, object lists, colours) |
| `startup.rs`, `asm_stubs.rs` | `startup/` (`open_close.rs`, `intro.rs`, `game_loop.rs`, `mod.rs` with `fta_main`), `startup/amiga_leftovers.rs` |
| `statedump.rs` | `statedump.rs` |

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

## No C left in the build

`build.rs`, the `cc` build dependency, `csrc/layout.c` and `csrc/narr_data.c` are gone. The story text
and tables are now `src/game/data/story_text.rs`, generated from `narr.asm` by `tools/narr2rs.py`
(a unit test had compared the bytes with the old C data before it was deleted). The struct size check
became a unit test in `src/amiga.rs`, and `main` calls `game::startup::run_game` directly. The rest of
`csrc/` is reference only and is not compiled.

## Ordinary Rust calls between game and platform

The game and `src/platform/` no longer meet through the C calling convention. There is no
`extern "C"` and no `#[no_mangle]` left; the platform functions are plain `pub fn`s, re-exported by
their module (`platform::gfx`, `exec`, `dos`, `audio`, ...) and imported with `use`. They were renamed
from the Amiga CamelCase to snake_case (`SetAPen` is `set_a_pen`, `BltBitMap` is `blt_bit_map`, `Open`
is `open_file`). The platform calls the game's input handlers (`fta_key`, `fta_mouse_move`, ...)
directly the same way. About 150 `unsafe` blocks that only wrapped those calls were removed.

## Owned memory instead of Amiga structs and raw pointers

The Amiga-style structs (`RastPort`, `BitMap`, `ViewPort`, `src/amiga.rs`) are gone. Drawing uses the
safe `gfx` module (owned bitmaps, pens, fonts, a `Scene` per frame). The big memory areas are owned
buffers in `Memory` (`image`, `sector` with the region map at `MAP_OFFSET`, `terrain`, `shadow`,
`shape`), read through `map_byte`, `sector_byte`, `terrain_byte`. The sprite blitter works on
`Loc { area, pos }` places (a bit plane of the page being drawn, the queue or mask region, or the
sprite memory) through `BlitMem`, which reads zeros outside an area and drops writes outside it;
`SeqInfo` and the background saves hold offsets, not pointers. Text is `&str` (word wrap uses a local
buffer), menu labels and item names are static data, and the save file is written field by field.
Actors and missiles are handed around as `ActorRef` / `MissileRef` (a number that looks the actor up
on each use), the brothers' item counts are `Hero::stuff()`, and the object lists are
`region_objects(n)` / `global_objects()`. No `*mut` / `*const` is left in the game code.

What is still `unsafe` is the shared game state itself: the `static mut` structs (`the_hero`,
`the_world`, `the_actors`, ...) and the `screen()`, `memory()`, `player()` accessors. Replacing those
with one `Game` value passed down the call tree is the next step.
