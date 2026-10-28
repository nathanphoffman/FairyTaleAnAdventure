//! Looking around: revealing hidden objects within reach.

use super::OBJECTS;
use crate::game::actors::interaction::actor_distance;
use crate::game::data::state::ActorRef;
use crate::game::data::state::Game;
use crate::game::ui::text::show_event;
use crate::game::world::objects::change_object;

/// `Look` finds hidden objects closer than this.
const LOOK_RANGE: i32 = 40;
const EVENT_NOTHING_HIDDEN: i32 = 20;
const EVENT_FOUND_HIDDEN: i32 = 38;

/// Reveals hidden objects within reach.
pub(super) fn look_around(game: &mut Game) {
    let mut found = false;

    for i in 0..game.actors.drawn as i32 {
        let actor = ActorRef::new(i as usize);

        if game.actor(actor).kind == OBJECTS
            && game.actor(actor).race == 0
            && actor_distance(game, i, 0) < LOOK_RANGE
        {
            found = true;
            change_object(game, i, 1);
        }
    }

    show_event(
        game,
        if found {
            EVENT_FOUND_HIDDEN
        } else {
            EVENT_NOTHING_HIDDEN
        },
    );
}
