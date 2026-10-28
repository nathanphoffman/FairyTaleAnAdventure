//! The dragon: how it moves and breathes fire.

use super::{
    Ctx, DEAD, DRAGON_BREATH_FACING, DRAGON_DEAD_FRAME, DRAGON_DYING_FRAME, DYING, EFFECT_FIREBALL,
    EFFECT_FIREBALL_PITCH, MAX_MISSILE, MISSILE_ROCK_TYPE, MISSILE_SPEED_FIREBALL, Pc, WEAPON_BOW,
};
use crate::game::data::state::Game;

use crate::game::data::object_tables::{bowshotx, bowshoty, gunshoty};
use crate::game::data::random::{rand2, rand4, rand256};
use crate::game::files::loading::play_effect;

impl Ctx {
    /// The dragon: it always shows image 0 (the C sets the dying and dead images and then overwrites
    /// them), and now and then breathes a fireball.
    pub(super) fn dragon(&mut self, game: &mut Game) -> Pc {
        let actor = self.actor;
        self.image = 0;

        if self.state == DYING {
            self.image = DRAGON_DYING_FRAME;
        } else if self.state == DEAD {
            self.image = DRAGON_DEAD_FRAME;
        } else if rand4(&mut game.rng) == 0 {
            game.missile_mut(self.missile).speed = MISSILE_SPEED_FIREBALL;
            game.actors.next_missile = game.actors.next_missile.wrapping_add(1);
            self.image = (rand2(&mut game.rng) + 1) as i16;
            let pitch = EFFECT_FIREBALL_PITCH + rand256(&mut game.rng);
            play_effect(game, EFFECT_FIREBALL, pitch);
            game.actor_mut(actor).facing = DRAGON_BREATH_FACING;
            game.missile_mut(self.missile).missile_type = MISSILE_ROCK_TYPE;

            return Pc::DragShoot;
        }

        self.image = 0;

        Pc::Tail
    }

    /// Sets up the missile just started (also used by the dragon): where it starts, which way it
    /// goes, and who shot it.
    pub(super) fn drag_shoot(&mut self, game: &mut Game) -> Pc {
        let actor = self.actor;
        let ms = self.missile;
        let d = self.facing as usize;
        let dx = bowshotx.get(d).copied().unwrap_or(0) as i32;
        game.missile_mut(ms).abs_x = (game.actor(actor).abs_x as i32 + dx) as u16;
        game.missile_mut(ms).abs_y = game.actor(actor).abs_y;
        let dy = if game.actor(actor).weapon == WEAPON_BOW {
            bowshoty.get(d).copied().unwrap_or(0)
        } else {
            gunshoty.get(d).copied().unwrap_or(0)
        };

        game.missile_mut(ms).abs_y = (game.missile(ms).abs_y as i32 + dy as i32) as u16;
        game.missile_mut(ms).time_of_flight = 0;
        game.missile_mut(ms).direction = game.actor(actor).facing;
        game.missile_mut(ms).archer = self.slot as i8;

        if game.actors.next_missile > MAX_MISSILE {
            game.actors.next_missile = 0;
        }

        game.actors.frustration = 0;

        Pc::Cpx
    }
}
