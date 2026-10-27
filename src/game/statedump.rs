//! Test helpers (from csrc/statedump.c): writing the main game state as text so tests can compare
//! state instead of pixels, and the scripted test commands (teleport, kill, give, ...).
//!
//! The dump is read-only. The commands are queued from the display thread and applied at the top
//! of the game's main loop by `run_pending_command`, so they never run in the middle of a frame.

use crate::game::actors::encounters::check_death;
use crate::game::data::state::{the_actors, the_hero, the_world};
use crate::game::data::tables::{seed1, seed2};
use crate::game::data::types::Shape;
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
pub fn run_pending_command() {
    unsafe {
        let Some(command) = lock_pending().take() else {
            return;
        };

        match command {
            Command::Teleport { x, y } => {
                transfer_hero(x as u16, y as u16, 1);
            }

            Command::Give { slot, count } => *the_hero.stuff.add(slot as usize) = count as u8,
            Command::SetStat { stat: 0, value } => the_hero.hunger = value as i16,
            Command::SetStat { stat: 1, value } => the_hero.fatigue = value as i16,
            Command::SetStat { value, .. } => the_hero.wealth = value as i16,
            Command::SlayEnemies => slay_enemies(),
            Command::KillHero { luck: remaining } => {
                the_hero.luck = remaining as i16;
                the_actors.list[0].vitality = 0;
                check_death(0, DEATH_BY_TEST);
            }
        }
    }
}

/// Kills every enemy that is out, as if each had been defeated.
unsafe fn slay_enemies() {
    unsafe {
        for i in FIRST_ENEMY_SLOT..the_actors.active {
            if the_actors.list[i as usize].kind == ENEMY {
                the_actors.list[i as usize].vitality = 0;
                check_death(i as i32, 0);
            }
        }
    }
}

/// The state as text: the brother, place, stats, clock, random seeds, every actor, and the
/// inventory.
pub fn state_text() -> String {
    unsafe {
        let mut out = hero_and_world_lines();
        out.push_str(&format!("seed {} {}\n", { seed1 }, { seed2 }));

        for i in 0..20 {
            out.push_str(&actor_line(i, &*(&raw const the_actors.list[i])));
        }

        out.push_str("stuff");

        for slot in 0..DUMPED_STUFF_SLOTS {
            out.push_str(&format!(" {}", *the_hero.stuff.add(slot)));
        }

        out.push('\n');

        out
    }
}

/// The first lines of the dump: where the hero is, his stats, and the clock and light.
unsafe fn hero_and_world_lines() -> String {
    unsafe {
        let mut out = String::new();
        out.push_str(&format!(
            "brother {} region {} hero {},{} map {},{}\n",
            { the_hero.brother },
            { the_world.region },
            { the_hero.x },
            { the_hero.y },
            { the_world.map_x },
            { the_world.map_y }
        ));

        out.push_str(&format!(
            "brave {} luck {} kind {} wealth {} hunger {} fatigue {}\n",
            { the_hero.brave },
            { the_hero.luck },
            { the_hero.kindness },
            { the_hero.wealth },
            { the_hero.hunger },
            { the_hero.fatigue }
        ));

        out.push_str(&format!(
            "daynight {} light {} riding {} flying {} anix {},{}\n",
            { the_world.day_clock },
            { the_world.light_level },
            { the_hero.riding },
            { the_hero.flying },
            { the_actors.active },
            { the_actors.drawn }
        ));

        out
    }
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
pub fn dump_state(path: &Path) {
    if let Err(error) = std::fs::write(path, state_text()) {
        eprintln!("Could not write the state dump {}: {error}", path.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actor_line_lists_every_field() {
        let mut a: Shape = unsafe { std::mem::zeroed() };
        a.abs_x = 100;
        a.abs_y = 200;
        a.vitality = -3;
        a.vel_x = -1;
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
