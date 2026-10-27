//! The raft, the swan and the turtle: when they carry the hero, and where the turtle swims on its own.

use super::*;

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
    let right = (dir + 1) & 7;
    if water_ahead(right) {
        return right;
    }
    let left = (right - 2) & 7;
    if water_ahead(left) {
        return left;
    }
    (left - 1) & 7
}

impl Ctx {
    /// The swan (sprite file 11) or the turtle: carries the hero when the hero is close enough.
    pub(super) unsafe fn carrier(&mut self) -> Pc {
        unsafe {
            let an = self.an;
            self.k = 0;
            self.j = 0;
            let hero = &raw const the_actors.list[0];
            if the_actors.loaded_actor_file == SWAN_FILE {
                if swan_carries(
                    the_actors.raft_nearness,
                    the_actors.carrier_slot,
                    *the_hero.stuff.add(STUFF_SWAN_ITEM) != 0,
                ) {
                    self.d = (*hero).facing as i16;
                    (*an).abs_x = (*hero).abs_x;
                    XTEST = (*hero).abs_x;
                    (*an).abs_y = (*hero).abs_y;
                    YTEST = (*hero).abs_y;
                    the_hero.riding = RIDING_SWAN;
                } else {
                    XTEST = (*an).abs_x;
                    YTEST = (*an).abs_y;
                }
                self.dex = self.d;
                self.e = 0;
            } else {
                if turtle_carries(the_actors.raft_nearness, the_actors.carrier_slot) {
                    self.d = (*hero).facing as i16;
                    XTEST = (*hero).abs_x;
                    YTEST = (*hero).abs_y;
                    the_hero.riding = RIDING_TURTLE;
                    self.dex = self.d + self.d;
                    if (*hero).state as i16 == WALKING {
                        self.dex += self.cycle & 1;
                    }
                } else {
                    self.turtle_swims();
                }
                self.j = terrain(XTEST, YTEST);
                if self.j == TURTLE_WATER {
                    (*an).abs_x = XTEST;
                    (*an).abs_y = YTEST;
                }
                self.e = 1;
            }
            center_extent_on(self.e, XTEST as i16, YTEST as i16);
            Pc::Raise
        }
    }

    /// The turtle on its own: keeps going the same way while there is deep water ahead, otherwise
    /// tries turning right, then left, then back to straight on the left again.
    unsafe fn turtle_swims(&mut self) {
        unsafe {
            let an = self.an;
            let (ax, ay) = ((*an).abs_x as i32, (*an).abs_y as i32);
            self.d = turtle_heading(self.d, |dir| {
                terrain(pos_x(ax, dir, 3), pos_y(ay, dir, 3)) == TURTLE_WATER
            });
            XTEST = pos_x(ax, self.d, 3);
            YTEST = pos_y(ay, self.d, 3);
            the_hero.riding = 0;
            self.dex = self.d + self.d + (self.cycle & 1);
        }
    }

    /// The raft: follows the hero when the hero is very near and the terrain is water.
    pub(super) unsafe fn raft(&mut self) -> Pc {
        unsafe {
            let an = self.an;
            the_hero.riding = 0;
            if the_actors.carrier_slot != RIDING_RAFT || the_actors.raft_nearness != 2 {
                return Pc::Statc;
            }
            XTEST = the_actors.list[0].abs_x;
            YTEST = the_actors.list[0].abs_y;
            self.j = terrain(XTEST, YTEST);
            if self.j < 3 || self.j > 5 {
                return Pc::Statc;
            }
            (*an).abs_x = XTEST;
            (*an).abs_y = YTEST;
            the_hero.riding = RIDING_RAFT;
            Pc::Statc
        }
    }
}

#[cfg(test)]
mod tests {
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
