//! What happens at the end of an actor's move: the other states (dying, sinking, frozen ...), the sprite
//! frame, the environment, keeping the hero inside the world and working out where to draw.

use super::*;

impl Ctx {
    /// The remaining states: dying, dead, frozen, oscillating, asleep and falling.
    pub(super) unsafe fn other_states(&mut self) -> Pc {
        unsafe {
            let an = self.an;
            let s = self.s;
            if s == DYING {
                let d = self.d;
                if (*an).tactic > 4 {
                    self.dex = if d == 0 || d > 4 { 80 } else { 81 };
                } else if (*an).tactic > 0 {
                    self.dex = if d == 0 || d > 4 { 81 } else { 80 };
                } else {
                    (*an).state = DEAD as i8;
                    self.dex = FRAME_FLAT;
                }
                the_actors.frustration = 0;
            } else if s == DEAD || s == FROZEN {
                self.dex = FRAME_FLAT;
            } else if s == OSCIL {
                self.dex = FRAME_OSCIL + (self.cycle & 1);
            } else if s == OSCIL + 1 {
                self.dex = FRAME_OSCIL;
            } else if s == SLEEP {
                self.dex = FRAME_SLEEP;
            } else if s == FALL && (*an).tactic < 30 {
                self.j = (*an).tactic as i16 / 5 + the_hero.brother * 6;
                self.dex = fallstates.get(self.j as usize).copied().unwrap_or(0) as i16;
                (*an).tactic += 1;
                (*an).vel_x = ((*an).vel_x as i32 * 3 / 4) as i8;
                (*an).vel_y = ((*an).vel_y as i32 * 3 / 4) as i8;
            }
            Pc::Cpx
        }
    }

    /// Works out the terrain under the actor, and carries on sliding if it is on ice.
    pub(super) unsafe fn cpx(&mut self) -> Pc {
        unsafe {
            let an = self.an;
            self.j = terrain((*an).abs_x, (*an).abs_y);
            if self.k == ENVIRON_ICE {
                (*an).abs_x = ((*an).abs_x as i32 + (*an).vel_x as i32 / 4) as u16;
                (*an).abs_y = ((*an).abs_y as i32 + (*an).vel_y as i32 / 4) as u16;
            }
            Pc::Tail
        }
    }

    /// After the movement: finishes a death, then turns the terrain under the actor into its `environ`
    /// (water depth, ice, the pit).
    pub(super) unsafe fn tail(&mut self) -> Pc {
        unsafe {
            let an = self.an;
            let i = self.i;
            if self.s == DYING {
                (*an).tactic = (*an).tactic.wrapping_sub(1);
                if (*an).tactic == 0 {
                    (*an).state = DEAD as i8;
                    if (*an).race == RACE_WITCH {
                        (*an).race = RACE_WITCH_BODY;
                        (*an).vitality = 10;
                        (*an).state = STILL;
                        (*an).weapon = 0;
                        leave_item(i as i16, TALISMAN);
                    }
                    if (*an).race == RACE_LASSO_CARRIER {
                        leave_item(i as i16, LASSO);
                    }
                }
            }
            let j = self.j;
            if i == 0 && the_actors.raft_nearness != 0 {
                self.k = 0; // can't drown on the raft
            } else if j == 0 {
                self.k = 0;
            } else if j == 6 {
                self.k = ENVIRON_SLOW;
            } else if j == 7 {
                self.k = ENVIRON_ICE;
            } else if j == 8 {
                self.k = ENVIRON_BACKWARDS;
            } else if j == 9 && i == 0 && the_world.extent_type == PIT_XTYPE {
                if (*an).state as i16 != FALL {
                    self.dex = fallstates[(the_hero.brother * 6) as usize] as i16;
                    (*an).state = FALL as i8;
                    (*an).tactic = 0;
                    the_hero.luck = the_hero.luck.wrapping_sub(PIT_LUCK_COST);
                    set_mood(1);
                }
                self.k = ENVIRON_ICE;
            } else if j == 2 {
                self.k = 2;
            } else if j == 3 {
                self.k = 5;
            } else if j == TERRAIN_SLOW_SINK || j == TERRAIN_DEEP_WATER {
                self.wade(j);
            }
            if self.k == 0 && self.s == SINK {
                (*an).state = STILL;
            }
            Pc::Raise
        }
    }

    /// Stores the environ, adjusts the sprite for some enemies, keeps the hero inside the world and
    /// scrolls the map to it, and applies lava and drowning damage.
    pub(super) unsafe fn raise(&mut self) -> Pc {
        unsafe {
            let an = self.an;
            let i = self.i;
            let cycle = self.cycle as i32;
            if the_world.new_region >= NO_REGION {
                (*an).environ = self.k;
            }
            let race = (*an).race as i8;
            if (*an).kind == ENEMY {
                self.enemy_frame(race);
            }
            (*an).index = self.dex as i8;
            if i == 0 {
                self.hero_wrap();
                the_hero.x = (*an).abs_x;
                the_hero.y = (*an).abs_y;
                move_camera_to(the_hero.x as i32, the_hero.y as i32);
                the_hero.safe_flag = self.j as i8;
            }
            if (*an).vitality == 0 {
                return Pc::Statc;
            }
            if the_actors.fiery_death != 0 {
                if i == 0 && *the_hero.stuff.add(STUFF_FIREPROOF) != 0 {
                    (*an).environ = 0;
                } else if (*an).environ > ENVIRON_SINKING {
                    (*an).vitality = 0;
                } else if (*an).environ > 2 {
                    (*an).vitality = (*an).vitality.wrapping_sub(1);
                }
                check_death(i, DEATH_LAVA);
            }
            if (*an).environ == ENVIRON_DROWNED && (cycle & 7) == 0 && race != 2 && race != 3 {
                (*an).vitality = (*an).vitality.wrapping_sub(1);
                check_death(i, DEATH_DROWN);
            }
            Pc::Statc
        }
    }

    /// The sprite frames of snakes (race 4), the spectre (8) and the race-7 enemy at zero vitality.
    unsafe fn enemy_frame(&mut self, race: i8) {
        unsafe {
            let an = self.an;
            let cycle = self.cycle as i32;
            let state = (*an).state as i16;
            if race == 4 && state < WALKING {
                self.dex = ((cycle & 1) + diroff(self.d) as i32) as i16;
            } else if race == 4 && state < DYING {
                self.dex = (((cycle / 2) & 1) + diroff(self.d) as i32) as i16;
            } else if race == 8 {
                if state == DEAD {
                    (*an).abs_x = 0;
                } else if state == DYING {
                    self.dex = 0x3f;
                } else {
                    self.dex = ((cycle & 3) * 2) as i16;
                    if self.dex > 4 {
                        self.dex -= 1;
                    }
                    match self.i % 3 {
                        0 => self.dex = 0x25,
                        1 => self.dex += 0x28,
                        _ => self.dex += 0x30,
                    }
                }
            } else if race == 7 && (*an).vitality == 0 {
                // The original has `an->state == STILL` here: a comparison that does nothing (an
                // assignment may have been meant). Kept as it was.
                self.dex = 1;
            }
        }
    }

    /// The hero leaving the edge of the world comes back on the other side (outdoor regions only);
    /// whatever the hero rides is moved along.
    unsafe fn hero_wrap(&mut self) {
        unsafe {
            let an = self.an;
            if the_world.region >= OUTDOOR_REGIONS {
                return;
            }
            if (*an).abs_x < WORLD_MIN {
                (*an).abs_x = WORLD_MAX;
            } else if (*an).abs_x > WORLD_MAX {
                (*an).abs_x = WORLD_MIN;
            } else if (*an).abs_y < WORLD_MIN {
                (*an).abs_y = WORLD_MAX;
            } else if (*an).abs_y > WORLD_MAX {
                (*an).abs_y = WORLD_MIN;
            } else {
                return;
            }
            if the_hero.riding > 1 {
                the_actors.list[CARRIER_SLOT].abs_x = (*an).abs_x;
                the_actors.list[CARRIER_SLOT].abs_y = (*an).abs_y;
            }
        }
    }

    /// Where the actor is on screen: relative to the map, with the sprite's anchor offset.
    pub(super) unsafe fn statc(&mut self) {
        unsafe {
            let an = self.an;
            let (ox, oy) = if (*an).kind == CARRIER && the_hero.riding == RIDING_SWAN {
                (32, 40)
            } else if (*an).kind == RAFT || (*an).kind == CARRIER || (*an).kind == DRAGON {
                (16, 16)
            } else {
                (8, 26)
            };
            (*an).rel_x = wrap_coordinate((*an).abs_x as i32 - the_world.map_x as i32 - ox) as u16;
            (*an).rel_y = wrap_coordinate((*an).abs_y as i32 - the_world.map_y as i32 - oy) as u16;
        }
    }
}
