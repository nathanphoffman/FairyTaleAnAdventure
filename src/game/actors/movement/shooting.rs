//! Shooting arrows and fireballs: starting a missile when an actor lets go.

use super::{
    Ctx, EFFECT_FIREBALL, EFFECT_FIREBALL_PITCH, ENEMY, ENVIRON_SINKING, MISSILE_SPEED_ARROW,
    MISSILE_SPEED_FIREBALL, NO_SHOOT_XTYPE, Pc, SHOOT1, SHOOT3, STILL, STUFF_ARROWS,
    WEAPON_FIREBALL, diroff,
};
use crate::game::data::state::Game;

use crate::game::data::random::rand256;
use crate::game::data::state::{the_actors, the_world};
use crate::game::files::loading::play_effect;
use crate::game::ui::messages::print;

/// The facing frames for shooting come after the 8 walking ones (`facing + 8`); the bow-drawn and
/// released frames sit 10 or 11 on from the shooting frame (11 for enemies).
const SHOOTING_FACING_OFFSET: i16 = 8;
const BOW_RELEASED_FRAME: i16 = 11;
const HERO_BOW_DRAWN_FRAME: i16 = 10;
const ENEMY_BOW_DRAWN_FRAME: i16 = 11;

/// A weapon number of 4 or more is a missile weapon; the missile type is the weapon minus 3.
const MISSILE_WEAPON_BASE: i8 = 3;

/// Sound effects for the bow and the fireball, and their base pitch.
const EFFECT_ARROW: i16 = 4;
const EFFECT_ARROW_PITCH: i32 = 400;

impl Ctx {
    /// Aiming and firing a bow or a fireball. No shooting under water or in peaceful places.
    pub(super) unsafe fn shooting(&mut self, game: &mut Game) -> Pc {
        unsafe {
            if self.environ > ENVIRON_SINKING || the_world.extent_type > NO_SHOOT_XTYPE {
                return Pc::Cpx;
            }

            let early_exit = if self.state == SHOOT3 {
                self.release_shot(game)
            } else if self.state == SHOOT1 {
                self.start_shot(game)
            } else {
                None
            };

            if let Some(pc) = early_exit {
                return pc;
            }

            self.missile.missile_type = self.actor.weapon - MISSILE_WEAPON_BASE;

            Pc::DragShoot
        }
    }

    /// The shot is let go: a fireball just stays drawn, an arrow leaves the bow (and one is used up
    /// for the hero). `Some` means nothing is fired.
    unsafe fn release_shot(&mut self, game: &mut Game) -> Option<Pc> {
        unsafe {
            let mut actor = self.actor;
            let d8 = self.facing + SHOOTING_FACING_OFFSET;

            if actor.weapon == WEAPON_FIREBALL {
                self.image = diroff(d8);

                return Some(Pc::Cpx);
            }

            self.image = diroff(d8) + BOW_RELEASED_FRAME;
            actor.state = STILL;

            if self.slot == 0 {
                if game.hero.stuff()[STUFF_ARROWS] == 0 {
                    return Some(Pc::Cpx);
                }

                game.hero.stuff_mut()[STUFF_ARROWS] -= 1;
            }

            self.missile.speed = MISSILE_SPEED_ARROW;
            the_actors.next_missile = the_actors.next_missile.wrapping_add(1);
            play_effect(EFFECT_ARROW, EFFECT_ARROW_PITCH + rand256());

            None
        }
    }

    /// The actor starts to shoot: a fireball is fired at once, a bow needs arrows (for the hero).
    /// `Some` means nothing is fired.
    unsafe fn start_shot(&mut self, game: &mut Game) -> Option<Pc> {
        unsafe {
            let mut actor = self.actor;
            let d8 = self.facing + SHOOTING_FACING_OFFSET;

            self.image = diroff(d8)
                + if actor.kind == ENEMY {
                    ENEMY_BOW_DRAWN_FRAME
                } else {
                    HERO_BOW_DRAWN_FRAME
                };

            if actor.weapon == WEAPON_FIREBALL {
                self.missile.speed = MISSILE_SPEED_FIREBALL;
                the_actors.next_missile = the_actors.next_missile.wrapping_add(1);
                actor.state = SHOOT3 as i8;
                self.image = diroff(d8);
                play_effect(EFFECT_FIREBALL, EFFECT_FIREBALL_PITCH + rand256());
            } else if self.slot == 0 && game.hero.stuff()[STUFF_ARROWS] == 0 {
                print(game, "No Arrows!");

                return Some(Pc::Cpx);
            } else {
                self.missile.speed = 0;
            }

            None
        }
    }
}
