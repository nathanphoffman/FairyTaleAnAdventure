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
  The rate is a fixed 50 to 300 (default 150) or the monitor's own refresh (vsync, off by default), set in the Esc menu or
  with `FTA_FPS`. The scale is always a whole number (no shimmer) and the map is drawn on all cores.
- Test hooks for the headless build: `FTA_NATIVE=1920x1080` and `FTA_ZOOM=0.5` size and zoom the
  screenshots of a Remastered run; `FTA_FPS_LOG=1` prints the pictures a second of the real window.
- Not done: objects and enemies far from the old screen are only loaded by the game when they are
  near the old screen's area, so at 4x zoom out some stay out of sight until you get closer; the world
  outside the loaded region is black.

## Talking, looking, taking, maps (DONE; Remastered and Reimagined only)
- `Game/Remastered/Conversation`: hover a person for their face (cut from their sprite), click to walk up
  (to under 35 pixels, so a Yell is told off as in the original) and open a bubble with Yell, Say, Ask and,
  for a bartender, Buy (a list of the things for sale, greyed when too dear, on a different wood).
- Shift: the pointer is an eye (`EyeCursor`); a shift click walks to the spot (not through a door) and
  does the Look action.
- Click on water swims in; click on the black outside of a room indoors walks to the exit door.
- `Game/Remastered/Interaction`: a hand pointer over a chest or anything Take picks up; a click walks
  over and takes that one.
- `MenuGreying`: the classic Buy and Give boxes are dark grey with no bartender / nobody near.
- `Game/Remastered/Messages`: `MessageHistory` keeps the printed lines; the mouse wheel over the text
  panel, Page Up and Page Down scroll the beige message window back through them.
- `Game/Remastered/Map`: `WorldOverview` paints the whole outdoor world once (a pixel a tile, from the
  game's own overview colours); `MinimapPicture` is the corner map; M opens `MapScreen` (drag or arrows
  scroll, wheel zooms, C finds the hero, M or Esc closes). The music key moved from M to J in these modes.
- Red squares round the classic Talk, Take and Look boxes while the shortcut for them is in use (`PanelButtonOutline`).
  Greyed Buy and Give boxes cannot be pressed (mouse or letter). The message window has a scroll bar.
- Indoors only the building the hero is in is drawn, however far out the view is zoomed (`InteriorMask`).
- The minimap shows 1.5 times more land than one tile to a unit. Vsync starts off (150 pictures a second).
- Click to attack (`Game/Remastered/Combat`): a click on an enemy walks the hero up and holds the fight button
  (`Ui.PathFighting`) until it is dead; a click on another enemy changes the target, any other click ends it.
  A red crosshair is the pointer over an attackable enemy and sits on the one being attacked. The blocked
  click circle is yellow now. A click while the game is paused gives the walk or attack order, carried out
  when the game goes on.
- The game menu has Disable Map / Enable Map (`MapSettings`), and "Back To Main Menu" and "Exit Game" are set apart
  at the bottom. People and things load much further out (`Objects.WIDE_*`), all from one picture file at a time.
  The menu music starts again whenever the main menu comes back.
- A "< Back" button above the classic buttons while a submenu is open (`PanelBackButton`). The black behind the
  text panel is a wooden board (`PanelBackground`; the black labels inside the menu boxes stay).
- Inventory (`Game/Remastered/Inventory`): I or the classic List button opens a stone panel over the running game
  (the old full-screen list never shows in these modes). Weapons, Magic, Supplies, Keys and Treasures, each item in a
  square with a checkerboard edge and a large picture (`ItemIcons`, from the object pictures); not carried is dim, the
  weapon in hand has a gold ring. A click equips a weapon, casts a stone, tries a key or uses the shell and sun stone
  through the game's own menus; X or I closes it; clicks outside it go to the world.
- Not done: a map of the indoors; the minimap and big map show the daylight colours only.
