# Reimagined plan

Status: **planning only. Nothing in the game has been changed for this yet.**

Reimagined is the third mode on the launch menu ("the remastered experience with reworked content").
Today it plays exactly like Remastered. This plan is for what makes it different.

## How it relates to the other modes
- Original -> Remastered -> Reimagined, each built on the one before. Reimagined keeps everything
  Remastered has (click to move, click to attack, the minimap and big map, the inventory screen, the
  woodgrain panel and so on) and only adds or replaces rules.
- Original and Remastered must not change at all. `tests/regress.sh` and `tests/state_regress.sh` keep
  guarding that, and Reimagined gets scenarios of its own.
- Reimagined code goes in `Game/Reimagined/`, hooked into the shared rules at small points that check
  `game.Experience == Experience.Reimagined` (the same way `Game/Remastered/` does it).

## The big idea: a tabletop RPG style fight
Today a fight is decided frame by frame. Every game frame (50 a second) each fighting actor checks
whether its weapon tip is near a target, and if so it hits, with no cooldown. Fights are over in a
fraction of a second and there is nothing to read. In Reimagined a fight is a series of attacks you can
follow, each with a dice roll shown in the text box.

### The roll
- To hit: roll **2d6 + modifier** and compare it with the target's flat **Defense**. Equal or higher hits.
- Hero modifier: weapon skill (dirk, mace, sword and so on) plus a bonus from bravery.
- Enemy modifier: from its kind (the `Arms` value in the encounter table is a start).
- Defense: one flat number for every kind of enemy (a table to tune), and one for the hero (from bravery
  and luck, since the game has no armor).
- Natural 12 (double sixes) is a critical hit with extra damage; natural 2 is a fumble that misses.
- Damage: rolled on a hit, by weapon (for example dirk 1d4, mace 1d6+1, sword 1d6+2). Kept small so
  health bars last several attacks.

### What the player reads in the text box
One short line per attack, using the message window that already scrolls back:

> Julian swings the sword: 7 + 3 = 10 against Defense 9. Hit! 4 damage.
> The wraith strikes: 5 + 2 = 7 against Defense 11. Miss.

To keep the log readable: one line per attack, enemy misses can be shortened or turned off in a setting,
and several enemies fighting at once are summarised ("3 wraiths attack: 1 hit, 2 miss").

## Pace
- **Attack rhythm:** an actor attacks once per round instead of every frame. The round length is set by
  the weapon (a dirk fast, a sword medium, a mace slow), so weapons finally differ in speed as well as
  damage. Enemies get a speed of their own.
- **Animations:** the swing animation is the same table of states, but each state is held for several
  frames, so a swing takes about a second and the hit is rolled at a set moment of the swing (the
  follow-through), not on every frame.
- **Ranged:** arrows and the wand use the same roll, with a penalty for distance.
- **Click to attack:** keeps working as it is. The hero walks up and then fights at the pace above.

## Where it hooks in (current code)
- `Melee.ResolveMelee` / `Melee.SwingAt` decide hits each frame, and `Hits.HitActor` applies the damage.
  Reimagined replaces the hit decision with the roll and moves it to once per swing.
- `Walking.Fighting` steps the swing state each frame (`Tables.SwingTransitions`). Reimagined holds each
  state longer.
- The difficulty levels (`Difficulty`) scale damage today. In Reimagined they would change enemy Defense
  and the hit modifiers instead, so the numbers in the log stay honest.
- New tables (enemy Defense and speed, weapon dice and speed) live in one data file in `Game/Reimagined/`.

## Phases
1. **Core rolls:** dice, the Defense table, hit and damage rolls, the log lines. Behind the mode check,
   with unit tests for the maths and a seeded random source so scenarios are repeatable.
2. **Pace:** attack rounds, weapon speeds, slower swing animations, the hit moment of a swing.
3. **Balance:** tune Defense, damage and health so a fight takes a few rounds, for every enemy kind.
4. **Log polish:** summaries for groups, a setting for how much to print, colours if the panel allows.
5. **More RPG layer (later, if wanted):** things like levelling or stat points from bravery, armor in the
   inventory screen, special attacks. Not decided.
6. **Other reworked content:** anything else that should be different from Remastered. Not decided.

## Tests
- Unit tests: roll maths (range, natural 2 and 12), hit versus Defense, damage ranges, weapon speeds.
- Scenarios: a scripted Reimagined fight with a fixed seed, compared with saved results like the others.
- The existing Original and Remastered scenarios must stay unchanged.

## Open questions for you
- Should the hero's Defense come from bravery and luck, or should we add armor to the inventory?
- Do enemies keep today's health numbers, or should they be raised to match the smaller damage rolls?
- Should dying and the good fairy stay as they are?
- How much text is too much in the box: every enemy attack, or only attacks that hit the hero?
- Is a critical hit for double sixes right, or do you want a different rule?

## Built so far

- **Inferno skull** (Reimagined only). It sits in the bird totem's place in the pack (slot 13, F5 on the hotbar, "Fire" in the
  classic Magic menu), so the world's bird totem is a skull here. Casting sends a fireball the way the hero faces
  (`Game/Reimagined/Magic/InfernoSkull.cs`). It bursts on an enemy, a wall, or after 40 frames, lights 14 animated flames and
  does 30 to 37 damage to every enemy within 80 pixels. It never hurts the hero or friendly people. Each cast uses one
  skull. The GRAPHIX cheat gives 5. The fireball and flames use the wand blast picture (a blue crystal) for now.
