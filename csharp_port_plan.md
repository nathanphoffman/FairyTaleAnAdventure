# C# port plan

Goal: port the Rust game to C# with SDL2-CS, in a `csharp/` folder. Rust stays the reference.
Every phase is done when its scenarios match the Rust output exactly (same hashes).

## Phase 0: Setup
- Create `csharp/` with a `dotnet` solution and the SDL2-CS NuGet package.
- Open an SDL window and draw one test frame.

## Phase 1: Test harness and Output layer first
- DONE: the `IOutput` interface, the screenless `Recorder`, the test-script parsers
  (`FTA_INJECT`, `FTA_CLICK`, `FTA_CMD`), the command queue and the PPM writer, with unit tests in
  `csharp/FaeryTale.Tests`. `Hero`, `MapData` and `World` are empty placeholders until Phase 2.
- Moved to Phase 3: the state dump (`src/game/statedump.rs`), because it needs the game state.
- Accept the same flags and env vars as the Rust binary (`--skip-launch`, `FTA_INJECT`,
  `FTA_CLICK`, `FTA_CMD`, `FTA_SHOT`, `FTA_SHOT_AFTER`, `FTA_FAST`, `FTA_SAVES`) and write the same `.ppm`.
- DONE: `FTA_IMPL=csharp tests/regress.sh` (and `state_regress.sh`) runs the C# build against the
  existing golden files. It refuses `--update`, so only Rust can change the goldens.
- Still to do when the game exists: read the env vars in the main loop, and the FTA_SHOT/FTA_STATE hooks.

## Phase 2: Data and files (`game/data`, `game/files`, about 3.4k lines)
- Asset loading and the save-file format. Saves must stay compatible with the Rust version.

## Phase 3: World and actors (`game/world`, `game/actors`, about 8.3k lines)
- Core game logic and the biggest risk. Needs no graphics, since the `Recorder` stands in for the screen. Port in pieces and run `state_regress.sh` after each.
- Watch integer overflow and casts: C# is unchecked by default and casts differ. Mask values explicitly.

## Phase 4: Graphics (`gfx` about 1.7k lines, `game/render` about 4.4k lines)
- Bitmaps, fonts, drawing. First screenshot hashes should match here (`regress.sh`).

## Phase 5: Input, UI, startup (about 6.4k lines)
- Controls, menus, splash, intro, wood-board menu.

## Phase 6: Platform (`platform`, about 2.3k lines)
- Display, audio, paths on SDL2-CS. Audio is the likeliest to need tuning.

## Phase 7: Cleanup
- Full regression suites and the soak test (`tests/soak.sh`).
- Check frame rate and garbage-collection pauses (avoid per-frame allocations).
- Apply the project rules in `CLAUDE.md` (6 or fewer per file, and so on).
