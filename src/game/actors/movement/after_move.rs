//! What happens at the end of an actor's move: the other states (dying, sinking, frozen ...), the sprite
//! frame, the environment, keeping the hero inside the world and working out where to draw.

use super::{
    CARRIER, CARRIER_SLOT, Ctx, DEAD, DEATH_DROWN, DEATH_LAVA, DRAGON, DYING, ENEMY,
    ENVIRON_BACKWARDS, ENVIRON_DROWNED, ENVIRON_ICE, ENVIRON_SINKING, ENVIRON_SLOW, FALL,
    FRAME_FLAT, FRAME_OSCIL, FRAME_SLEEP, FROZEN, LASSO, NO_REGION, OSCIL, OUTDOOR_REGIONS,
    PIT_LUCK_COST, PIT_XTYPE, Pc, RACE_LASSO_CARRIER, RACE_WITCH, RACE_WITCH_BODY, RAFT,
    RIDING_SWAN, SINK, SLEEP, STILL, STUFF_FIREPROOF, TALISMAN, TERRAIN_DEEP_WATER,
    TERRAIN_SLOW_SINK, WALKING, WORLD_MAX, WORLD_MIN, diroff, terrain,
};

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
                let d = self.facing;

                if (*actor).tactic > 4 {
                    self.image = if d == 0 || d > 4 { 80 } else { 81 };
                } else if (*actor).tactic > 0 {
                    self.image = if d == 0 || d > 4 { 81 } else { 80 };
                } else {
                    (*actor).state = DEAD as i8;
                    self.image = FRAME_FLAT;
                }

                the_actors.frustration = 0;
            } else if s == DEAD || s == FROZEN {
                self.image = FRAME_FLAT;
            } else if s == OSCIL {
                self.image = FRAME_OSCIL + (self.cycle & 1);
            } else if s == OSCIL + 1 {
                self.image = FRAME_OSCIL;
            } else if s == SLEEP {
                self.image = FRAME_SLEEP;
            } else if s == FALL && (*actor).tactic < 30 {
                self.terrain = (*actor).tactic as i16 / 5 + the_hero.brother * 6;
                self.image = fallstates.get(self.terrain as usize).copied().unwrap_or(0) as i16;
                (*actor).tactic += 1;
                (*actor).vel_x = ((*actor).vel_x as i32 * 3 / 4) as i8;
                (*actor).vel_y = ((*actor).vel_y as i32 * 3 / 4) as i8;
            }

            Pc::Cpx
        }
    }

    /// Works out the terrain under the actor, and carries on sliding if it is on ice.
    pub(super) unsafe fn cpx(&mut self) -> Pc {
        unsafe {
            let actor = self.actor;
            self.terrain = terrain((*actor).abs_x, (*actor).abs_y);

            if self.environ == ENVIRON_ICE {
                (*actor).abs_x = ((*actor).abs_x as i32 + (*actor).vel_x as i32 / 4) as u16;
                (*actor).abs_y = ((*actor).abs_y as i32 + (*actor).vel_y as i32 / 4) as u16;
            }

            Pc::Tail
        }
    }

    /// After the movement: finishes a death, then turns the terrain under the actor into its `environ`
    /// (water depth, ice, the pit).
    pub(super) unsafe fn tail(&mut self) -> Pc {
        unsafe {
            if self.state == DYING {
                self.finish_dying();
            }

            self.settle_environ();

            if self.environ == 0 && self.state == SINK {
                (*self.actor).state = STILL;
            }

            Pc::Raise
        }
    }

    /// A dying actor counts down its tactic; at 0 it is dead. The witch leaves her body and the
    /// talisman, the lasso keeper the lasso.
    unsafe fn finish_dying(&mut self) {
        unsafe {
            let actor = self.actor;
            let i = self.slot;

            (*actor).tactic = (*actor).tactic.wrapping_sub(1);

            if (*actor).tactic == 0 {
                (*actor).state = DEAD as i8;

                if (*actor).race == RACE_WITCH {
                    (*actor).race = RACE_WITCH_BODY;
                    (*actor).vitality = 10;
                    (*actor).state = STILL;
                    (*actor).weapon = 0;
                    leave_item(i as i16, TALISMAN);
                }

                if (*actor).race == RACE_LASSO_CARRIER {
                    leave_item(i as i16, LASSO);
                }
            }
        }
    }

    /// Turns the terrain under the actor into its `environ`: water depth, slow ground, ice, going
    /// backwards, or falling into the pit.
    unsafe fn settle_environ(&mut self) {
        unsafe {
            let actor = self.actor;
            let i = self.slot;
            let j = self.terrain;

            if i == 0 && the_actors.raft_nearness != 0 {
                self.environ = 0; // can't drown on the raft
            } else if j == 0 {
                self.environ = 0;
            } else if j == 6 {
                self.environ = ENVIRON_SLOW;
            } else if j == 7 {
                self.environ = ENVIRON_ICE;
            } else if j == 8 {
                self.environ = ENVIRON_BACKWARDS;
            } else if j == 9 && i == 0 && the_world.extent_type == PIT_XTYPE {
                if (*actor).state as i16 != FALL {
                    self.image = fallstates[(the_hero.brother * 6) as usize] as i16;
                    (*actor).state = FALL as i8;
                    (*actor).tactic = 0;
                    the_hero.luck = the_hero.luck.wrapping_sub(PIT_LUCK_COST);
                    set_mood(1);
                }

                self.environ = ENVIRON_ICE;
            } else if j == 2 {
                self.environ = 2;
            } else if j == 3 {
                self.environ = 5;
            } else if j == TERRAIN_SLOW_SINK || j == TERRAIN_DEEP_WATER {
                self.wade(j);
            }
        }
    }

    /// Stores the environ, adjusts the sprite for some enemies, keeps the hero inside the world and
    /// scrolls the map to it, and applies lava and drowning damage.
    pub(super) unsafe fn raise(&mut self) -> Pc {
        unsafe {
            let actor = self.actor;
            let i = self.slot;

            if the_world.new_region >= NO_REGION {
                (*actor).environ = self.environ;
            }

            let race = (*actor).race as i8;

            if (*actor).kind == ENEMY {
                self.enemy_frame(race);
            }

            (*actor).index = self.image as i8;

            if i == 0 {
                self.hero_wrap();
                the_hero.x = (*actor).abs_x;
                the_hero.y = (*actor).abs_y;
                move_camera_to(the_hero.x as i32, the_hero.y as i32);
                the_hero.safe_flag = self.terrain as i8;
            }

            if (*actor).vitality == 0 {
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
            let actor = self.actor;
            let i = self.slot;

            if the_actors.fiery_death != 0 {
                if i == 0 && *the_hero.stuff.add(STUFF_FIREPROOF) != 0 {
                    (*actor).environ = 0;
                } else if (*actor).environ > ENVIRON_SINKING {
                    (*actor).vitality = 0;
                } else if (*actor).environ > 2 {
                    (*actor).vitality = (*actor).vitality.wrapping_sub(1);
                }

                check_death(i, DEATH_LAVA);
            }
        }
    }

    /// A drowning actor loses a point of vitality every eighth frame (wraiths and snakes are
    /// immune).
    unsafe fn take_drowning_damage(&mut self) {
        unsafe {
            let actor = self.actor;
            let i = self.slot;
            let cycle = self.cycle as i32;
            let race = (*actor).race as i8;

            if (*actor).environ == ENVIRON_DROWNED && (cycle & 7) == 0 && race != 2 && race != 3 {
                (*actor).vitality = (*actor).vitality.wrapping_sub(1);
                check_death(i, DEATH_DROWN);
            }
        }
    }

    /// The sprite frames of snakes (race 4), the spectre (8) and the race-7 enemy at zero vitality.
    unsafe fn enemy_frame(&mut self, race: i8) {
        unsafe {
            let actor = self.actor;
            let cycle = self.cycle as i32;
            let state = (*actor).state as i16;

            if race == 4 && state < WALKING {
                self.image = ((cycle & 1) + diroff(self.facing) as i32) as i16;
            } else if race == 4 && state < DYING {
                self.image = (((cycle / 2) & 1) + diroff(self.facing) as i32) as i16;
            } else if race == 8 {
                if state == DEAD {
                    (*actor).abs_x = 0;
                } else if state == DYING {
                    self.image = 0x3f;
                } else {
                    self.image = ((cycle & 3) * 2) as i16;

                    if self.image > 4 {
                        self.image -= 1;
                    }

                    match self.slot % 3 {
                        0 => self.image = 0x25,
                        1 => self.image += 0x28,
                        _ => self.image += 0x30,
                    }
                }
            } else if race == 7 && (*actor).vitality == 0 {
                // The original has `an->state == STILL` here: a comparison that does nothing (an
                // assignment may have been meant). Kept as it was.
                self.image = 1;
            }
        }
    }

    /// The hero leaving the edge of the world comes back on the other side (outdoor regions only);
    /// whatever the hero rides is moved along.
    unsafe fn hero_wrap(&mut self) {
        unsafe {
            let actor = self.actor;

            if the_world.region >= OUTDOOR_REGIONS {
                return;
            }

            if (*actor).abs_x < WORLD_MIN {
                (*actor).abs_x = WORLD_MAX;
            } else if (*actor).abs_x > WORLD_MAX {
                (*actor).abs_x = WORLD_MIN;
            } else if (*actor).abs_y < WORLD_MIN {
                (*actor).abs_y = WORLD_MAX;
            } else if (*actor).abs_y > WORLD_MAX {
                (*actor).abs_y = WORLD_MIN;
            } else {
                return;
            }

            if the_hero.riding > 1 {
                the_actors.list[CARRIER_SLOT].abs_x = (*actor).abs_x;
                the_actors.list[CARRIER_SLOT].abs_y = (*actor).abs_y;
            }
        }
    }

    /// Where the actor is on screen: relative to the map, with the sprite's anchor offset.
    pub(super) unsafe fn statc(&mut self) {
        unsafe {
            let actor = self.actor;
            let (ox, oy) = if (*actor).kind == CARRIER && the_hero.riding == RIDING_SWAN {
                (32, 40)
            } else if (*actor).kind == RAFT || (*actor).kind == CARRIER || (*actor).kind == DRAGON {
                (16, 16)
            } else {
                (8, 26)
            };

            (*actor).rel_x =
                wrap_coordinate((*actor).abs_x as i32 - the_world.map_x as i32 - ox) as u16;
            (*actor).rel_y =
                wrap_coordinate((*actor).abs_y as i32 - the_world.map_y as i32 - oy) as u16;
        }
    }
}
