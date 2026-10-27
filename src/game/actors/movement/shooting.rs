//! Shooting arrows and fireballs: starting a missile when an actor lets go.

use super::{
    Ctx, ENEMY, ENVIRON_SINKING, MISSILE_SPEED_ARROW, MISSILE_SPEED_FIREBALL, NO_SHOOT_XTYPE, Pc,
    SHOOT1, SHOOT3, STILL, STUFF_ARROWS, WEAPON_FIREBALL, diroff,
};

use crate::game::data::random::rand256;
use crate::game::data::state::{the_actors, the_hero, the_world};
use crate::game::files::loading::play_effect;
use crate::game::ui::messages::print;

impl Ctx {
    /// Aiming and firing a bow or a fireball. No shooting under water or in peaceful places.
    pub(super) unsafe fn shooting(&mut self) -> Pc {
        unsafe {
            if self.environ > ENVIRON_SINKING || the_world.extent_type > NO_SHOOT_XTYPE {
                return Pc::Cpx;
            }

            let early_exit = if self.state == SHOOT3 {
                self.release_shot()
            } else if self.state == SHOOT1 {
                self.draw_bow_or_fireball()
            } else {
                None
            };

            if let Some(pc) = early_exit {
                return pc;
            }

            (*self.missile).missile_type = (*self.actor).weapon - 3;

            Pc::DragShoot
        }
    }

    /// The shot is let go: a fireball just stays drawn, an arrow leaves the bow (and one is used up
    /// for the hero). `Some` means nothing is fired.
    unsafe fn release_shot(&mut self) -> Option<Pc> {
        unsafe {
            let actor = self.actor;
            let d8 = self.facing + 8;

            if (*actor).weapon == WEAPON_FIREBALL {
                self.image = diroff(d8);

                return Some(Pc::Cpx);
            }

            self.image = diroff(d8) + 11;
            (*actor).state = STILL;

            if self.slot == 0 {
                if *the_hero.stuff.add(STUFF_ARROWS) == 0 {
                    return Some(Pc::Cpx);
                }

                *the_hero.stuff.add(STUFF_ARROWS) -= 1;
            }

            (*self.missile).speed = MISSILE_SPEED_ARROW;
            the_actors.next_missile = the_actors.next_missile.wrapping_add(1);
            play_effect(4, 400 + rand256());

            None
        }
    }

    /// The actor starts to shoot: a fireball is fired at once, a bow needs arrows (for the hero).
    /// `Some` means nothing is fired.
    unsafe fn draw_bow_or_fireball(&mut self) -> Option<Pc> {
        unsafe {
            let actor = self.actor;
            let d8 = self.facing + 8;

            self.image = diroff(d8) + if (*actor).kind == ENEMY { 11 } else { 10 };

            if (*actor).weapon == WEAPON_FIREBALL {
                (*self.missile).speed = MISSILE_SPEED_FIREBALL;
                the_actors.next_missile = the_actors.next_missile.wrapping_add(1);
                (*actor).state = SHOOT3 as i8;
                self.image = diroff(d8);
                play_effect(5, 1800 + rand256());
            } else if self.slot == 0 && *the_hero.stuff.add(STUFF_ARROWS) == 0 {
                print(c"No Arrows!".as_ptr());

                return Some(Pc::Cpx);
            } else {
                (*self.missile).speed = 0;
            }

            None
        }
    }
}
