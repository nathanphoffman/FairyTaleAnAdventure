//! What happens at the end of an actor's move: the other states (dying, sinking, frozen ...), the sprite
//! frame, the environment, keeping the hero inside the world and working out where to draw.

use super::{
    ANCHOR_LARGE, ANCHOR_PERSON, ANCHOR_SWAN, CARRIER, CARRIER_SLOT, Ctx, DEAD, DEATH_DROWN,
    DEATH_LAVA, DRAGON, DROWN_IMMUNE_RACES, DROWN_PERIOD_MASK, DYING, DYING_FACING_LIMIT,
    DYING_FIRST_HALF, ENEMY, ENVIRON_BACKWARDS, ENVIRON_DROWNED, ENVIRON_ICE, ENVIRON_SHALLOW,
    ENVIRON_SINKING, ENVIRON_SLOW, ENVIRON_WADE, FALL, FALL_DRAG, FALL_FRAMES, FALL_STEPS,
    FALL_STEPS_PER_FRAME, FRAME_DYING, FRAME_FLAT, FRAME_OSCIL, FRAME_SLEEP, FROZEN, LASSO,
    NO_REGION, OSCIL, OUTDOOR_REGIONS, PIT_LUCK_COST, PIT_XTYPE, Pc, RACE_LASSO_CARRIER,
    RACE_SNAKE, RACE_SPECTRE, RACE_SPECTRE_ALIVE, RACE_WITCH, RACE_WITCH_BODY, RAFT, RIDING_SWAN,
    SINK, SLEEP, SPECTRE_BASE_FRAMES, SPECTRE_CYCLE_MASK, SPECTRE_DEAD_FRAME, SPECTRE_FRAME_GROUPS,
    SPECTRE_SKIP_FRAME, STILL, STUFF_FIREPROOF, TALISMAN, TERRAIN_BACKWARDS, TERRAIN_DEEP_WATER,
    TERRAIN_ICE, TERRAIN_PIT, TERRAIN_SHALLOW, TERRAIN_SLOW, TERRAIN_SLOW_SINK, TERRAIN_WADE,
    VELOCITY_QUARTER, WALKING, WITCH_BODY_VITALITY, WORLD_MAX, WORLD_MIN, diroff, terrain,
};
use crate::game::data::state::Game;
use crate::game::data::state::player;

use crate::game::actors::encounters::check_death;
use crate::game::data::object_tables::fallstates;
use crate::game::data::state::{the_actors, the_hero, the_world};
use crate::game::ui::music::set_mood;
use crate::game::world::map::{move_camera_to, wrap_coordinate};
use crate::game::world::objects::leave_item;

impl Ctx {
    /// The remaining states: dying, dead, frozen, oscillating, asleep and falling.
    pub(super) unsafe fn other_states(&mut self) -> Pc {
        unsafe {
            let actor = self.actor;
            let s = self.state;

            if s == DYING {
                self.dying_frame();
            } else if s == DEAD || s == FROZEN {
                self.image = FRAME_FLAT;
            } else if s == OSCIL {
                // Masks with FRAME_OSCIL, keeping only the bits that matter here.
                self.image = FRAME_OSCIL + (self.cycle & 1);
            } else if s == OSCIL + 1 {
                self.image = FRAME_OSCIL;
            } else if s == SLEEP {
                self.image = FRAME_SLEEP;
            } else if s == FALL && actor.tactic < FALL_STEPS {
                self.falling_frame();
            }

            Pc::Cpx
        }
    }

    /// A dying actor flips between two frames, which way round depends on its facing; when the
    /// countdown (the tactic) ends it is dead and lies flat.
    unsafe fn dying_frame(&mut self) {
        unsafe {
            let mut actor = self.actor;
            let d = self.facing;
            let facing_back = d == 0 || d > DYING_FACING_LIMIT;

            if actor.tactic > DYING_FIRST_HALF {
                self.image = if facing_back {
                    FRAME_DYING.0
                } else {
                    FRAME_DYING.1
                };
            } else if actor.tactic > 0 {
                self.image = if facing_back {
                    FRAME_DYING.1
                } else {
                    FRAME_DYING.0
                };
            } else {
                actor.state = DEAD as i8;
                self.image = FRAME_FLAT;
            }

            the_actors.frustration = 0;
        }
    }

    /// One step of the fall into the pit: the next falling frame, and the slide slows down.
    unsafe fn falling_frame(&mut self) {
        unsafe {
            let mut actor = self.actor;
            self.terrain =
                actor.tactic as i16 / FALL_STEPS_PER_FRAME + the_hero.brother * FALL_FRAMES;
            self.image = fallstates.get(self.terrain as usize).copied().unwrap_or(0) as i16;
            actor.tactic += 1;
            actor.vel_x = (actor.vel_x as i32 * FALL_DRAG.0 / FALL_DRAG.1) as i8;
            actor.vel_y = (actor.vel_y as i32 * FALL_DRAG.0 / FALL_DRAG.1) as i8;
        }
    }

    /// Works out the terrain under the actor, and carries on sliding if it is on ice.
    pub(super) fn cpx(&mut self) -> Pc {
        let mut actor = self.actor;
        self.terrain = terrain(actor.abs_x, actor.abs_y);

        if self.environ == ENVIRON_ICE {
            actor.abs_x = (actor.abs_x as i32 + actor.vel_x as i32 / VELOCITY_QUARTER) as u16;
            actor.abs_y = (actor.abs_y as i32 + actor.vel_y as i32 / VELOCITY_QUARTER) as u16;
        }

        Pc::Tail
    }

    /// After the movement: finishes a death, then turns the terrain under the actor into its `environ`
    /// (water depth, ice, the pit).
    pub(super) unsafe fn tail(&mut self, game: &mut Game) -> Pc {
        unsafe {
            if self.state == DYING {
                self.finish_dying();
            }

            self.settle_environ(game);

            if self.environ == 0 && self.state == SINK {
                self.actor.state = STILL;
            }

            Pc::Raise
        }
    }

    /// A dying actor counts down its tactic; at 0 it is dead. The witch leaves her body and the
    /// talisman, the lasso keeper the lasso.
    fn finish_dying(&mut self) {
        let mut actor = self.actor;
        let i = self.slot;

        actor.tactic = actor.tactic.wrapping_sub(1);

        if actor.tactic == 0 {
            actor.state = DEAD as i8;

            if actor.race == RACE_WITCH {
                actor.race = RACE_WITCH_BODY;
                actor.vitality = WITCH_BODY_VITALITY;
                actor.state = STILL;
                actor.weapon = 0;
                leave_item(i as i16, TALISMAN);
            }

            if actor.race == RACE_LASSO_CARRIER {
                leave_item(i as i16, LASSO);
            }
        }
    }

    /// Turns the terrain under the actor into its `environ`: water depth, slow ground, ice, going
    /// backwards, or falling into the pit.
    unsafe fn settle_environ(&mut self, game: &mut Game) {
        unsafe {
            let mut actor = self.actor;
            let i = self.slot;
            let j = self.terrain;

            if i == 0 && the_actors.raft_nearness != 0 {
                self.environ = 0; // can't drown on the raft
            } else if j == 0 {
                self.environ = 0;
            } else if j == TERRAIN_SLOW {
                self.environ = ENVIRON_SLOW;
            } else if j == TERRAIN_ICE {
                self.environ = ENVIRON_ICE;
            } else if j == TERRAIN_BACKWARDS {
                self.environ = ENVIRON_BACKWARDS;
            } else if j == TERRAIN_PIT && i == 0 && the_world.extent_type == PIT_XTYPE {
                if actor.state as i16 != FALL {
                    self.image = fallstates[(the_hero.brother * FALL_FRAMES) as usize] as i16;
                    actor.state = FALL as i8;
                    actor.tactic = 0;
                    the_hero.luck = the_hero.luck.wrapping_sub(PIT_LUCK_COST);
                    set_mood(1);
                }

                self.environ = ENVIRON_ICE;
            } else if j == TERRAIN_SHALLOW {
                self.environ = ENVIRON_SHALLOW;
            } else if j == TERRAIN_WADE {
                self.environ = ENVIRON_WADE;
            } else if j == TERRAIN_SLOW_SINK || j == TERRAIN_DEEP_WATER {
                self.wade(game, j);
            }
        }
    }

    /// Stores the environ, adjusts the sprite for some enemies, keeps the hero inside the world and
    /// scrolls the map to it, and applies lava and drowning damage.
    pub(super) unsafe fn raise(&mut self) -> Pc {
        unsafe {
            let mut actor = self.actor;
            let i = self.slot;

            if the_world.new_region >= NO_REGION {
                actor.environ = self.environ;
            }

            let race = actor.race as i8;

            if actor.kind == ENEMY {
                self.enemy_frame(race);
            }

            actor.index = self.image as i8;

            if i == 0 {
                self.hero_wrap();
                the_hero.x = actor.abs_x;
                the_hero.y = actor.abs_y;
                move_camera_to(the_hero.x as i32, the_hero.y as i32);
                the_hero.safe_flag = self.terrain as i8;
            }

            if actor.vitality == 0 {
                return Pc::Statc;
            }

            self.take_lava_damage();
            self.take_drowning_damage();

            Pc::Statc
        }
    }

    /// While lava is flowing the actor in deep lava dies and one wading in it is hurt, unless the
    /// hero carries the fireproof item.
    unsafe fn take_lava_damage(&mut self) {
        unsafe {
            let mut actor = self.actor;
            let i = self.slot;

            if the_actors.fiery_death != 0 {
                if i == 0 && player().stuff()[STUFF_FIREPROOF] != 0 {
                    actor.environ = 0;
                } else if actor.environ > ENVIRON_SINKING {
                    actor.vitality = 0;
                } else if actor.environ > ENVIRON_SHALLOW {
                    actor.vitality = actor.vitality.wrapping_sub(1);
                }

                check_death(i, DEATH_LAVA);
            }
        }
    }

    /// A drowning actor loses a point of vitality every eighth frame (wraiths and snakes are
    /// immune).
    fn take_drowning_damage(&mut self) {
        let mut actor = self.actor;
        let i = self.slot;
        let cycle = self.cycle as i32;
        let race = actor.race as i8;

        if actor.environ == ENVIRON_DROWNED
                // Masks with DROWN_PERIOD_MASK, keeping only the bits that matter here.
                && (cycle & DROWN_PERIOD_MASK) == 0
                && !DROWN_IMMUNE_RACES.contains(&race)
        {
            actor.vitality = actor.vitality.wrapping_sub(1);
            check_death(i, DEATH_DROWN);
        }
    }

    /// The sprite frames of snakes (race 4), the spectre (8) and the race-7 enemy at zero vitality.
    fn enemy_frame(&mut self, race: i8) {
        let mut actor = self.actor;
        let cycle = self.cycle as i32;
        let state = actor.state as i16;

        if race == RACE_SNAKE as i8 && state < WALKING {
            // Masks off the low bits (a cheap remainder or a flag test).
            self.image = ((cycle & 1) + diroff(self.facing) as i32) as i16;
        } else if race == RACE_SNAKE as i8 && state < DYING {
            // Masks off the low bits (a cheap remainder or a flag test).
            self.image = (((cycle / 2) & 1) + diroff(self.facing) as i32) as i16;
        } else if race == RACE_SPECTRE {
            if state == DEAD {
                actor.abs_x = 0;
            } else if state == DYING {
                self.image = SPECTRE_DEAD_FRAME;
            } else {
                // Masks with SPECTRE_CYCLE_MASK, keeping only the bits that matter here.
                self.image = ((cycle & SPECTRE_CYCLE_MASK) * 2) as i16;

                if self.image > SPECTRE_SKIP_FRAME {
                    self.image -= 1;
                }

                match self.slot % SPECTRE_FRAME_GROUPS {
                    0 => self.image = SPECTRE_BASE_FRAMES.0,
                    1 => self.image += SPECTRE_BASE_FRAMES.1,
                    _ => self.image += SPECTRE_BASE_FRAMES.2,
                }
            }
        } else if race == RACE_SPECTRE_ALIVE && actor.vitality == 0 {
            // The original has `an->state == STILL` here: a comparison that does nothing (an
            // assignment may have been meant). Kept as it was.
            self.image = 1;
        }
    }

    /// The hero leaving the edge of the world comes back on the other side (outdoor regions only);
    /// whatever the hero rides is moved along.
    unsafe fn hero_wrap(&mut self) {
        unsafe {
            let mut actor = self.actor;

            if the_world.region >= OUTDOOR_REGIONS {
                return;
            }

            if actor.abs_x < WORLD_MIN {
                actor.abs_x = WORLD_MAX;
            } else if actor.abs_x > WORLD_MAX {
                actor.abs_x = WORLD_MIN;
            } else if actor.abs_y < WORLD_MIN {
                actor.abs_y = WORLD_MAX;
            } else if actor.abs_y > WORLD_MAX {
                actor.abs_y = WORLD_MIN;
            } else {
                return;
            }

            if the_hero.riding > 1 {
                the_actors.list[CARRIER_SLOT].abs_x = actor.abs_x;
                the_actors.list[CARRIER_SLOT].abs_y = actor.abs_y;
            }
        }
    }

    /// Where the actor is on screen: relative to the map, with the sprite's anchor offset.
    pub(super) unsafe fn statc(&mut self) {
        unsafe {
            let mut actor = self.actor;
            let (ox, oy) = if actor.kind == CARRIER && the_hero.riding == RIDING_SWAN {
                ANCHOR_SWAN
            } else if actor.kind == RAFT || actor.kind == CARRIER || actor.kind == DRAGON {
                ANCHOR_LARGE
            } else {
                ANCHOR_PERSON
            };

            actor.rel_x = wrap_coordinate(actor.abs_x as i32 - the_world.map_x as i32 - ox) as u16;
            actor.rel_y = wrap_coordinate(actor.abs_y as i32 - the_world.map_y as i32 - oy) as u16;
        }
    }
}
