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
- DONE: all of `game/data` (types, the game state without the video part, the random generator,
  the tables and story text). The big tables were converted from the Rust source by a script, then
  kept by hand. Names are the Rust snake_case names in PascalCase.
- DONE: paths, disk image reads, the save file format (`Transfer`, `GameTransfer`, `SaveFiles`) and
  the sprite file table. A save written by the Rust game loads and saves back byte for byte
  (fixture in `csharp/FaeryTale.Tests/Fixtures`).
- Moved on: IFF pictures and sprite loading go with Phase 4 (they need bitmaps), the sound loading
  goes with Phase 6 (audio), and the `savegame()` menu flow goes with Phase 5 (it needs the UI).

## Phase 3: World and actors (`game/world`, `game/actors`, about 8.3k lines)
- Core game logic and the biggest risk. Needs no graphics, since the `Recorder` stands in for the screen. Port in pieces and run `state_regress.sh` after each.
- Watch integer overflow and casts: C# is unchecked by default and casts differ. Mask values explicitly.
- Naming: the Rust `world` module is the C# `Land` namespace and `actors` is `Creatures` (a namespace
  called `World` or `Actors` would clash with the state classes of those names).
- DONE and checked against real data: `world/map` (lookup, camera, steps) and `world/regions`. A
  test loads all 10 regions from the game disk and compares hashes with ones the Rust game made
  (`MapProbeTests`).
- DONE: `ui/text`, `ui/messages` and `ui/music` (the parts the world code calls). Audio goes through
  the `IAudioPlayer` interface, silent until Phase 6.
- DONE world: places, bed, clock, key_doors, objects, survival, travel_doors (`Game/Land`).
- DONE actors: encounters, carriers, spawning, steering, nearby_speech and `interaction/*`
  (`Game/Creatures`). Also `render/screen_fx` fading and message_page, `render/display` (tick waits),
  and the state dump text (`Game/Testing/StateDump`).
- DONE actors: `ai`, `combat` and `movement` (a state machine, `MoveContext`), `revive`. Also `ui/menus`
  and the sprite-loading calls (`SpriteLoading`).
- Disk "wait" functions are gone: reads finish at once, so code that checked "is the read done" just
  proceeds (comments say so where it matters).
- DONE: the test commands (`TestCommands`), so Phase 3 is complete. 148 unit tests pass.
- Still open: running whole scenarios against `state_golden.txt` needs the main loop and input
  (Phase 5), so that comparison happens after Phase 5.

## Phase 4: Graphics (`gfx` about 1.7k lines, `game/render` about 4.4k lines)
- DONE: bitmaps, drawing, fonts (with the built-in 8x8 font table, MIT licence from the font8x8
  crate), scene composing and the overlay drawing (`Game/../Gfx`); the screen, the software blitter,
  sprite drawing with scenery masks, map tiles and scrolling, the witch beam, the overview map,
  the story-page border, the compass, item list, print queue, intro zoom and win sequence, the
  IFF picture loader, and `Video` as an `IOutput` (`Game/Render`).
- Checked against Rust on the real game data: a test draws the map and sprites in two regions and
  compares hashes of the screen planes (`RenderProbeTests`), including the undo of the sprites.
- Windows go through `IDisplayDevice` (a counting stand-in until Phase 6).
- Left for Phase 5 with the launch flow: `gfx/wood` (the menu boards).

## Phase 5: Input, UI, startup (about 6.4k lines)
- DONE: input handler and keys (`Game/Input`), the rest of the UI (items menu, magic, talk, buy,
  give, use, game and save menus, saving and loading), the main loop, opening and the test path
  of start-up (`Game/Startup`), the headless platform pieces the tests need (`Platform`: key
  names, input bridge, a display that runs the test scripts and takes screenshots).
- The C# build runs the scenarios: `FTA_IMPL=csharp tests/regress.sh` and `state_regress.sh`.
- DONE, not yet checked by a test run: the splash, full intro with page turns, and the launch menu
  with the wooden boards (`Splash`, `StoryPages`, `LaunchMenu`, `MenuBackground`, `Gfx/Wood`).
- Quirk kept: `LCtrl` in a test script is not an SDL key name, so it does nothing in either
  version; the goldens depend on that.

## Phase 6: Platform (`platform`, about 2.3k lines)
- DONE: the SDL window (`Platform/SdlDisplay`: 640x400 logical size, 50 Hz pacing, keyboard, mouse,
  F11/Alt+Enter fullscreen, gamepad in `PadInput`, `controls.cfg` remap in `KeyRemap`), the audio
  engine (`Audio/`: sequencer, mixer, sound bank, SDL device), and loading of the music waveforms
  from `v6`. Runs with no window (`--headless`, or FTA_SHOT / FTA_STATE / FTA_FAST set) for tests.
- Checked: builds, runs on SDL's dummy drivers, 221 unit tests, all scenarios still `ok`.
- NOT checked: how it looks and sounds on a real screen and speakers. Audio was not compared
  against Rust output (the Rust WAV dump could be used for that).
- Not ported: FTA_AUDIO_DUMP, FTA_MUSIC_EXPORT, FTA_RECORD/FTA_REPLAY, FTA_WATCHDOG.

## Phase 7: Cleanup
- DONE: `FTA_IMPL=csharp tests/soak.sh` (random keys and clicks; checks for unhandled exceptions).
  Speed: 3000 ticks take 4 s in a Release build (about 15 times faster than the 50 Hz needed) but
  45 s in Debug (about 66 ticks a second, barely enough), so run the game with `-c Release`.
- DONE: no folder has more than 6 items (files were moved into subfolders; namespaces were left as
  they were, so no code changed). Tests are grouped the same way.
- Not done: 59 of 138 C# files have more than 6 methods (worst: `Recorder` 36, `VideoOutput` 34,
  `Screen` 33, `Sound` 20), and checking garbage-collection pauses on a real window.
- Full regression suites and the soak test (`tests/soak.sh`).
- Check frame rate and garbage-collection pauses (avoid per-frame allocations).
- Apply the project rules in `CLAUDE.md` (6 or fewer per file, and so on).
