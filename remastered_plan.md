# Remastered plan

The launch menu has three modes: Original, Remastered ("the original with modern conveniences")
and Reimagined (Remastered plus new content). The rules are shared. A mode is chosen with
`game.Experience`, and Original must always behave exactly as before (the regression scenarios
check this).

## How the modes share code
- The rules stay in `Game/`. New behaviour that only some modes want goes in `Game/Remastered/`.
- A new thing hooks into the rules at one small point and checks `game.Experience`.
- Only when a mode needs a different rule do we split that one piece into its own file.

## Phase R1: mouse over the whole screen, click to move (DONE)
- In Remastered and Reimagined the pointer moves over the whole screen (Original keeps it in the
  text panel). Clicks in the panel still work the menus and the compass.
- A left click on the play field sets a destination. `PathFinder` (A* on a grid of 8 pixel
  cells) plans the way, and `ClickToMove` steers the hero along it by setting the walk direction,
  so the hero uses the normal movement code and its collision rules.
- "Can I step here" is the game's own `Blocking.BlockedAt`, not a second copy. Deep water is
  avoided and shallow water costs extra, so the route stays on land when it can.
- The arrow keys, the compass, fighting and menus work as before and cancel the walk.
- If the hero gets stuck (a monster in the way) the route is planned again, then given up.
- Test scenario: `FTA_EXPERIENCE=remastered` starts a run in that mode (for the scripts).

## Next ideas (not started)
- Hold the button to keep walking towards the pointer; click an actor to talk or fight.
- Show a marker at the destination; draw the route.
- A map screen, bigger menus, upscaled graphics.

## Difficulty levels (DONE)
- Explorer, Easy, Normal (default, easier than the original), Hard (the original's numbers), Very Hard
  and Extreme. Each has four coefficients: enemy health, enemy damage, player health, player damage
  (`Game/Remastered/Difficulty.cs`). Every hit goes through `Hits.HitActor`, where the damage is scaled
  (a hit on the hero by enemy damage over player health, a hit by the hero by player damage over
  enemy health); the part of a point that does not come out whole is carried to the next hit.
- Changed from the game menu (Esc) while playing; the level is kept for the session.
- Not scaled: hunger, lava and other damage that does not come from a hit.

## World drawer (DONE)
- `Game/Remastered/World`: `WorldCapture` notes every sprite the old drawing places (the old drawing
  still runs underneath, so the game state is exactly the same), `WorldImages` decodes the tile and
  sprite bitplanes once, `WorldRenderer` draws the tiles and sprites from the camera at any size and
  zoom (with the same palette, so day, night and fades match) and hides sprites behind scenery with
  the old drawing's own rules and masks, `WorldView` is the one transform between window pixels and
  world positions (the mouse, click to move and the door icon all use it).
- `Platform/Window/Native`: `NativeComposer` puts the world, the text panel (bottom middle), the menus
  and the pointer together at the window's own size; story pages and the item list are the old picture
  scaled up and centred. `SdlDisplay.Native` runs borderless fullscreen, the zoom (wheel, + and -),
  and several pictures per game tick: the ones between ticks draw the camera and the characters part
  of the way along their last move, worked out from the clock for the moment each picture is shown.
  The rate is the monitor's own refresh (vsync, default) or a fixed 50 to 300, set in the Esc menu or
  with `FTA_FPS`. The scale is always a whole number (no shimmer) and the map is drawn on all cores.
- Test hooks for the headless build: `FTA_NATIVE=1920x1080` and `FTA_ZOOM=0.5` size and zoom the
  screenshots of a Remastered run; `FTA_FPS_LOG=1` prints the pictures a second of the real window.
- Not done: objects and enemies far from the old screen are only loaded by the game when they are
  near the old screen's area, so at 4x zoom out some stay out of sight until you get closer; the world
  outside the loaded region is black.
