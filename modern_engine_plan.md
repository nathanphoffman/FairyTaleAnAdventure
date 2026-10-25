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
- [ ] Still without scenarios: caves/underground, swan/turtle, key doors, combat kills.
- [x] Record and replay: `FTA_RECORD=file` logs keys and mouse; `FTA_REPLAY=file` repeats them.
      Checked: replay gives an identical state file.
- [x] `regress.sh` golden pixel hashes were stale (they failed on the previous commit too); screens checked by eye, hashes updated. `soak.sh` passes.

### 2. Cut `main()` into functions (still C, no behaviour change)
Input, hero update, actor update, combat, spawning, render, menus. Run tests after each cut.
- [x] `handle_key` (key and menu handling at the top of the loop). State and pixel tests unchanged.
- [x] `update_hero` (timers, dead/fairy, shoot/ride/fight/walk state). Tests unchanged.
Remaining blocks inside the `main()` loop of `csrc/fmain.c` (line numbers as of commit 184cc37,
they move after each cut; the `/* ... */` comments in the loop mark most boundaries):
- [ ] ~1466 raft/turtle proximity setup (10 lines). Do together with the next one.
- [ ] ~1478-1876 the per-actor motion loop `for (i=0; i<anix; i++)` (about 400 lines, has `goto`
      labels such as `newloc`, `blocked`, `checkdev2`). Biggest and riskiest. Split it by actor
      kind (hero, carrier/raft, setfig, enemy) only where the gotos stay inside one piece.
- [ ] ~1877 "check for bed" and "update map position"
- [ ] ~1902 "check for door" (about 65 lines)
- [ ] ~1968 "update map position" and "generate map", ~1985 "undo witch fx"
- [ ] ~1990-2245 map scroll/redraw (`scrollmap`, `map_draw`, `viewchange:`) and the per-tick
      world updates: day/night, encounter spawning (~2068), mood/music (~2195-2208)
- [ ] ~2248-2315 combat hit detection loop (weapon reach, `newx`/`newy`, damage)
- [ ] ~2316 flying missiles, ~2337 sprite sort (bubble sort), ~2340 `drawnow:`
- [ ] ~2371 finish map repair, ~2376 "do witch fx"
- [ ] ~2395-2626 sprite drawing loop (masking, `passmode`, `blithigh`; about 230 lines)
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
Left off: everything except `fmain.c`, `fmain2.c` and the files that stay C is ported. `fmain2.c` is next.
#### `fmain2.c` split (1790 lines, about 50 functions)
Port it as groups, one commit per group, each into its own `rsrc/` file (K&R functions keep their
C names; globals defined in `fmain2.c` move with their group and keep their names). Order is leaf
groups first:
1. `messages.rs`: `print`, `print_cont`, `extract`, `announce_container`, `announce_treasure`,
   `name`, `map_message`, `message_off`, plus `mesbuf`/`mbuf`. Text only, easy to check by eye.
2. `savegame.rs`: `savegame`, `saveload`, `sl_*` (already big-endian field by field), `locktest`,
   `waitnewdisk`. Checked by the save/load round trip in `tests/regress.sh`.
3. `objects.rs`: `do_objects`, `leave_item`, `change_object`, `set_objects`.
4. `loading.rs`: `shape_read`, `read_shapes`, `seekn`, `prep`, `load_next`, `read_score`,
   `copypage`, `flipscan`, `skipint`, `into_chip`, `cpytest`, `copy_protect_junk`.
5. `actors.rs`: `dohit`, `aftermath`, `proxcheck`, `nearest_fig`, `calc_dist`, `move_figure`,
   `do_tactic`, `move_extent`, `rescue`, `eat`, `set_loc`, `turtle_eggs`.
6. `screen_fx.rs`: `drawcompass` (+ `comptable`), `fade_page`, `colorplay`, `ppick`, `fade_down`,
   `fade_normal`, `stillscreen`, `win_colors`, `day_fade`.
7. `witch.rs`: `witch_fx` alone (about 270 lines, the riskiest; split into steps inside).
Dead code stays behind: the `AMIGA_ASM_68K` blocks and `#if 0` sections are not ported.
The `static` helpers and globals stay private to their file unless `fmain.c` names them.

Decision for `fmain.c`: cut `main()` into functions in `csrc/` first (step 2, the method above), then
port the pieces to `rsrc/`. This is the one exception to "csrc is never edited": only step-2 cuts, with
no behaviour change. Do the cuts before starting `fmain.c` in `rsrc/`; `fmain2.c` can be ported earlier.

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
