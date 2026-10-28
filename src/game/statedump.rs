//! Test helpers (from csrc/statedump.c): writing the main game state as text so tests can compare
//! state instead of pixels, and the scripted test commands (teleport, kill, give, ...).
//!
//! The dump is read-only. The commands are queued from the display thread and applied at the top
//! of the game's main loop by `run_pending_command`, so they never run in the middle of a frame.

use crate::game::actors::encounters::check_death;
use crate::game::data::state::Game;
use crate::game::data::types::Shape;
use crate::game::output::Output;
use crate::game::world::places::transfer_hero;
use std::path::Path;
use std::sync::Mutex;

/// Number of inventory slots the dump lists (the weapons and keys, up to the arrows).
const DUMPED_STUFF_SLOTS: usize = 35;

/// Actor kind of enemies.
const ENEMY: i8 = 2;

/// First actor slot that can be an enemy.
const FIRST_ENEMY_SLOT: i16 = 3;

/// Death type used when the "kill the hero" command ends the game.
const DEATH_BY_TEST: i32 = 5;

/// The commands a test script can queue (`FTA_CMD`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Command {
    /// Move the hero to the point, like going through a door.
    Teleport { x: i32, y: i32 },
    /// Kill the hero, who has the given luck left.
    KillHero { luck: i32 },
    /// Set inventory slot `slot` to `count`.
    Give { slot: i32, count: i32 },
    /// Set a stat: 0 hunger, 1 fatigue, anything else wealth.
    SetStat { stat: i32, value: i32 },
    /// Kill every enemy that is out.
    SlayEnemies,
}

/// The queued command. The display thread writes it and the game thread reads it.
static PENDING: Mutex<Option<Command>> = Mutex::new(None);

/// The pending command. A thread that panicked while holding the lock leaves the command intact, so
/// the lock is used anyway.
fn lock_pending() -> std::sync::MutexGuard<'static, Option<Command>> {
    PENDING
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Queues a command to run at the start of the next frame (a later one replaces an earlier one).
pub fn queue_command(command: Command) {
    *lock_pending() = Some(command);
}

/// Runs the queued command, if any. Called at the top of every frame.
pub fn run_pending_command(game: &mut Game, out: &mut dyn Output) {
    let Some(command) = lock_pending().take() else {
        return;
    };

    match command {
        Command::Teleport { x, y } => {
            transfer_hero(game, out, x as u16, y as u16, 1);
        }

        Command::Give { slot, count } => game.hero.stuff_mut()[slot as usize] = count as u8,
        Command::SetStat { stat: 0, value } => game.hero.hunger = value as i16,
        Command::SetStat { stat: 1, value } => game.hero.fatigue = value as i16,
        Command::SetStat { value, .. } => game.hero.wealth = value as i16,
        Command::SlayEnemies => slay_enemies(game, out),
        Command::KillHero { luck: remaining } => {
            game.hero.luck = remaining as i16;
            game.actors.list[0].vitality = 0;
            check_death(game, out, 0, DEATH_BY_TEST);
        }
    }
}

/// Kills every enemy that is out, as if each had been defeated.
fn slay_enemies(game: &mut Game, out: &mut dyn Output) {
    for i in FIRST_ENEMY_SLOT..game.actors.active {
        if game.actors.list[i as usize].kind == ENEMY {
            game.actors.list[i as usize].vitality = 0;
            check_death(game, out, i as i32, 0);
        }
    }
}

/// The state as text: the brother, place, stats, clock, random seeds, every actor, and the
/// inventory.
pub fn state_text(game: &mut Game) -> String {
    let mut out = hero_world_lines(game);
    out.push_str(&format!("seed {} {}\n", { game.rng.seed1 }, {
        game.rng.seed2
    }));

    for i in 0..20 {
        out.push_str(&actor_line(i, &game.actors.list[i]));
    }

    out.push_str("stuff");

    for slot in 0..DUMPED_STUFF_SLOTS {
        out.push_str(&format!(" {}", game.hero.stuff()[slot]));
    }

    out.push('\n');

    out
}

/// The first lines of the dump: where the hero is, his stats, and the clock and light.
fn hero_world_lines(game: &mut Game) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "brother {} region {} hero {},{} map {},{}\n",
        { game.hero.brother },
        { game.world.region },
        { game.hero.x },
        { game.hero.y },
        { game.world.map_x },
        { game.world.map_y }
    ));

    out.push_str(&format!(
        "brave {} luck {} kind {} wealth {} hunger {} fatigue {}\n",
        { game.hero.brave },
        { game.hero.luck },
        { game.hero.kindness },
        { game.hero.wealth },
        { game.hero.hunger },
        { game.hero.fatigue }
    ));

    out.push_str(&format!(
        "daynight {} light {} riding {} flying {} anix {},{}\n",
        { game.world.day_clock },
        { game.world.light_level },
        { game.hero.riding },
        { game.hero.flying },
        { game.actors.active },
        { game.actors.drawn }
    ));

    out
}

/// One line describing actor `i`.
fn actor_line(i: usize, a: &Shape) -> String {
    format!(
        "actor {} pos {},{} type {} race {} idx {} wpn {} env {} goal {} tac {} state {} face {} vit {} vel {},{}\n",
        i,
        a.abs_x,
        a.abs_y,
        a.kind,
        a.race,
        a.index,
        a.weapon,
        a.environ,
        a.goal,
        a.tactic,
        a.state,
        a.facing,
        a.vitality,
        a.vel_x,
        a.vel_y
    )
}

/// Writes `state_text` to the file at `path`.
pub fn dump_state(game: &mut Game, path: &Path) {
    if let Err(error) = std::fs::write(path, state_text(game)) {
        eprintln!("Could not write the state dump {}: {error}", path.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actor_line_lists_every_field() {
        let a = Shape {
            abs_x: 100,
            abs_y: 200,
            vitality: -3,
            vel_x: -1,
            ..Default::default()
        };
        assert_eq!(
            actor_line(4, &a),
            "actor 4 pos 100,200 type 0 race 0 idx 0 wpn 0 env 0 goal 0 tac 0 state 0 face 0 vit -3 vel -1,0\n"
        );
    }

    #[test]
    fn a_later_command_replaces_the_queued_one() {
        queue_command(Command::SlayEnemies);
        queue_command(Command::Give { slot: 1, count: 2 });
        assert_eq!(
            lock_pending().take(),
            Some(Command::Give { slot: 1, count: 2 })
        );
    }
}
