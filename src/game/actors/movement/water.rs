//! Wading into water: sinking, drowning, climbing back out.

use super::{
    Ctx, DEAD, DYING, ENVIRON_DROWNED, ENVIRON_SINKING, PIT_SECTOR, SINK, STILL, TERRAIN_SLOW_SINK,
    TERRAIN_WADE,
};
use crate::game::data::state::Game;
use crate::game::data::state::Video;

use crate::game::world::places::{find_place, transfer_hero};

/// Depth of water that starts to matter, and how deep each wadeable terrain may be before the
/// actor climbs out: terrain 3 up to depth 5, terrain 4 up to depth 10.
const WATER_DEPTH_MIN: i8 = 2;
const WADE_LIMIT_SHALLOW: i8 = 5;
const WADE_LIMIT_DEEP: i8 = 10;

/// The hero who drowns in the pit goes to the dungeon region and arrives here.
const DUNGEON_REGION: u16 = 9;
const DUNGEON_ARRIVAL: (u16, u16) = (0x1080, 34950);

/// Standing in water (depth `k` above 2) on ground that is dry (`j` 0), or wading with a depth over
/// the limit of the terrain (3: 5, 4: 10): the actor climbs out one step a frame.
pub(super) fn climbs_out_of_water(k: i8, j: i16) -> bool {
    k > WATER_DEPTH_MIN
        && (j == 0
            || (j == TERRAIN_WADE && k > WADE_LIMIT_SHALLOW)
            || (j == TERRAIN_SLOW_SINK && k > WADE_LIMIT_DEEP))
}

/// What wading does to an actor this frame.
#[derive(Debug, PartialEq, Eq)]
enum Wading {
    Nothing,
    /// Fully under: the actor stops moving (or falls through the pit).
    Drowned,
    /// Deep enough to show the sinking frame.
    Sinking,
}

/// The depth after one frame in water of terrain type `j` (shallow 4 or deep 5), and what that does.
/// The depth moves one step towards its limit; the dead are not affected.
fn wade_step(depth: i8, j: i16, dead_or_dying: bool) -> (i8, Wading) {
    let limit: i8 = if j == TERRAIN_SLOW_SINK {
        10
    } else {
        ENVIRON_DROWNED
    };

    if depth > limit {
        (depth - 1, Wading::Nothing)
    } else if depth < limit {
        let depth = depth + 1;
        let outcome = if dead_or_dying {
            Wading::Nothing
        } else if depth == ENVIRON_DROWNED {
            Wading::Drowned
        } else if depth > ENVIRON_SINKING {
            Wading::Sinking
        } else {
            Wading::Nothing
        };

        (depth, outcome)
    } else {
        (depth, Wading::Nothing)
    }
}

impl Ctx {
    /// In water (terrain 4 is shallow, 5 deep): the depth moves one step towards its limit, and the
    /// actor sinks, or at the pit sector falls through to the dungeon.
    pub(super) fn wade(&mut self, game: &mut Game, video: &mut Video, j: i16) {
        let actor = self.actor;
        let (depth, outcome) =
            wade_step(self.environ, j, self.state == DYING || self.state == DEAD);
        self.environ = depth;

        match outcome {
            Wading::Nothing => {}
            Wading::Drowned => {
                if game.hero.sector == PIT_SECTOR {
                    if self.slot == 0 {
                        self.environ = 0;
                        game.world.new_region = DUNGEON_REGION;
                        transfer_hero(game, video, DUNGEON_ARRIVAL.0, DUNGEON_ARRIVAL.1, 0);
                        find_place(game, video, 1);
                    } else {
                        game.actor_mut(actor).vitality = 0;
                    }
                }

                game.actor_mut(actor).state = STILL;
            }

            Wading::Sinking => game.actor_mut(actor).state = SINK as i8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::ENVIRON_DROWNED;
    use super::super::ENVIRON_SINKING;
    use super::*;

    #[test]
    fn actors_climb_out_of_shallow_water() {
        assert!(climbs_out_of_water(3, 0));
        assert!(!climbs_out_of_water(2, 0));
        assert!(climbs_out_of_water(6, 3));
        assert!(!climbs_out_of_water(5, 3));
        assert!(climbs_out_of_water(11, 4));
        assert!(!climbs_out_of_water(10, 4));
        assert!(!climbs_out_of_water(20, 5));
    }

    #[test]
    fn wading_deeper_sinks_then_drowns() {
        // deep water (5): wade towards ENVIRON_DROWNED
        assert_eq!(wade_step(0, 5, false), (1, Wading::Nothing));
        assert_eq!(
            wade_step(ENVIRON_SINKING, 5, false),
            (ENVIRON_SINKING + 1, Wading::Sinking)
        );

        assert_eq!(
            wade_step(ENVIRON_DROWNED - 1, 5, false),
            (ENVIRON_DROWNED, Wading::Drowned)
        );

        assert_eq!(
            wade_step(ENVIRON_DROWNED, 5, false),
            (ENVIRON_DROWNED, Wading::Nothing)
        );

        // shallow water (4) stops at 10 and comes back up from deeper
        assert_eq!(wade_step(9, 4, false), (10, Wading::Nothing));
        assert_eq!(wade_step(10, 4, false), (10, Wading::Nothing));
        assert_eq!(wade_step(12, 4, false), (11, Wading::Nothing));

        // the dead are not affected
        assert_eq!(
            wade_step(ENVIRON_SINKING, 5, true),
            (ENVIRON_SINKING + 1, Wading::Nothing)
        );
    }
}
