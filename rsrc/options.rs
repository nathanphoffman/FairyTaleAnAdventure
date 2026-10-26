//! What choosing an entry of a menu does (`do_option` from csrc/fmain.c): the Items and Magic
//! menus (in `items_menu` and `magic`), and here talking, buying, the game menu, using items,
//! keys, saving and giving.

use super::actors::{eat, get_turtle, nearest_fig};
use super::asm_map::{newx, newy};
use super::asm_text::{event, prq, speak};
use super::doors::{bumped, doorfind};
use super::items_menu::items_option;
use super::magic::magic_option;
use super::menus::set_options;
use super::messages::{extract, print, print_cont};
use super::mood::setmood;
use super::objects::{SetFigEntry, leave_item};
use super::random::rand64;
use super::savegame::{savegame, svflag};
use super::tables::{jtrans, ob_list8, ob_listg};
use super::types::{InvItem, Shape};
use super::menus::{gomenu};
use super::game_tables::{inv_list, region_num, setfig_table};
use super::globals::{active_carrier, anim_list, brave, cmode, daynight, dayperiod, fatigue, hero_x, hero_y, hitgo, kind as hero_kind, luck, nearest, nearest_person, quitflag, stuff, wealth, witchflag};


// Menu numbers (`enum cmodes` in csrc/fmain.c).
const ITEMS: i32 = 0;
const MAGIC: i32 = 1;
const TALK: i32 = 2;
const BUY: i32 = 3;
const GAME: i32 = 4;
const SAVEX: i32 = 5;
const KEYS: i32 = 6;
const GIVE: i32 = 7;
const USE: i32 = 8;
const FILE: i32 = 9;

// Actor kinds and states (csrc/ftale.h).
const ENEMY: i8 = 2;
const SETFIG: i8 = 4;
const CARRIER: i8 = 5;
const DEAD: i8 = 15;
const TALKING: i8 = 19;
/// The tactic that makes a person stand and talk.
const TACTIC_TALK: i8 = 15;

// Inventory slots.
const KEYBASE: usize = 16;
const ARROWS: usize = 8;
const SHELL: usize = 6;
const WRIT: usize = 28;
const BONE: usize = 29;

// Print queue entries.
const PRQ_STATS: i32 = 4;
const PRQ_VITALITY: i32 = 7;

/// `nearest_fig` search ranges for yelling and for the quieter talk options.
const YELL_RANGE: i16 = 100;
const TALK_RANGE: i16 = 50;
/// Anyone closer than this is told off for yelling.
const TOO_CLOSE_TO_YELL: i32 = 35;

// Talk menu entries and the people (`race & 0x7f`) that can be talked to.
const YELL: i16 = 5;
const WIZARD: u8 = 0;
const PRIEST: u8 = 1;
const GUARD_FRONT: u8 = 2;
const GUARD_BACK: u8 = 3;
const PRINCESS: u8 = 4;
const KING: u8 = 5;
const NOBLE: u8 = 6;
const SORCERESS: u8 = 7;
const BARTENDER: u8 = 8;
const WITCH: u8 = 9;
const SPECTRE: u8 = 10;
const GHOST: u8 = 11;
const RANGER: u8 = 12;
const BEGGAR: u8 = 13;
/// Kindness the hero needs for the wizard and priest to be friendly.
const KIND_ENOUGH: i16 = 10;
/// Object list slots whose state the conversations change.
const PRIEST_WRIT_SLOT: usize = 10;
const SORCERESS_SLOT: usize = 9;
const PRINCESS_FREED_SLOT: usize = 9;
/// Luck given by the sorceress, and the roll it must beat.
const SORCERESS_LUCK: i16 = 5;
const CARRIER_TURTLE: i16 = 5;
/// Hero vitality when healed by the priest is `15 + brave / 4`.
const FULL_VITALITY_BASE: i16 = 15;
const PRIEST_BLESSINGS: u16 = 3;
const NIGHT_PERIOD: i16 = 7;
const TIRED: i16 = 5;
const RANGER_SWAMP_REGION: u16 = 2;

// Buy menu: the bartender is race 0x88; entries 5-11 pick a pair in `jtrans`.
const BARTENDER_RACE: u8 = 0x88;
const LAST_BUY_ENTRY: i16 = 11;
const FIRST_MENU_ENTRY: i16 = 5;
const FOOD: usize = 0;
const BOUGHT_FOOD_EVENT: i32 = 22;
const BOUGHT_ARROWS_EVENT: i32 = 23;
const FOOD_HUNGER: i32 = 50;
const ARROWS_BOUGHT: u8 = 10;

// Game menu entries.
const MUSIC: i16 = 6;
const QUIT: i16 = 8;
const LOAD: i16 = 9;
// Save/exit menu entries.
const SAVE: i16 = 5;
const EXIT: i16 = 6;

// Use menu entries.
const KEY_ENTRY: i16 = 7;
const SHELL_ENTRY: i16 = 6;
const SUNSTONE_ENTRY: i16 = 8;
const WEAPONS: i16 = 5;
const SUNSTONE_SPEECH: i32 = 60;
/// No summoning the turtle inside this box (the swamp it lives in is elsewhere).
const NO_TURTLE_AREA: (u16, u16, u16, u16) = (11194, 21373, 10205, 16208);

// Keys: the menu entries start at 5 and the nine directions to try a door in.
const DIRECTIONS: i32 = 9;
const DOOR_REACH: i32 = 16;

// Give menu.
const GIVE_GOLD: i16 = 5;
const GIVE_BONE: i16 = 8;
const GOLD_GIVEN: i16 = 2;
const BEGGAR_RACE: u8 = 0x8d;
const SPECTRE_RACE: u8 = 0x8a;
const BEGGAR_SPEECH: i32 = 24;
const BEGGAR_THANKS: i32 = 50;
const SPECTRE_NOT_HERE: i32 = 21;
const SPECTRE_THANKS: i32 = 48;
const SPECTRE_BONE_DROP: i16 = 140;

/// Does what menu entry `hit` of the current menu `cmode` says.
pub fn do_option(hit: i16) {
    unsafe {
        let refresh = match cmode as i32 {
            ITEMS => {
                items_option(hit);
                true
            }
            MAGIC => magic_option(hit),
            TALK => talk_option(hit),
            BUY => buy_option(hit),
            GAME => game_option(hit),
            USE => use_option(hit),
            SAVEX => save_exit_option(hit),
            FILE => file_option(hit),
            KEYS => keys_option(hit),
            GIVE => give_option(hit),
            _ => true,
        };
        if refresh {
            set_options();
        }
    }
}

unsafe fn add_stuff(slot: usize, amount: u8) {
    unsafe { *stuff.add(slot) = (*stuff.add(slot)).wrapping_add(amount) };
}

unsafe fn item_name(i: usize) -> *const std::ffi::c_char {
    unsafe { (*(&raw const inv_list).cast::<InvItem>().add(i)).name }
}

/// Yell (louder, reaches further) or say something to the nearest person.
unsafe fn talk_option(hit: i16) -> bool {
    unsafe {
        let distance = nearest_fig(1, if hit == YELL { YELL_RANGE } else { TALK_RANGE });
        if nearest == 0 {
            return true;
        }
        let an = &raw mut anim_list[nearest as usize];
        if (*an).state == DEAD {
            return true;
        }
        if (*an).kind == SETFIG {
            if hit == YELL && (distance as u32) < TOO_CLOSE_TO_YELL as u32 {
                speak(8);
                return true;
            }
            let person = (*an).race & 0x7f;
            if (*(&raw const setfig_table)
                .cast::<SetFigEntry>()
                .add(person as usize))
            .can_talk
                != 0
            {
                (*an).state = TALKING;
                (*an).tactic = TACTIC_TALK;
            }
            person_speaks(person, an);
        } else if (*an).kind == CARRIER && active_carrier == CARRIER_TURTLE {
            if *stuff.add(SHELL) != 0 {
                speak(57);
            } else {
                *stuff.add(SHELL) = 1;
                speak(56);
            }
        } else if (*an).kind == ENEMY {
            speak((*an).race as i32);
        }
        true
    }
}

/// What each kind of person says. Some of them also change the world (the priest heals, the
/// sorceress gives luck).
unsafe fn person_speaks(person: u8, an: *mut Shape) {
    unsafe {
        let goal = (*an).goal as i32;
        match person {
            WIZARD => speak(if hero_kind < KIND_ENOUGH {
                35
            } else {
                27 + goal
            }),
            PRIEST => priest_speaks(),
            GUARD_FRONT | GUARD_BACK => speak(15),
            PRINCESS => {
                if ob_list8[PRINCESS_FREED_SLOT].ob_stat != 0 {
                    speak(16);
                }
            }
            KING => {
                if ob_list8[PRINCESS_FREED_SLOT].ob_stat != 0 {
                    speak(17);
                }
            }
            NOBLE => speak(20),
            SORCERESS => {
                if ob_listg[SORCERESS_SLOT].ob_stat != 0 {
                    if luck < rand64() as i16 {
                        luck += SORCERESS_LUCK;
                    }
                } else {
                    speak(45);
                    ob_listg[SORCERESS_SLOT].ob_stat = 1;
                }
                prq(PRQ_VITALITY);
            }
            BARTENDER => {
                if fatigue < TIRED {
                    speak(13);
                } else if dayperiod > NIGHT_PERIOD {
                    speak(12);
                } else {
                    speak(14);
                }
            }
            WITCH => speak(46),
            SPECTRE => speak(47),
            GHOST => speak(49),
            RANGER => speak(if region_num == RANGER_SWAMP_REGION {
                22
            } else {
                53 + goal
            }),
            BEGGAR => speak(23),
            _ => {}
        }
    }
}

/// The priest gives the writ's reward, scolds the unkind, or heals.
unsafe fn priest_speaks() {
    unsafe {
        if *stuff.add(WRIT) != 0 {
            if ob_listg[PRIEST_WRIT_SLOT].ob_stat == 0 {
                speak(39);
                ob_listg[PRIEST_WRIT_SLOT].ob_stat = 1;
            } else {
                speak(19);
            }
        } else if hero_kind < KIND_ENOUGH {
            speak(40);
        } else {
            speak(36 + (daynight % PRIEST_BLESSINGS) as i32);
            anim_list[0].vitality = FULL_VITALITY_BASE + brave / 4;
            prq(PRQ_STATS);
        }
    }
}

/// Buys what the bartender sells, if the hero can pay.
unsafe fn buy_option(hit: i16) -> bool {
    unsafe {
        nearest = nearest_person;
        if nearest == 0 {
            return true;
        }
        if anim_list[nearest as usize].race != BARTENDER_RACE {
            return true;
        }
        if hit > LAST_BUY_ENTRY {
            return false;
        }
        let pair = ((hit - FIRST_MENU_ENTRY) * 2) as isize;
        let item = *jtrans.as_ptr().offset(pair) as u8 as usize;
        let price = *jtrans.as_ptr().offset(pair + 1) as i16;
        if wealth > price {
            wealth -= price;
            prq(PRQ_VITALITY);
            if item == FOOD {
                event(BOUGHT_FOOD_EVENT);
                eat(FOOD_HUNGER);
            } else if item == ARROWS {
                add_stuff(item, ARROWS_BOUGHT);
                event(BOUGHT_ARROWS_EVENT);
            } else {
                add_stuff(item, 1);
                extract(c"% bought a ".as_ptr());
                print_cont(item_name(item));
                print_cont(c".".as_ptr());
            }
        } else {
            print(c"Not enough money!".as_ptr());
        }
        true
    }
}

/// Music on/off, and going to the save/exit and load menus.
unsafe fn game_option(hit: i16) -> bool {
    unsafe {
        if hit == MUSIC {
            setmood(1);
        }
        if hit == QUIT {
            gomenu(SAVEX as i16);
        }
        if hit == LOAD {
            svflag = 0;
            gomenu(FILE as i16);
        }
        true
    }
}

/// Picks a weapon, the key menu, the turtle shell, or the sunstone.
unsafe fn use_option(hit: i16) -> bool {
    unsafe {
        if hit == KEY_ENTRY {
            if hitgo != 0 {
                gomenu(KEYS as i16);
            } else {
                extract(c"% has no keys!".as_ptr());
            }
            return false;
        }
        if hit < WEAPONS {
            if hitgo != 0 {
                anim_list[0].weapon = (hit + 1) as i8;
            } else {
                extract(c"% doesn't have one.".as_ptr());
            }
        }
        if hit == SHELL_ENTRY && hitgo != 0 {
            let (x_low, x_high, y_low, y_high) = NO_TURTLE_AREA;
            if hero_x < x_high && hero_x > x_low && hero_y < y_high && hero_y > y_low {
                return true;
            }
            get_turtle();
        }
        if hit == SUNSTONE_ENTRY && witchflag != 0 {
            speak(SUNSTONE_SPEECH);
        }
        gomenu(ITEMS as i16);
        true
    }
}

/// Saving goes to the file menu; exiting ends the game.
unsafe fn save_exit_option(hit: i16) -> bool {
    unsafe {
        if hit == EXIT {
            quitflag = 1;
        }
        if hit == SAVE {
            svflag = 1;
            gomenu(FILE as i16);
        }
        true
    }
}

/// A save slot was chosen: save or load, then back to the game menu.
fn file_option(hit: i16) -> bool {
    savegame(hit);
    gomenu(GAME as i16);
    true
}

/// Tries a key on every door and chest around the hero.
unsafe fn keys_option(hit: i16) -> bool {
    unsafe {
        let key = (hit - FIRST_MENU_ENTRY) as usize;
        bumped = 0;
        if *stuff.add(key + KEYBASE) != 0 {
            let mut direction = 0;
            while direction < DIRECTIONS {
                let x = newx(hero_x as i32, direction, DOOR_REACH) as u16;
                let y = newy(hero_y as i32, direction, DOOR_REACH) as u16;
                if doorfind(x, y, key as u32 + 1) != 0 {
                    *stuff.add(key + KEYBASE) -= 1;
                    break;
                }
                direction += 1;
            }
            if direction >= DIRECTIONS {
                extract(c"% tried a ".as_ptr());
                print_cont(item_name(KEYBASE + key));
                print_cont(c" but it didn't".as_ptr());
                print(c"fit.".as_ptr());
            }
        }
        gomenu(ITEMS as i16);
        true
    }
}

/// Gives gold to a beggar, or the bone to the spectre.
unsafe fn give_option(hit: i16) -> bool {
    unsafe {
        if nearest_person == 0 {
            return true;
        }
        let person = &raw mut anim_list[nearest_person as usize];
        let race = (*person).race;
        if hit == GIVE_GOLD && wealth > GOLD_GIVEN {
            wealth -= GOLD_GIVEN;
            if rand64() > hero_kind as i32 {
                hero_kind += 1;
            }
            prq(PRQ_STATS);
            prq(PRQ_VITALITY);
            if race == BEGGAR_RACE {
                speak(BEGGAR_SPEECH + (*person).goal as i32);
            } else {
                speak(BEGGAR_THANKS);
            }
        } else if hit == GIVE_BONE && *stuff.add(BONE) != 0 {
            if race != SPECTRE_RACE {
                speak(SPECTRE_NOT_HERE);
            } else {
                speak(SPECTRE_THANKS);
                *stuff.add(BONE) = 0;
                leave_item(nearest_person, SPECTRE_BONE_DROP);
            }
        }
        gomenu(ITEMS as i16);
        true
    }
}
