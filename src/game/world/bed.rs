//! Sleeping in a bed: standing on a bed tile indoors for a while puts a tired hero to sleep (from
//! the main loop in csrc/fmain.c).

use crate::game::data::state::ActorRef;
use crate::game::data::state::Game;
use crate::game::data::state::{the_hero, the_world};
use crate::game::ui::text::show_event;
use crate::game::world::map::tile_index;

// Actor kinds and states (csrc/ftale.h).
const STILL: i8 = 13;
const SLEEP: i8 = 23;

/// Indoors region, the tiles that are beds, and how long to stand still on one before sleeping.
const INSIDE_REGION: u16 = 8;

/// Tiles are 16 pixels wide and 32 high.
const TILE_WIDTH_SHIFT: u32 = 4;
const TILE_HEIGHT_SHIFT: u32 = 5;
const BED_TILES: [u8; 4] = [161, 52, 162, 53];
const BED_WAIT: i16 = 30;
const TOO_RESTED: i16 = 50;
const EVENT_NOT_TIRED: i32 = 25;
const EVENT_SLEEPING: i32 = 26;

/// Sleeping puts the hero at the bottom of the tile.
const BED_Y_MASK: u16 = 0x1f;

/// Copies the hero's position, and puts the hero to bed after standing on a bed for 30 ticks.
pub fn check_bed(game: &mut Game) {
    unsafe {
        let mut hero = ActorRef::new(0);

        if hero.state != STILL {
            the_world.bed_wait = 0;
        }

        the_hero.x = hero.abs_x;
        the_hero.y = hero.abs_y;

        if the_world.region == INSIDE_REGION {
            let tile = game.memory.sector_byte(tile_index(
                game,
                // Shifts by TILE_WIDTH_SHIFT, converting between pixels, tiles, sectors or bytes.
                (the_hero.x >> TILE_WIDTH_SHIFT) as i32,
                // Shifts by TILE_HEIGHT_SHIFT, converting between pixels, tiles, sectors or bytes.
                (the_hero.y >> TILE_HEIGHT_SHIFT) as i32,
            ));

            let (wait, action) = bed_step(
                the_world.bed_wait,
                BED_TILES.contains(&tile),
                the_hero.fatigue,
            );

            the_world.bed_wait = wait;

            match action {
                BedAction::Nothing => {}
                BedAction::NotTired => show_event(game, EVENT_NOT_TIRED),
                BedAction::FallAsleep => {
                    show_event(game, EVENT_SLEEPING);
                    hero.abs_y |= BED_Y_MASK;
                    the_hero.y = hero.abs_y;
                    hero.state = SLEEP;
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum BedAction {
    Nothing,
    /// Stood on the bed long enough, but is too rested to sleep.
    NotTired,
    FallAsleep,
}

/// One frame indoors: `wait` counts how long the hero has stood on a bed; after `BED_WAIT` frames
/// he sleeps if tired enough. Stepping off the bed starts the count again.
fn bed_step(wait: i16, on_bed: bool, fatigue: i16) -> (i16, BedAction) {
    if !on_bed {
        return (0, BedAction::Nothing);
    }

    let wait = wait + 1;
    let action = if wait != BED_WAIT {
        BedAction::Nothing
    } else if fatigue < TOO_RESTED {
        BedAction::NotTired
    } else {
        BedAction::FallAsleep
    };

    (wait, action)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tired_hero_falls_asleep_after_thirty_frames_on_a_bed() {
        let (mut wait, mut last) = (0, BedAction::Nothing);

        for frame in 1..=BED_WAIT {
            (wait, last) = bed_step(wait, true, TOO_RESTED);
            assert_eq!(last == BedAction::FallAsleep, frame == BED_WAIT);
        }

        assert_eq!(wait, BED_WAIT);
        assert_eq!(last, BedAction::FallAsleep);
    }

    #[test]
    fn a_rested_hero_is_told_he_is_not_tired() {
        assert_eq!(
            bed_step(BED_WAIT - 1, true, TOO_RESTED - 1).1,
            BedAction::NotTired
        );
    }

    #[test]
    fn leaving_the_bed_resets_the_count() {
        assert_eq!(bed_step(20, false, 100), (0, BedAction::Nothing));
    }
}
