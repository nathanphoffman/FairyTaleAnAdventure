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
