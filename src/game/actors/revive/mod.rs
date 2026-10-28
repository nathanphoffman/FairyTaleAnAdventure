//! Bringing the hero back (from csrc/fmain.c): starting a brother's adventure at the beginning, or
//! reviving the same brother at the last safe spot after the good fairy.
//!
//! The three brothers (Julian, Phillip, Kevin) play in turn. When one dies for good the next takes
//! over, with the first one's bones and goods left where they fell, until the third has died.

mod brothers;

mod safe_spot;

use self::brothers::{
    LAST_BROTHER, announce_brother, leave_fallen_brother, reset_raft_setfig, show_brother_placards,
    take_over_next_brother,
};
use crate::game::data::state::Game;

use self::safe_spot::{put_hero_at_safe_spot, restore_hero};

use crate::game::data::state::ActorRef;
use crate::game::render::display::reset_input;
use crate::game::render::screen_fx::fade_down;
use crate::game::ui::music::set_mood;

use crate::platform::audio::stopscore;

// Actor kinds and goals.
const PHIL: i8 = 0;
const USER: i8 = 0;

/// Value of `view_status` for "fade in" (3) and "game over screen" (2).
const VIEW_FADE_IN: i8 = 3;
const VIEW_GAME_OVER: i8 = 2;

/// Revives the hero. With `new` set, a brother starts fresh: the previous one's body is left in
/// the world, the next brother's stats and an empty pack (just a dirk) are loaded, the story text
/// is shown, and the hero starts at the first safe spot. Without `new` (the good fairy), the same
/// brother returns to the last safe spot after a fade.
///
/// Either way the hero gets full health for the bravery, the clock is set to the morning
/// (8000), and hunger and fatigue are cleared.
pub fn revive(game: &mut Game, new: i16) {
    reset_raft_setfig(game);

    let hero = ActorRef::new(0);
    game.actors.list[hero.slot()].kind = PHIL;
    game.actors.list[hero.slot()].goal = USER; // yes, we can usurp control from the user

    reset_input();
    game.actors.battle_flag = 0;
    game.hero.good_fairy_timer = 0;
    game.actors.next_missile = 0;

    if new != 0 {
        stopscore();
        leave_fallen_brother(game);
        take_over_next_brother(game, hero);
        show_brother_placards(game);
        announce_brother(game);
    } else {
        fade_down(game);
    }

    put_hero_at_safe_spot(game, hero);
    restore_hero(game, hero);

    if game.hero.brother > LAST_BROTHER {
        game.screen.view_status = VIEW_GAME_OVER;
    } else {
        game.screen.view_status = VIEW_FADE_IN;
        set_mood(game, 1);
    }

    game.world.extent_type = 0;
    game.actors.fiery_death = 0;
}

#[cfg(test)]
mod tests {
    use super::brothers::{
        FIRST_REGION, FIRST_SAFE_SPOT, GOLDBASE, reset_for_next_brother, start_next_brother,
    };
    use crate::game::data::types::Shape;

    use super::safe_spot::{
        CAMERA_OFFSET, START_DAYNIGHT, START_LIGHT, STILL, move_to_safe_spot, restore_stats,
    };

    use crate::game::data::state::{Hero, STUFF_SLOTS, World};
    use crate::game::data::types::Bro;

    #[test]
    fn a_new_brother_starts_clean_at_the_first_safe_spot() {
        let (mut hero, mut world): (Hero, World) = (Default::default(), Default::default());
        world.secret_timer = 5;
        world.light_timer = 6;
        world.freeze_timer = 7;
        world.region = 9;
        reset_for_next_brother(&mut hero, &mut world);
        assert_eq!(
            (world.secret_timer, world.light_timer, world.freeze_timer),
            (0, 0, 0)
        );

        assert_eq!(world.region, FIRST_REGION);
        assert_eq!(
            (hero.safe_x, hero.safe_y, hero.safe_r),
            (FIRST_SAFE_SPOT.0, FIRST_SAFE_SPOT.1, FIRST_REGION)
        );
    }

    #[test]
    fn the_hero_reappears_at_the_safe_spot_with_the_camera_centred() {
        let (mut hero, mut body, mut world): (Hero, Shape, World) =
            (Default::default(), Default::default(), Default::default());
        hero.safe_x = 1000;
        hero.safe_y = 2000;
        hero.safe_r = 5;
        move_to_safe_spot(&mut hero, &mut body, &mut world);
        assert_eq!(
            (hero.x, hero.y, body.abs_x, body.abs_y),
            (1000, 2000, 1000, 2000)
        );

        assert_eq!(
            (world.map_x, world.map_y),
            (1000 - CAMERA_OFFSET.0, 2000 - CAMERA_OFFSET.1)
        );

        assert_eq!(world.new_region, 5);
    }

    #[test]
    fn revived_hero_is_healthy_rested_and_it_is_morning() {
        let (mut hero, mut body, mut world): (Hero, Shape, World) =
            (Default::default(), Default::default(), Default::default());
        hero.brave = 40;
        hero.hunger = 90;
        hero.fatigue = 80;
        body.state = 15;
        body.environ = 3;
        world.day_clock = 100;
        restore_stats(&mut hero, &mut body, &mut world);
        assert_eq!(
            (body.vitality, body.state, body.environ, body.race),
            (25, STILL, 0, 255)
        );

        assert_eq!((hero.hunger, hero.fatigue), (0, 0));
        assert_eq!(
            (world.day_clock, world.light_level),
            (START_DAYNIGHT, START_LIGHT)
        );
    }

    #[test]
    fn next_brother_starts_with_his_own_stats_and_only_a_dirk() {
        let next = Bro {
            brave: 20,
            luck: 35,
            kind: 15,
            wealth: 15,
        };

        let mut hero = Hero {
            brother: 1,
            philstuff: [7; STUFF_SLOTS],
            ..Default::default()
        };

        let mut body: Shape = Default::default();

        start_next_brother(&mut hero, &mut body, &next);
        assert_eq!(
            (hero.brave, hero.luck, hero.kindness, hero.wealth),
            (20, 35, 15, 15)
        );

        assert_eq!(hero.brother, 2);
        assert_eq!(body.weapon, 1);
        assert_eq!(hero.philstuff[0], 1);
        assert!(hero.philstuff[1..GOLDBASE].iter().all(|&n| n == 0));
        assert_eq!(hero.philstuff[GOLDBASE], 7); // what is past the item slots is left alone
    }
}
