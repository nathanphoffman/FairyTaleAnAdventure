//! The raft, the swan and the turtle: when they carry the hero, and where the turtle swims on its own.

use super::{
    CARRIER_SLOT, Ctx, FACING_MASK, Pc, RIDING_RAFT, RIDING_SWAN, RIDING_TURTLE, STUFF_SWAN_ITEM,
    SWAN_FILE, TERRAIN_DEEP_WATER, TERRAIN_WADE, TURTLE_STEP, TURTLE_WATER, WALKING, XTEST, YTEST,
    pos_x, pos_y, terrain,
};
use crate::game::data::state::ActorRef;
use crate::game::data::state::player;

use crate::game::actors::interaction::center_extent_on;
use crate::game::data::state::{the_actors, the_hero};

/// The swan carries the hero once he is near, with the swan on its slot and the lasso in the pack.
fn swan_carries(nearness: i16, carrier_slot: i16, has_lasso: bool) -> bool {
    nearness != 0 && carrier_slot == CARRIER_SLOT as i16 && has_lasso
}

/// The turtle carries the hero whenever he is near and on its slot.
fn turtle_carries(nearness: i16, carrier_slot: i16) -> bool {
    nearness != 0 && carrier_slot == CARRIER_SLOT as i16
}

/// The turtle's direction of swimming: straight on while there is deep water ahead, otherwise one
/// step right, then two to the left of that, then one more to the left; the last is used whether
/// or not it is water. `water_ahead(dir)` says if the turtle could swim that way.
fn turtle_heading(dir: i16, water_ahead: impl Fn(i16) -> bool) -> i16 {
    if water_ahead(dir) {
        return dir;
    }

    // Masks with FACING_MASK, keeping only the bits that matter here.
    let right = (dir + 1) & FACING_MASK;

    if water_ahead(right) {
        return right;
    }

    // Masks with FACING_MASK, keeping only the bits that matter here.
    let left = (right - 2) & FACING_MASK;

    if water_ahead(left) {
        return left;
    }

    // Masks with FACING_MASK, keeping only the bits that matter here.
    (left - 1) & FACING_MASK
}

impl Ctx {
    /// The swan (sprite file 11) or the turtle: carries the hero when the hero is close enough.
    pub(super) unsafe fn carrier(&mut self) -> Pc {
        unsafe {
            self.environ = 0;
            self.terrain = 0;

            if the_actors.loaded_actor_file == SWAN_FILE {
                self.swan_moves();
            } else {
                self.turtle_moves();
            }

            center_extent_on(self.step, XTEST as i16, YTEST as i16);

            Pc::Raise
        }
    }

    /// The swan goes where the hero goes if he is near and has the lasso, otherwise it stays.
    unsafe fn swan_moves(&mut self) {
        unsafe {
            let mut actor = self.actor;
            let hero = ActorRef::new(0);

            if swan_carries(
                the_actors.raft_nearness,
                the_actors.carrier_slot,
                player().stuff()[STUFF_SWAN_ITEM] != 0,
            ) {
                self.facing = hero.facing as i16;
                actor.abs_x = hero.abs_x;
                XTEST = hero.abs_x;
                actor.abs_y = hero.abs_y;
                YTEST = hero.abs_y;
                the_hero.riding = RIDING_SWAN;
            } else {
                XTEST = actor.abs_x;
                YTEST = actor.abs_y;
            }

            self.image = self.facing;
            self.step = 0;
        }
    }

    /// The turtle carries the hero when he is near, otherwise it swims on its own.
    unsafe fn turtle_moves(&mut self) {
        unsafe {
            let mut actor = self.actor;
            let hero = ActorRef::new(0);

            if turtle_carries(the_actors.raft_nearness, the_actors.carrier_slot) {
                self.facing = hero.facing as i16;
                XTEST = hero.abs_x;
                YTEST = hero.abs_y;
                the_hero.riding = RIDING_TURTLE;
                self.image = self.facing + self.facing;

                if hero.state as i16 == WALKING {
                    // Masks off the low bits (a cheap remainder or a flag test).
                    self.image += self.cycle & 1;
                }
            } else {
                self.turtle_swims();
            }

            self.terrain = terrain(XTEST, YTEST);

            if self.terrain == TURTLE_WATER {
                actor.abs_x = XTEST;
                actor.abs_y = YTEST;
            }

            self.step = 1;
        }
    }

    /// The turtle on its own: keeps going the same way while there is deep water ahead, otherwise
    /// tries turning right, then left, then back to straight on the left again.
    unsafe fn turtle_swims(&mut self) {
        unsafe {
            let actor = self.actor;
            let (ax, ay) = (actor.abs_x as i32, actor.abs_y as i32);
            self.facing = turtle_heading(self.facing, |dir| {
                terrain(pos_x(ax, dir, TURTLE_STEP), pos_y(ay, dir, TURTLE_STEP)) == TURTLE_WATER
            });

            XTEST = pos_x(ax, self.facing, TURTLE_STEP);
            YTEST = pos_y(ay, self.facing, TURTLE_STEP);
            the_hero.riding = 0;

            // Masks off the low bits (a cheap remainder or a flag test).
            self.image = self.facing + self.facing + (self.cycle & 1);
        }
    }

    /// The raft: follows the hero when the hero is very near and the terrain is water.
    pub(super) unsafe fn raft(&mut self) -> Pc {
        unsafe {
            let mut actor = self.actor;
            the_hero.riding = 0;

            if the_actors.carrier_slot != RIDING_RAFT || the_actors.raft_nearness != 2 {
                return Pc::Statc;
            }

            XTEST = the_actors.list[0].abs_x;
            YTEST = the_actors.list[0].abs_y;
            self.terrain = terrain(XTEST, YTEST);

            if !(TERRAIN_WADE..=TERRAIN_DEEP_WATER).contains(&self.terrain) {
                return Pc::Statc;
            }

            actor.abs_x = XTEST;
            actor.abs_y = YTEST;
            the_hero.riding = RIDING_RAFT;

            Pc::Statc
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::CARRIER_SLOT;
    use super::*;

    #[test]
    fn the_swan_needs_the_lasso_and_the_turtle_does_not() {
        assert!(swan_carries(1, CARRIER_SLOT as i16, true));
        assert!(!swan_carries(1, CARRIER_SLOT as i16, false));
        assert!(!swan_carries(0, CARRIER_SLOT as i16, true));
        assert!(!swan_carries(1, CARRIER_SLOT as i16 + 1, true));
        assert!(turtle_carries(1, CARRIER_SLOT as i16));
        assert!(!turtle_carries(0, CARRIER_SLOT as i16));
        assert!(!turtle_carries(1, 0));
    }

    #[test]
    fn the_turtle_keeps_to_the_water() {
        // straight on, then right, then two left of that, then one more left
        assert_eq!(turtle_heading(2, |_| true), 2);
        assert_eq!(turtle_heading(2, |d| d == 3), 3);
        assert_eq!(turtle_heading(2, |d| d == 1), 1);
        assert_eq!(turtle_heading(2, |_| false), 0);

        // the compass wraps
        assert_eq!(turtle_heading(7, |d| d == 0), 0);
        assert_eq!(turtle_heading(0, |d| d == 7), 7);
    }
}
