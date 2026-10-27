//! Walking, standing still and fighting: sliding on ice, going around obstacles, being blocked.

use super::water::climbs_out_of_water;
use super::{
    BLOCK_DOOR, BLOCK_PASSABLE_WITH_KEY, Ctx, ENVIRON_BACKWARDS, ENVIRON_ICE, ENVIRON_SHALLOW,
    ENVIRON_SLOW, FACING_MASK, FRAME_FRUSTRATED, FRAME_OSCIL, FRUST, PIT_SECTOR, Pc, RACE_SNAKE,
    RACE_WRAITH, RIDING_SWAN, RIDING_TURTLE, STUFF_GOLD_KEY_SLOT, XTEST, YTEST, diroff, pos_x,
    pos_y, terrain,
};

use crate::game::actors::interaction::blocked_at;
use crate::game::actors::steering::{STEER_OFFSET, steer_actor};
use crate::game::data::random::rand4;
use crate::game::data::state::{the_actors, the_hero};
use crate::game::data::tables::swing_transitions;
use crate::game::world::key_doors::{bumped_locked_door, open_door_near};
use crate::game::world::map::{step_x, step_y};

/// Blocked for this many frames the hero oscillates; for this many more, he looks frustrated.
const FRUSTRATION_OSCILLATE: i8 = 20;
const FRUSTRATION_FRUSTRATED: i8 = 40;

/// Step lengths in pixels per frame.
const STEP_TURTLE: i16 = 3;
const STEP_BACKWARDS: i16 = -2;
const STEP_SLOW_GROUND: i16 = 4;
const STEP_SHALLOW: i16 = 1;
const STEP_NORMAL: i16 = 2;

/// Water deeper than this (or exactly `ENVIRON_SHALLOW`) slows the walk.
const ENVIRON_DEEP: i8 = 6;

/// Sliding on ice: the top speed (while riding the swan, and not), and the offset used to read the
/// direction's step from the step tables.
const ICE_TOP_SPEED_SWAN: i16 = 40;
const ICE_TOP_SPEED: i16 = 42;
const ICE_STEP_ORIGIN: i32 = 20;
const ICE_STEP_SPEED: i32 = 2;
const ICE_X_MARGIN: i16 = 8;

/// A walking frame cycles through 8 (flipping the low 3 bits plays them backwards).
const WALK_FRAMES_MASK: i32 = 7;

/// Snake pictures start here, two frames for each pair of facings.
const SNAKE_FIRST_FRAME: i16 = 0x24;

/// Velocity is kept as four times the movement of the last step.
const VELOCITY_SCALE: i32 = 4;

/// The fighting states 6 and 7 are shown as state 8 (the follow-through).
const SWING_STATE_FOLLOW_THROUGH: i16 = 8;
const SWING_STATE_FIRST: i16 = 6;
const SWING_STATE_SECOND: i16 = 7;
const FIRST_NPC_SLOT: i32 = 2;

/// The hero's frame once the way has been blocked for a while: oscillating, then a frustrated one.
fn frustrated_frame(frustration: i8, cycle: i16) -> Option<i16> {
    if frustration > FRUSTRATION_FRUSTRATED {
        Some(FRAME_FRUSTRATED)
    } else if frustration > FRUSTRATION_OSCILLATE {
        // Swap between the two impatient frames every second tick.
        Some(FRAME_OSCIL + ((cycle >> 1) & 1))
    } else {
        None
    }
}

/// How far an actor walks in a frame: the hero riding the turtle goes fastest, walking backwards or
/// through slow ground is slower, and a little water (environ 2 or deeper than 6) slows to 1.
fn step_length(hero_on_turtle: bool, environ: i8) -> i16 {
    if hero_on_turtle {
        STEP_TURTLE
    } else if environ == ENVIRON_BACKWARDS {
        STEP_BACKWARDS
    } else if environ == ENVIRON_SLOW {
        STEP_SLOW_GROUND
    } else if environ == ENVIRON_SHALLOW || environ > ENVIRON_DEEP {
        STEP_SHALLOW
    } else {
        STEP_NORMAL
    }
}

impl Ctx {
    /// Walking: sliding on ice, picking a step, and trying to go around what is in the way.
    pub(super) unsafe fn walking(&mut self) -> Pc {
        unsafe {
            let actor = self.actor;

            if self.environ == ENVIRON_ICE {
                if let Some(next) = self.slide_on_ice() {
                    return next;
                }

                self.environ = 0;
            }

            self.step = step_length(
                self.slot == 0 && the_hero.riding == RIDING_TURTLE,
                self.environ,
            );

            XTEST = pos_x((*actor).abs_x as i32, self.facing, self.step);
            YTEST = pos_y((*actor).abs_y as i32, self.facing, self.step);
            self.terrain = blocked_at(XTEST as i16, YTEST as i16, self.slot as i16) as i16;

            if self.slot == 0 && self.hero_bumps_into_door() {
                return Pc::NewLoc;
            }

            if self.terrain != 0 {
                Pc::CheckDev1
            } else {
                Pc::NewLoc
            }
        }
    }

    /// The hero walking into a door opens it (key doors have their own keys); walking away from one
    /// forgets that it was locked. Returns true when the gold key lets him through.
    unsafe fn hero_bumps_into_door(&mut self) -> bool {
        unsafe {
            if self.terrain == BLOCK_DOOR {
                open_door_near(XTEST, YTEST, 0);
            } else {
                bumped_locked_door = 0;
            }

            *the_hero.stuff.add(STUFF_GOLD_KEY_SLOT) != 0 && self.terrain == BLOCK_PASSABLE_WITH_KEY
        }
    }

    /// On ice the actor keeps its momentum: pushes the velocity toward the wanted direction up to a
    /// limit and moves by it. Returns where to go next, or `None` when it hit something and should
    /// walk normally this tick.
    unsafe fn slide_on_ice(&mut self) -> Option<Pc> {
        unsafe {
            let actor = self.actor;
            self.step = if the_hero.riding == RIDING_SWAN {
                ICE_TOP_SPEED_SWAN
            } else {
                ICE_TOP_SPEED
            };

            let nvx = ((*actor).vel_x as i32
                + step_x(ICE_STEP_ORIGIN, self.facing as i32, ICE_STEP_SPEED)
                - ICE_STEP_ORIGIN) as i16;
            let nvy = ((*actor).vel_y as i32
                + step_y(ICE_STEP_ORIGIN, self.facing as i32, ICE_STEP_SPEED)
                - ICE_STEP_ORIGIN) as i16;

            if nvx.abs() < self.step - ICE_X_MARGIN {
                (*actor).vel_x = nvx as i8;
            }

            if nvy.abs() < self.step {
                (*actor).vel_y = nvy as i8;
            }

            XTEST = ((*actor).abs_x as i32 + (*actor).vel_x as i32 / VELOCITY_SCALE) as u16;
            YTEST = ((*actor).abs_y as i32 + (*actor).vel_y as i32 / VELOCITY_SCALE) as u16;

            if the_hero.riding == RIDING_SWAN {
                steer_actor(0, -(nvx as i32), -(nvy as i32), STEER_OFFSET);
                self.facing = (*actor).facing as i16;

                return Some(Pc::NewLoc);
            }

            if blocked_at(XTEST as i16, YTEST as i16, self.slot as i16) == 0 {
                return Some(Pc::NewLoc);
            }

            None
        }
    }

    /// Blocked: try one direction to the right.
    pub(super) unsafe fn check_dev1(&mut self) -> Pc {
        unsafe {
            let actor = self.actor;

            // Masks with FACING_MASK, keeping only the bits that matter here.
            self.facing = (self.facing + 1) & FACING_MASK;
            XTEST = pos_x((*actor).abs_x as i32, self.facing, self.step);
            YTEST = pos_y((*actor).abs_y as i32, self.facing, self.step);

            if blocked_at(XTEST as i16, YTEST as i16, self.slot as i16) != 0 {
                Pc::CheckDev2
            } else {
                Pc::NewLoc
            }
        }
    }

    /// Still blocked: try one direction to the left of the original.
    pub(super) unsafe fn check_dev2(&mut self) -> Pc {
        unsafe {
            let actor = self.actor;

            // Masks with FACING_MASK, keeping only the bits that matter here.
            self.facing = (self.facing - 2) & FACING_MASK;
            XTEST = pos_x((*actor).abs_x as i32, self.facing, self.step);
            YTEST = pos_y((*actor).abs_y as i32, self.facing, self.step);

            if blocked_at(XTEST as i16, YTEST as i16, self.slot as i16) != 0 {
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
            self.pick_walking_frame();

            if self.environ == ENVIRON_ICE {
                return Pc::Cpx;
            }

            if self.environ == ENVIRON_BACKWARDS {
                // Walking backwards: flipping the three low bits plays the walk frames in reverse order.
                self.image ^= WALK_FRAMES_MASK as i16;
            }

            self.look_at_new_terrain();

            if climbs_out_of_water(self.environ, self.terrain) {
                if the_hero.sector != PIT_SECTOR {
                    self.environ -= 1;
                }

                return Pc::Raise;
            }

            self.take_the_step();

            Pc::Tail
        }
    }

    /// Faces the actor the way it will step, and picks the walking frame for this tick (a rider and
    /// the wraiths keep their one frame).
    unsafe fn pick_walking_frame(&mut self) {
        unsafe {
            let actor = self.actor;
            (*actor).facing = self.facing as i8;
            self.frame_base = diroff(self.facing);
            self.image = self.frame_base;

            if !(the_hero.riding != 0 && self.slot == 0) && (*actor).race != RACE_WRAITH {
                self.image = (self.image as i32
                    // Masks with WALK_FRAMES_MASK, keeping only the bits that matter here.
                    + ((self.cycle as i32 + self.slot) & WALK_FRAMES_MASK))
                    as i16;
            }
        }
    }

    /// The terrain at the new place; wraiths and snakes are immune to water, and snakes have their
    /// own pictures.
    unsafe fn look_at_new_terrain(&mut self) {
        unsafe {
            let actor = self.actor;
            self.terrain = terrain(XTEST, YTEST);

            if (*actor).race == RACE_WRAITH || (*actor).race == RACE_SNAKE {
                self.terrain = 0;
            }

            if (*actor).race == RACE_SNAKE {
                // Snake pictures: two frames (bit 1 of the cycle) for each pair of facings, starting at image 0x24.
                self.image = (self.cycle & 2) + ((self.facing & 0xfe) << 2) + SNAKE_FIRST_FRAME;
            }
        }
    }

    /// Moves to the new place, remembering the step as the velocity (for sliding on ice).
    unsafe fn take_the_step(&mut self) {
        unsafe {
            let actor = self.actor;
            (*actor).vel_x =
                ((XTEST.wrapping_sub((*actor).abs_x) as i16 as i32) * VELOCITY_SCALE) as i8;
            (*actor).vel_y =
                ((YTEST.wrapping_sub((*actor).abs_y) as i16 as i32) * VELOCITY_SCALE) as i8;
            (*actor).abs_x = XTEST;
            (*actor).abs_y = YTEST;
            the_actors.frustration = 0;
        }
    }

    /// Walking found no way through. The hero shows a frustrated frame after a while; others change
    /// tactic. (The C placed this label inside the `STILL` branch.)
    pub(super) unsafe fn blocked(&mut self) -> Pc {
        unsafe {
            if self.slot == 0 {
                the_actors.frustration = the_actors.frustration.wrapping_add(1);

                if let Some(frame) = frustrated_frame(the_actors.frustration, self.cycle) {
                    self.image = frame;
                }

                return Pc::Cpx;
            }

            (*self.actor).tactic = FRUST;

            Pc::Still
        }
    }

    /// Standing still: the frame after the first of the facing's walking frames.
    pub(super) unsafe fn still(&mut self) -> Pc {
        unsafe {
            self.image = self.frame_base + 1;

            if self.environ == ENVIRON_ICE {
                return Pc::Cpx;
            }

            self.terrain = terrain((*self.actor).abs_x, (*self.actor).abs_y);

            Pc::Tail
        }
    }

    /// Swinging a weapon (states 0-8): steps through the swing table.
    pub(super) unsafe fn fighting(&mut self) -> Pc {
        unsafe {
            let actor = self.actor;
            self.frame_base = diroff(self.facing + 8);
            let next = swing_transitions[self.state as usize][rand4() as usize] as i16;
            (*actor).state = next as i8;
            self.state = next;

            if (self.slot > FIRST_NPC_SLOT && self.state == SWING_STATE_FIRST)
                || self.state == SWING_STATE_SECOND
            {
                self.state = SWING_STATE_FOLLOW_THROUGH;
            }

            self.image = self.state + self.frame_base;
            the_actors.frustration = 0;

            Pc::Cpx
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::FRAME_FRUSTRATED;
    use super::super::FRAME_OSCIL;
    use super::*;

    #[test]
    fn being_stuck_shows_impatience() {
        assert_eq!(frustrated_frame(20, 0), None);
        assert_eq!(frustrated_frame(21, 0), Some(FRAME_OSCIL));
        assert_eq!(frustrated_frame(21, 2), Some(FRAME_OSCIL + 1));
        assert_eq!(frustrated_frame(41, 0), Some(FRAME_FRUSTRATED));
    }

    #[test]
    fn walking_speeds() {
        assert_eq!(step_length(true, 0), 3);
        assert_eq!(step_length(false, ENVIRON_BACKWARDS), -2);
        assert_eq!(step_length(false, ENVIRON_SLOW), 4);
        assert_eq!(step_length(false, 2), 1);
        assert_eq!(step_length(false, 7), 1);
        assert_eq!(step_length(false, 0), 2);
        assert_eq!(step_length(false, 6), 2);
    }
}
