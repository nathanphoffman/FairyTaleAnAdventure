//! Walking, standing still and fighting: sliding on ice, going around obstacles, being blocked.

use super::*;

/// The hero's frame once the way has been blocked for a while: oscillating, then a frustrated one.
fn frustrated_frame(frustration: i8, cycle: i16) -> Option<i16> {
    if frustration > 40 {
        Some(FRAME_FRUSTRATED)
    } else if frustration > 20 {
        Some(FRAME_OSCIL + ((cycle >> 1) & 1))
    } else {
        None
    }
}

impl Ctx {
    /// Walking: sliding on ice, picking a step, and trying to go around what is in the way.
    pub(super) unsafe fn walking(&mut self) -> Pc {
        unsafe {
            let an = self.an;
            if self.k == ENVIRON_ICE {
                if let Some(next) = self.slide_on_ice() {
                    return next;
                }
                self.k = 0;
            }
            let k = self.k;
            self.e = if self.i == 0 && the_hero.riding == RIDING_TURTLE {
                3
            } else if k == ENVIRON_BACKWARDS {
                -2
            } else if k == ENVIRON_SLOW {
                4
            } else if k == 2 || k > 6 {
                1
            } else {
                2
            };
            XTEST = pos_x((*an).abs_x as i32, self.d, self.e);
            YTEST = pos_y((*an).abs_y as i32, self.d, self.e);
            self.j = blocked_at(XTEST as i16, YTEST as i16, self.i as i16) as i16;
            if self.i == 0 {
                if self.j == BLOCK_DOOR {
                    open_door_near(XTEST, YTEST, 0);
                } else {
                    bumped_locked_door = 0;
                }
                if *the_hero.stuff.add(STUFF_GOLD_KEY_SLOT) != 0
                    && self.j == BLOCK_PASSABLE_WITH_KEY
                {
                    return Pc::NewLoc;
                }
            }
            if self.j != 0 {
                Pc::CheckDev1
            } else {
                Pc::NewLoc
            }
        }
    }

    /// On ice the actor keeps its momentum: pushes the velocity toward the wanted direction up to a
    /// limit and moves by it. Returns where to go next, or `None` when it hit something and should
    /// walk normally this tick.
    unsafe fn slide_on_ice(&mut self) -> Option<Pc> {
        unsafe {
            let an = self.an;
            self.e = if the_hero.riding == RIDING_SWAN {
                40
            } else {
                42
            };
            let nvx = ((*an).vel_x as i32 + step_x(20, self.d as i32, 2) - 20) as i16;
            let nvy = ((*an).vel_y as i32 + step_y(20, self.d as i32, 2) - 20) as i16;
            if nvx.abs() < self.e - 8 {
                (*an).vel_x = nvx as i8;
            }
            if nvy.abs() < self.e {
                (*an).vel_y = nvy as i8;
            }
            XTEST = ((*an).abs_x as i32 + (*an).vel_x as i32 / 4) as u16;
            YTEST = ((*an).abs_y as i32 + (*an).vel_y as i32 / 4) as u16;
            if the_hero.riding == RIDING_SWAN {
                steer_actor(0, -(nvx as i32), -(nvy as i32), 6);
                self.d = (*an).facing as i16;
                return Some(Pc::NewLoc);
            }
            if blocked_at(XTEST as i16, YTEST as i16, self.i as i16) == 0 {
                return Some(Pc::NewLoc);
            }
            None
        }
    }

    /// Blocked: try one direction to the right.
    pub(super) unsafe fn check_dev1(&mut self) -> Pc {
        unsafe {
            let an = self.an;
            self.d = (self.d + 1) & 7;
            XTEST = pos_x((*an).abs_x as i32, self.d, self.e);
            YTEST = pos_y((*an).abs_y as i32, self.d, self.e);
            if blocked_at(XTEST as i16, YTEST as i16, self.i as i16) != 0 {
                Pc::CheckDev2
            } else {
                Pc::NewLoc
            }
        }
    }

    /// Still blocked: try one direction to the left of the original.
    pub(super) unsafe fn check_dev2(&mut self) -> Pc {
        unsafe {
            let an = self.an;
            self.d = (self.d - 2) & 7;
            XTEST = pos_x((*an).abs_x as i32, self.d, self.e);
            YTEST = pos_y((*an).abs_y as i32, self.d, self.e);
            if blocked_at(XTEST as i16, YTEST as i16, self.i as i16) != 0 {
                Pc::Blocked
            } else {
                Pc::NewLoc
            }
        }
    }

    /// A free step was found: face that way, pick the walking frame, and move (unless the terrain
    /// slows the actor to a halt).
    pub(super) unsafe fn new_loc(&mut self) -> Pc {
        unsafe {
            let an = self.an;
            (*an).facing = self.d as i8;
            self.inum = diroff(self.d);
            self.dex = self.inum;
            if !(the_hero.riding != 0 && self.i == 0) && (*an).race != RACE_WRAITH {
                self.dex = (self.dex as i32 + ((self.cycle as i32 + self.i) & 7)) as i16;
            }
            if self.k == ENVIRON_ICE {
                return Pc::Cpx;
            }
            if self.k == ENVIRON_BACKWARDS {
                self.dex ^= 7;
            }
            self.j = terrain(XTEST, YTEST);
            // wraiths and snakes are immune to water
            if (*an).race == RACE_WRAITH || (*an).race == RACE_SNAKE {
                self.j = 0;
            }
            if (*an).race == RACE_SNAKE {
                self.dex = (self.cycle & 2) + ((self.d & 0xfe) << 2) + 0x24;
            }
            if climbs_out_of_water(self.k, self.j) {
                if the_hero.sector != PIT_SECTOR {
                    self.k -= 1;
                }
                return Pc::Raise;
            }
            (*an).vel_x = ((XTEST.wrapping_sub((*an).abs_x) as i16 as i32) * 4) as i8;
            (*an).vel_y = ((YTEST.wrapping_sub((*an).abs_y) as i16 as i32) * 4) as i8;
            (*an).abs_x = XTEST;
            (*an).abs_y = YTEST;
            the_actors.frustration = 0;
            Pc::Tail
        }
    }

    /// Walking found no way through. The hero shows a frustrated frame after a while; others change
    /// tactic. (The C placed this label inside the `STILL` branch.)
    pub(super) unsafe fn blocked(&mut self) -> Pc {
        unsafe {
            if self.i == 0 {
                the_actors.frustration = the_actors.frustration.wrapping_add(1);
                if let Some(frame) = frustrated_frame(the_actors.frustration, self.cycle) {
                    self.dex = frame;
                }
                return Pc::Cpx;
            }
            (*self.an).tactic = FRUST;
            Pc::Still
        }
    }

    /// Standing still: the frame after the first of the facing's walking frames.
    pub(super) unsafe fn still(&mut self) -> Pc {
        unsafe {
            self.dex = self.inum + 1;
            if self.k == ENVIRON_ICE {
                return Pc::Cpx;
            }
            self.j = terrain((*self.an).abs_x, (*self.an).abs_y);
            Pc::Tail
        }
    }

    /// Swinging a weapon (states 0-8): steps through the swing table.
    pub(super) unsafe fn fighting(&mut self) -> Pc {
        unsafe {
            let an = self.an;
            self.inum = diroff(self.d + 8);
            let next = swing_transitions[self.s as usize][rand4() as usize] as i16;
            (*an).state = next as i8;
            self.s = next;
            if (self.i > 2 && self.s == 6) || self.s == 7 {
                self.s = 8;
            }
            self.dex = self.s + self.inum;
            the_actors.frustration = 0;
            Pc::Cpx
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn being_stuck_shows_impatience() {
        assert_eq!(frustrated_frame(20, 0), None);
        assert_eq!(frustrated_frame(21, 0), Some(FRAME_OSCIL));
        assert_eq!(frustrated_frame(21, 2), Some(FRAME_OSCIL + 1));
        assert_eq!(frustrated_frame(41, 0), Some(FRAME_FRUSTRATED));
    }
}
