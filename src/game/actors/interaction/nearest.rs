//! Finding the nearest actor to the hero and the distance between two actors.

use super::OBJECTS;
use super::hits::actor_at;
use crate::game::data::state::Game;
use std::ffi::c_char;

/// Finds the nearest actor to the hero closer than `dist`, setting `nearest_object`; returns the
/// distance found. With `constraint` set, objects are ignored too.
pub fn find_nearest_figure(game: &mut Game, constraint: c_char, mut dist: i16) -> i32 {
    /// Index of the "still" object type that always counts.
    const ALWAYS_COUNTED_OBJECT: i8 = 0x1d;
    game.actors.nearest_object = 0;

    for i in 1..game.actors.drawn as i32 {
        let a = actor_at(i);

        if game.actor(a).kind == OBJECTS
            && (constraint != 0 || game.actor(a).index == ALWAYS_COUNTED_OBJECT)
        {
            continue;
        }

        let d = actor_distance(game, i, 0);

        if d < dist as i32 {
            game.actors.nearest_object = i as i16;
            dist = d as i16;
        }
    }

    dist as i32
}

/// The diagonal is approximated as 5/7 of the sum of the two offsets.
const DIAGONAL_NUMERATOR: i32 = 5;
const DIAGONAL_DENOMINATOR: i32 = 7;

/// Distance between two actors. The game avoids a square root: if one axis is more than twice
/// the other it is used alone, otherwise (x + y) * 5 / 7 approximates the diagonal.
pub fn actor_distance(game: &Game, a: i32, b: i32) -> i32 {
    let (a, b) = (
        &game.actors.list[actor_at(a).slot()],
        &game.actors.list[actor_at(b).slot()],
    );
    let x = (a.abs_x as i32 - b.abs_x as i32).abs();
    let y = (a.abs_y as i32 - b.abs_y as i32).abs();

    if x > y + y {
        x
    } else if y > x + x {
        y
    } else {
        (x + y) * DIAGONAL_NUMERATOR / DIAGONAL_DENOMINATOR
    }
}
