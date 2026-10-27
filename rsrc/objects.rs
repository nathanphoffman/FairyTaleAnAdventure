//! Items lying in the world and the people standing around (from csrc/fmain2.c): turning the
//! object lists of the current region into actors, and changing them.
//!
//! How the world's objects work: each region has a list of objects (`ob_table`) and there is one
//! global list (`ob_listg`) for things that follow the story, like the dead brothers' bags and the
//! spectre. Each entry has a position, an id and a state. Only the entries near the screen become
//! actors in `actor_list`, so the game never handles more than 20 actors at once:
//! - people (state 3 or 4) go in slots starting at 2, counted by `j1`;
//! - ordinary items go after the monsters, counted by `drawn_actor_count`.
//! The actor's `vitality` field is borrowed to remember which list entry it came from (the entry
//! number, plus 0x80 for the global list), so `change_object` can find the entry again later.

use super::actors::turtle_eggs;
use super::asm_map::terrain_at;
use super::game_tables::setfig_table;
use super::globals::{loaded_setfig_file, nextshape, seq_list, witch_on_screen};
use super::hdrive::motor_off;
use super::loading::sprite_files;
use super::loading::{finish_sprite_load, load_sprite_file};
use super::random::bitrand;
use super::state::{the_actors, the_world};
use super::tables::{CHEST, TURTLE, dstobs, glbobs, mapobs, ob_listg, ob_table, rand_treasure};
use super::types::{SeqInfo, Shape, WorldObject};
/// Mirror of the `setfig_table` entries in csrc/fmain.c: for each kind of person (wizard, priest,
/// guard, princess, king, noble, sorceress, bartender, witch, ranger ...), which sprite file
/// holds them, which image to start from, and whether they can talk.
#[repr(C)]
pub struct SetFigEntry {
    pub cfile_entry: i8,
    pub image_base: i8,
    pub can_talk: i8,
}

// Actor kinds and states (csrc/ftale.h).
const OBJECTS: i8 = 1;
const SETFIG: i8 = 4;
const STILL: i8 = 13;
const DEAD: i8 = 15;

/// What a chest turns into when opened.
const OPENED_CHEST: i8 = 0x1d;
/// The witch's set figure id.
const WITCH: i8 = 9;

// Object states (`ob_stat`).
const STAT_HIDDEN: i8 = 0;
const STAT_TAKEN: i8 = 2;
const STAT_PERSON: i8 = 3;
const STAT_DEAD_PERSON: i8 = 4;
const STAT_RAFT_ITEM: i8 = 5;
const STAT_SECOND_ITEM: i8 = 6;
/// A visible object (also what new random treasure starts as).
const STAT_SHOWN: i8 = 1;

/// Actors 0 and 1 are the hero and the raft; objects and people start after the carrier slots.
const FIRST_OBJECT_ACTOR: i16 = 2;
const MAX_ACTORS: i16 = 20;
/// Region numbers from this one on mean the map is stable (csrc/fmain.c `MAP_STABLE`).
const NO_REGION: u16 = 10;
/// Random treasures added to a region the first time it is visited.
const RANDOM_TREASURES: usize = 10;
/// Offset of the global object list's ids in `vitality`, and the bit marking "global list".
const OBJECT_INDEX_MASK: i16 = 0x7f;
const GLOBAL_LIST_BIT: i16 = 0x80;

// How far (in pixels around the screen) people and objects are looked at.
const PERSON_LOAD_X: (i16, i16) = (-100, 400);
const PERSON_LOAD_Y: (i16, i16) = (-100, 300);
const SHOWN_X: (i16, i16) = (-20, 340);
const SHOWN_Y: (i16, i16) = (-20, 190);
const SPRITE_CENTER_OFFSET: i16 = 8;
/// People stand this much higher than their position.
const PERSON_RAISE: i16 = 18;
/// Actor `race` for a person is their id plus this.
const PERSON_RACE_BASE: i32 = 0x80;

/// First free actor slot for people while the object lists are walked.
pub static mut J1: i16 = 0;

/// Whether `value` is strictly between the two limits.
fn inside(value: i16, (low, high): (i16, i16)) -> bool {
    value > low && value < high
}

/// Turns the global and current region's object lists into actors.
/// Called when the screen changes (new region, scrolling far, after loading a game).
pub fn spawn_object_actors() {
    unsafe {
        J1 = FIRST_OBJECT_ACTOR;
        add_objects_near((&raw mut ob_listg).cast(), glbobs, GLOBAL_LIST_BIT as i32);
        add_objects_near(
            ob_table[the_world.region as usize],
            mapobs[the_world.region as usize],
            0,
        );
        if J1 > 3 {
            the_actors.active = J1;
        }
    }
}

/// Drops `object` where actor `i` stands (slot 0 of the global list is used for this).
/// That slot is reused for each item the hero or a monster leaves behind.
pub fn leave_item(i: i16, object: i16) {
    unsafe {
        let drop = &mut *(&raw mut ob_listg).cast::<WorldObject>();
        let actor = &(*(&raw const the_actors.list))[i as usize];
        drop.xc = actor.abs_x;
        drop.yc = actor.abs_y.wrapping_add(10);
        drop.ob_id = object as i8;
        drop.ob_stat = STAT_SHOWN;
    }
}

/// Changes the object that actor `id` stands for: `flag` 1 = show, 2 = take. A chest opens
/// instead (its id changes to the opened chest, rather than its state).
pub fn change_object(id: i32, flag: i32) {
    unsafe {
        let vitality = (*(&raw const the_actors.list))[id as usize].vitality;
        let index = (vitality & OBJECT_INDEX_MASK) as usize;
        let object = if vitality & GLOBAL_LIST_BIT != 0 {
            (&raw mut ob_listg).cast::<WorldObject>().add(index)
        } else {
            ob_table[the_world.region as usize].add(index)
        };
        if (*object).ob_id == CHEST as i8 {
            (*object).ob_id = OPENED_CHEST;
        } else {
            (*object).ob_stat = flag as i8;
        }
    }
}

/// Scatters random treasure over a region the first time it is visited: 10 items at random
/// spots, retrying until the spot is not blocked (terrain type 0 means clear), with the item picked
/// from `rand_treasure`. `dstobs` remembers that it has been done.
unsafe fn distribute_treasure() {
    unsafe {
        let region = the_world.region as usize;
        let objects = ob_table[region];
        for _ in 0..RANDOM_TREASURES {
            let (mut x, mut y);
            loop {
                x = (bitrand(0x3fff) + ((the_world.region & 1) as i32 * 0x4000)) as i16;
                y = (bitrand(0x1fff) + ((the_world.region & 6) as i32 * 0x1000)) as i16;
                if terrain_at(x as i32, y as i32) == 0 {
                    break;
                }
            }
            let k = mapobs[region] as usize;
            mapobs[region] += 1;
            let object = &mut *objects.add(k);
            object.xc = x as u16;
            object.yc = y as u16;
            object.ob_id = rand_treasure[bitrand(15) as usize] as i8;
            object.ob_stat = STAT_SHOWN;
        }
        dstobs[region] = 1; // mark this region as distributed
    }
}

/// Makes sure the sprite file for the kind of person `id` is loaded. Several kinds share one
/// file, so it is only reloaded when the needed file differs from the loaded one.
unsafe fn load_set_figure(id: i8) {
    unsafe {
        let entry = (*(&raw const setfig_table)
            .cast::<SetFigEntry>()
            .add(id as usize))
        .cfile_entry;
        if sprite_files[entry as usize].file_id != sprite_files[loaded_setfig_file as usize].file_id
        {
            loaded_setfig_file = entry as i16;
            nextshape = (*(&raw mut seq_list).cast::<SeqInfo>().add(SETFIG as usize)).location;
            load_sprite_file(entry as i32);
            finish_sprite_load(SETFIG as i16);
            motor_off();
        }
    }
}

/// Turns the objects of one list into actors if they are close enough to the screen. `f` is
/// added to the object's number to make the actor's `vitality` (0x80 marks the global list).
///
/// For each entry: work out where it is on screen (the sprite is 16 wide, so 8 is subtracted to
/// centre it); people are only loaded if within about 100-400 pixels, and shown if within the
/// screen; items that are hidden or taken are skipped. A person already in the actor list at the
/// same place is not added twice (and is marked dead if its actor died).
pub fn add_objects_near(mut list: *mut WorldObject, mut length: i16, f: i32) {
    unsafe {
        turtle_eggs = 0;
        witch_on_screen = 0;
        if dstobs[the_world.region as usize] == 0 && the_world.new_region >= NO_REGION {
            distribute_treasure();
        }
        let mut index: i16 = 0;
        while length > 0 {
            length -= 1;
            if the_actors.drawn >= MAX_ACTORS {
                return;
            }
            add_object(&mut *list, index, f);
            list = list.add(1); // next object
            index += 1;
        }
    }
}

/// Adds one object of a list as an actor, if it is near enough and not hidden or taken.
unsafe fn add_object(object: &mut WorldObject, index: i16, f: i32) {
    unsafe {
        let xstart =
            (object.xc as i32 - the_world.map_x as i32 - SPRITE_CENTER_OFFSET as i32) as i16;
        let ystart =
            (object.yc as i32 - the_world.map_y as i32 - SPRITE_CENTER_OFFSET as i32) as i16;
        // States 3 and 4 are people; everything else is an item.
        let is_person = object.ob_stat == STAT_PERSON || object.ob_stat == STAT_DEAD_PERSON;
        let id = object.ob_id;
        if is_person {
            if !(inside(xstart, PERSON_LOAD_X) && inside(ystart, PERSON_LOAD_Y)) {
                return;
            }
            load_set_figure(id);
            if object.ob_id == WITCH && object.ob_stat == STAT_PERSON {
                witch_on_screen = 1;
            }
        }
        if object.ob_stat == STAT_TAKEN || object.ob_stat == STAT_HIDDEN {
            return;
        }
        if !(inside(xstart, SHOWN_X) && inside(ystart, SHOWN_Y)) {
            return;
        }
        let (actor, ystart) = if is_person {
            match add_person_actor(object, index, ystart) {
                Some(placed) => placed,
                None => return,
            }
        } else {
            (add_item_actor(object, index, f), ystart)
        };
        actor.abs_x = object.xc;
        actor.abs_y = object.yc;
        actor.rel_x = xstart as u16;
        actor.rel_y = ystart as u16;
    }
}

/// Makes a person (a set figure) out of a world object. Returns the actor and the screen row to use
/// (people stand a little higher than the object's spot), or `None` if the person is already
/// there as an actor (the object is then marked dead if the actor died).
unsafe fn add_person_actor(
    object: &mut WorldObject,
    index: i16,
    ystart: i16,
) -> Option<(&'static mut Shape, i16)> {
    unsafe {
        let id = object.ob_id;
        let actor = &mut (*(&raw mut the_actors.list))[J1 as usize];
        J1 += 1;
        if actor.abs_x == object.xc && actor.abs_y == object.yc {
            if actor.state == DEAD {
                object.ob_stat = STAT_DEAD_PERSON;
            }
            return None;
        }
        actor.kind = SETFIG;
        actor.index = (*(&raw const setfig_table)
            .cast::<SetFigEntry>()
            .add(id as usize))
        .image_base;
        actor.weapon = 0;
        actor.vitality = 2 + id as i16 + id as i16;
        actor.goal = index as i8;
        actor.environ = 0;
        actor.state = if object.ob_stat == STAT_DEAD_PERSON {
            DEAD
        } else {
            STILL
        };
        // A person's race is 0x80 plus their kind, so they can be told from monsters.
        actor.race = (id as i32 + PERSON_RACE_BASE) as u8;
        if J1 > the_actors.drawn {
            the_actors.drawn = J1;
        }
        Some((actor, ystart - PERSON_RAISE))
    }
}

/// Makes an item lying about (or the turtle's eggs, a raft...) out of a world object.
unsafe fn add_item_actor(object: &WorldObject, index: i16, f: i32) -> &'static mut Shape {
    unsafe {
        if object.ob_id == TURTLE as i8 {
            turtle_eggs = the_actors.drawn as i8;
        }
        let actor = &mut (*(&raw mut the_actors.list))[the_actors.drawn as usize];
        the_actors.drawn += 1;
        actor.kind = OBJECTS;
        actor.index = object.ob_id;
        actor.weapon = 0;
        actor.vitality = (index as i32 + f) as i16;
        actor.race = match object.ob_stat {
            STAT_SECOND_ITEM => 2,
            STAT_RAFT_ITEM => 0,
            _ => 1,
        };
        actor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn struct_sizes_match_the_c_structs() {
        assert_eq!(size_of::<SetFigEntry>(), 3);
    }

    #[test]
    fn inside_excludes_the_edges() {
        assert!(inside(0, SHOWN_X));
        assert!(!inside(-20, SHOWN_X));
        assert!(!inside(340, SHOWN_X));
    }
}
