//! What choosing an entry of a menu does (`choose_menu_entry` from csrc/fmain.c): the Items and Magic
//! menus (in `items_menu` and `magic`), and here talking, buying, the game menu, using items,
//! keys, saving and giving.

use crate::game::actors::interaction::{eat_food, find_nearest_figure, summon_turtle};
use crate::game::data::object_tables::{jtrans, ob_list8, ob_listg};
use crate::game::data::random::rand64;
use crate::game::data::state::{the_actors, the_hero, the_ui, the_witch, the_world};
use crate::game::data::tables::{item_list, setfig_table};
use crate::game::data::types::{InvItem, Shape};
use crate::game::files::savegame::{is_saving, savegame};
use crate::game::ui::items_menu::items_option;
use crate::game::ui::magic::magic_option;
use crate::game::ui::menus::go_to_menu;
use crate::game::ui::menus::update_menu_availability;
use crate::game::ui::messages::{print, print_cont, print_wrapped};
use crate::game::ui::music::set_mood;
use crate::game::ui::text::{queue_print, show_event, speak};
use crate::game::world::key_doors::{bumped_locked_door, open_door_near};
use crate::game::world::map::{step_x, step_y};
use crate::game::world::objects::{SetFigEntry, leave_item};

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

/// `find_nearest_figure` search ranges for yelling and for the quieter talk options.
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

/// Does what menu entry `chosen_entry` of the current menu `current_menu` says.
pub fn choose_menu_entry(chosen_entry: i16) {
    unsafe {
        let refresh = match the_ui.current_menu as i32 {
            ITEMS => {
                items_option(chosen_entry);
                true
            }
            MAGIC => magic_option(chosen_entry),
            TALK => talk_option(chosen_entry),
            BUY => buy_option(chosen_entry),
            GAME => game_option(chosen_entry),
            USE => use_option(chosen_entry),
            SAVEX => save_exit_option(chosen_entry),
            FILE => file_option(chosen_entry),
            KEYS => keys_option(chosen_entry),
            GIVE => give_option(chosen_entry),
            _ => true,
        };
        if refresh {
            update_menu_availability();
        }
    }
}

unsafe fn add_stuff(slot: usize, amount: u8) {
    unsafe { *the_hero.stuff.add(slot) = (*the_hero.stuff.add(slot)).wrapping_add(amount) };
}

unsafe fn item_name(i: usize) -> *const std::ffi::c_char {
    unsafe { (*(&raw const item_list).cast::<InvItem>().add(i)).name }
}

/// Yell (louder, reaches further) or say something to the nearest person.
unsafe fn talk_option(chosen_entry: i16) -> bool {
    unsafe {
        let distance = find_nearest_figure(
            1,
            if chosen_entry == YELL {
                YELL_RANGE
            } else {
                TALK_RANGE
            },
        );
        if the_actors.nearest_object == 0 {
            return true;
        }
        let an = &raw mut the_actors.list[the_actors.nearest_object as usize];
        let person = (*an).race & 0x7f;
        let can_talk = (*(&raw const setfig_table)
            .cast::<SetFigEntry>()
            .add(person as usize))
        .can_talk
            != 0;
        let target = Target {
            kind: (*an).kind,
            state: (*an).state,
            race: (*an).race,
            can_talk,
        };
        match talk_action(
            &target,
            chosen_entry == YELL,
            distance,
            the_actors.active_carrier,
            *the_hero.stuff.add(SHELL) != 0,
        ) {
            TalkAction::Nothing => {}
            TalkAction::TooCloseToYell => speak(8),
            TalkAction::Person { starts_talking } => {
                if starts_talking {
                    (*an).state = TALKING;
                    (*an).tactic = TACTIC_TALK;
                }
                person_speaks(person, an);
            }
            TalkAction::TurtleHasShell => speak(57),
            TalkAction::TurtleGivesShell => {
                *the_hero.stuff.add(SHELL) = 1;
                speak(56);
            }
            TalkAction::EnemyShouts(race) => speak(race as i32),
        }
        true
    }
}

/// Who the hero is talking to.
struct Target {
    kind: i8,
    state: i8,
    race: u8,
    /// The set figure's table entry says it can talk.
    can_talk: bool,
}

/// What a Talk or Yell does.
#[derive(Debug, PartialEq, Eq)]
enum TalkAction {
    Nothing,
    /// Yelling at someone standing right next to you.
    TooCloseToYell,
    /// A person answers (and turns to talk if it can).
    Person {
        starts_talking: bool,
    },
    TurtleHasShell,
    TurtleGivesShell,
    /// An enemy says its line (the speech number is its race).
    EnemyShouts(u8),
}

fn talk_action(
    target: &Target,
    yelling: bool,
    distance: i32,
    active_carrier: i16,
    has_shell: bool,
) -> TalkAction {
    if target.state == DEAD {
        return TalkAction::Nothing;
    }
    match target.kind {
        SETFIG if yelling && (distance as u32) < TOO_CLOSE_TO_YELL as u32 => {
            TalkAction::TooCloseToYell
        }
        SETFIG => TalkAction::Person {
            starts_talking: target.can_talk,
        },
        CARRIER if active_carrier == CARRIER_TURTLE => {
            if has_shell {
                TalkAction::TurtleHasShell
            } else {
                TalkAction::TurtleGivesShell
            }
        }
        ENEMY => TalkAction::EnemyShouts(target.race),
        _ => TalkAction::Nothing,
    }
}

/// What each kind of person says. Some of them also change the world (the priest heals, the
/// sorceress gives luck).
unsafe fn person_speaks(person: u8, an: *mut Shape) {
    unsafe {
        match person {
            PRIEST => priest_speaks(),
            SORCERESS => sorceress_speaks(),
            _ => {
                let facts = Talker {
                    kindness: the_hero.kindness,
                    fatigue: the_hero.fatigue,
                    day_period: the_world.day_period,
                    region: the_world.region,
                    goal: (*an).goal as i32,
                    princess_freed: ob_list8[PRINCESS_FREED_SLOT].ob_stat != 0,
                };
                if let Some(line) = plain_speech(person, &facts) {
                    speak(line);
                }
            }
        }
    }
}

/// What decides the lines of the people who only talk.
struct Talker {
    kindness: i16,
    fatigue: i16,
    day_period: i16,
    region: u16,
    /// The person's goal, which picks one of several lines for the wizard and the ranger.
    goal: i32,
    princess_freed: bool,
}

/// The speech number a person says, if they say anything. The priest and the sorceress are not here
/// because they also change things.
fn plain_speech(person: u8, t: &Talker) -> Option<i32> {
    match person {
        WIZARD => Some(if t.kindness < KIND_ENOUGH {
            35
        } else {
            27 + t.goal
        }),
        GUARD_FRONT | GUARD_BACK => Some(15),
        PRINCESS if t.princess_freed => Some(16),
        KING if t.princess_freed => Some(17),
        NOBLE => Some(20),
        BARTENDER => Some(if t.fatigue < TIRED {
            13
        } else if t.day_period > NIGHT_PERIOD {
            12
        } else {
            14
        }),
        WITCH => Some(46),
        SPECTRE => Some(47),
        GHOST => Some(49),
        RANGER => Some(if t.region == RANGER_SWAMP_REGION {
            22
        } else {
            53 + t.goal
        }),
        BEGGAR => Some(23),
        _ => None,
    }
}

/// The sorceress gives luck once she has been freed; before that she asks for help.
unsafe fn sorceress_speaks() {
    unsafe {
        let freed = ob_listg[SORCERESS_SLOT].ob_stat != 0;
        match sorceress_action(freed, the_hero.luck, || rand64() as i16) {
            SorceressAction::GiveLuck => the_hero.luck += SORCERESS_LUCK,
            SorceressAction::Nothing => {}
            SorceressAction::AskForHelp => {
                speak(45);
                ob_listg[SORCERESS_SLOT].ob_stat = 1;
            }
        }
        queue_print(PRQ_VITALITY);
    }
}

#[derive(Debug, PartialEq, Eq)]
enum SorceressAction {
    /// She is free and the roll was kind to the hero.
    GiveLuck,
    /// She is free but the roll was not high enough.
    Nothing,
    /// First meeting.
    AskForHelp,
}

/// `roll` is a 0-63 draw, taken only once she is free: luck is given when the hero's luck is below it.
fn sorceress_action(freed: bool, luck: i16, roll: impl FnOnce() -> i16) -> SorceressAction {
    if !freed {
        SorceressAction::AskForHelp
    } else if luck < roll() {
        SorceressAction::GiveLuck
    } else {
        SorceressAction::Nothing
    }
}

/// The priest gives the writ's reward, scolds the unkind, or heals.
unsafe fn priest_speaks() {
    unsafe {
        let has_writ = *the_hero.stuff.add(WRIT) != 0;
        match priest_action(
            has_writ,
            ob_listg[PRIEST_WRIT_SLOT].ob_stat,
            the_hero.kindness,
            the_world.day_clock,
        ) {
            PriestAction::WritReward => {
                speak(39);
                ob_listg[PRIEST_WRIT_SLOT].ob_stat = 1;
            }
            PriestAction::WritAgain => speak(19),
            PriestAction::Scold => speak(40),
            PriestAction::Bless { speech } => {
                speak(speech);
                the_actors.list[0].vitality = FULL_VITALITY_BASE + the_hero.brave / 4;
                queue_print(PRQ_STATS);
            }
        }
    }
}

/// What the priest does for the hero.
#[derive(Debug, PartialEq, Eq)]
enum PriestAction {
    /// Has the writ and the reward (object list slot) is still hidden: it appears.
    WritReward,
    /// Has the writ, reward already given.
    WritAgain,
    /// Not kind enough.
    Scold,
    /// Heals fully, with one of three blessings picked by the time of day.
    Bless { speech: i32 },
}

fn priest_action(has_writ: bool, reward_stat: i8, kindness: i16, clock: u16) -> PriestAction {
    if has_writ {
        if reward_stat == 0 {
            PriestAction::WritReward
        } else {
            PriestAction::WritAgain
        }
    } else if kindness < KIND_ENOUGH {
        PriestAction::Scold
    } else {
        PriestAction::Bless {
            speech: 36 + (clock % PRIEST_BLESSINGS) as i32,
        }
    }
}

/// What a click in the Buy menu does.
#[derive(Debug, PartialEq, Eq)]
enum BuyGate {
    /// Nobody (or not a bartender) near: nothing happens, the menu stays.
    Nothing,
    /// An entry past the last one: not handled here.
    NotHandled,
    /// Offers `item` for `price`.
    Offer { item: usize, price: i16 },
}

/// `race` is that of the nearest person (only looked at when `person` is not 0).
fn buy_gate(person: i16, race: u8, entry: i16, table: &[i8]) -> BuyGate {
    if person == 0 || race != BARTENDER_RACE {
        return BuyGate::Nothing;
    }
    if entry > LAST_BUY_ENTRY {
        return BuyGate::NotHandled;
    }
    let pair = ((entry - FIRST_MENU_ENTRY) * 2) as usize;
    BuyGate::Offer {
        item: table[pair] as u8 as usize,
        price: table[pair + 1] as i16,
    }
}

/// The hero must have strictly more gold than the price.
fn can_afford(gold: i16, price: i16) -> bool {
    gold > price
}

#[derive(Debug, PartialEq, Eq)]
enum Bought {
    Food,
    Arrows,
    Item,
}

#[derive(Debug, PartialEq, Eq)]
enum Purchase {
    TooPoor,
    Bought { wealth_left: i16, what: Bought },
}

/// Buying `item` for `price` with `wealth` gold.
fn purchase(wealth: i16, price: i16, item: usize) -> Purchase {
    if !can_afford(wealth, price) {
        return Purchase::TooPoor;
    }
    let what = if item == FOOD {
        Bought::Food
    } else if item == ARROWS {
        Bought::Arrows
    } else {
        Bought::Item
    };
    Purchase::Bought {
        wealth_left: wealth - price,
        what,
    }
}

/// Buys what the bartender sells, if the hero can pay.
unsafe fn buy_option(chosen_entry: i16) -> bool {
    unsafe {
        the_actors.nearest_object = the_actors.nearest_person;
        let near = the_actors.nearest_object;
        let race = if near == 0 {
            0
        } else {
            the_actors.list[near as usize].race
        };
        let (item, price) = match buy_gate(near, race, chosen_entry, &jtrans) {
            BuyGate::Nothing => return true,
            BuyGate::NotHandled => return false,
            BuyGate::Offer { item, price } => (item, price),
        };
        match purchase(the_hero.wealth, price, item) {
            Purchase::TooPoor => print(c"Not enough money!".as_ptr()),
            Purchase::Bought { wealth_left, what } => {
                the_hero.wealth = wealth_left;
                queue_print(PRQ_VITALITY);
                match what {
                    Bought::Food => {
                        show_event(BOUGHT_FOOD_EVENT);
                        eat_food(FOOD_HUNGER);
                    }
                    Bought::Arrows => {
                        add_stuff(item, ARROWS_BOUGHT);
                        show_event(BOUGHT_ARROWS_EVENT);
                    }
                    Bought::Item => {
                        add_stuff(item, 1);
                        print_wrapped(c"% bought a ".as_ptr());
                        print_cont(item_name(item));
                        print_cont(c".".as_ptr());
                    }
                }
            }
        }
        true
    }
}

/// Music on/off, and going to the save/exit and load menus.
unsafe fn game_option(chosen_entry: i16) -> bool {
    unsafe {
        if chosen_entry == MUSIC {
            set_mood(1);
        }
        if chosen_entry == QUIT {
            go_to_menu(SAVEX as i16);
        }
        if chosen_entry == LOAD {
            is_saving = 0;
            go_to_menu(FILE as i16);
        }
        true
    }
}

/// Picks a weapon, the key menu, the turtle shell, or the sunstone.
unsafe fn use_option(chosen_entry: i16) -> bool {
    unsafe {
        if chosen_entry == KEY_ENTRY {
            if the_ui.menu_hit_enabled != 0 {
                go_to_menu(KEYS as i16);
            } else {
                print_wrapped(c"% has no keys!".as_ptr());
            }
            return false;
        }
        if chosen_entry < WEAPONS {
            if the_ui.menu_hit_enabled != 0 {
                the_actors.list[0].weapon = (chosen_entry + 1) as i8;
            } else {
                print_wrapped(c"% doesn't have one.".as_ptr());
            }
        }
        if chosen_entry == SHELL_ENTRY && the_ui.menu_hit_enabled != 0 {
            let (x_low, x_high, y_low, y_high) = NO_TURTLE_AREA;
            if the_hero.x < x_high
                && the_hero.x > x_low
                && the_hero.y < y_high
                && the_hero.y > y_low
            {
                return true;
            }
            summon_turtle();
        }
        if chosen_entry == SUNSTONE_ENTRY && the_witch.on_screen != 0 {
            speak(SUNSTONE_SPEECH);
        }
        go_to_menu(ITEMS as i16);
        true
    }
}

/// Saving goes to the file menu; exiting ends the game.
unsafe fn save_exit_option(chosen_entry: i16) -> bool {
    unsafe {
        if chosen_entry == EXIT {
            the_ui.quit_requested = 1;
        }
        if chosen_entry == SAVE {
            is_saving = 1;
            go_to_menu(FILE as i16);
        }
        true
    }
}

/// A save slot was chosen: save or load, then back to the game menu.
fn file_option(chosen_entry: i16) -> bool {
    savegame(chosen_entry);
    go_to_menu(GAME as i16);
    true
}

/// Tries a key on every door and chest around the hero.
unsafe fn keys_option(chosen_entry: i16) -> bool {
    unsafe {
        let key = (chosen_entry - FIRST_MENU_ENTRY) as usize;
        bumped_locked_door = 0;
        if *the_hero.stuff.add(key + KEYBASE) != 0 {
            let mut direction = 0;
            while direction < DIRECTIONS {
                let x = step_x(the_hero.x as i32, direction, DOOR_REACH) as u16;
                let y = step_y(the_hero.y as i32, direction, DOOR_REACH) as u16;
                if open_door_near(x, y, key as u32 + 1) != 0 {
                    *the_hero.stuff.add(key + KEYBASE) -= 1;
                    break;
                }
                direction += 1;
            }
            if direction >= DIRECTIONS {
                print_wrapped(c"% tried a ".as_ptr());
                print_cont(item_name(KEYBASE + key));
                print_cont(c" but it didn't".as_ptr());
                print(c"fit.".as_ptr());
            }
        }
        go_to_menu(ITEMS as i16);
        true
    }
}

/// Gives gold to a beggar, or the bone to the spectre.
unsafe fn give_option(chosen_entry: i16) -> bool {
    unsafe {
        if the_actors.nearest_person == 0 {
            return true;
        }
        let person = &raw mut the_actors.list[the_actors.nearest_person as usize];
        let race = (*person).race;
        if chosen_entry == GIVE_GOLD && the_hero.wealth > GOLD_GIVEN {
            the_hero.wealth -= GOLD_GIVEN;
            if rand64() > the_hero.kindness as i32 {
                the_hero.kindness += 1;
            }
            queue_print(PRQ_STATS);
            queue_print(PRQ_VITALITY);
            if race == BEGGAR_RACE {
                speak(BEGGAR_SPEECH + (*person).goal as i32);
            } else {
                speak(BEGGAR_THANKS);
            }
        } else if chosen_entry == GIVE_BONE && *the_hero.stuff.add(BONE) != 0 {
            if race != SPECTRE_RACE {
                speak(SPECTRE_NOT_HERE);
            } else {
                speak(SPECTRE_THANKS);
                *the_hero.stuff.add(BONE) = 0;
                leave_item(the_actors.nearest_person, SPECTRE_BONE_DROP);
            }
        }
        go_to_menu(ITEMS as i16);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHOP: [i8; 14] = [0, 3, 8, 10, 11, 15, 1, 30, 2, 45, 3, 75, 13, 20];

    #[test]
    fn priest_rules() {
        assert_eq!(priest_action(true, 0, 0, 0), PriestAction::WritReward);
        assert_eq!(priest_action(true, 1, 99, 0), PriestAction::WritAgain);
        assert_eq!(
            priest_action(false, 0, KIND_ENOUGH - 1, 0),
            PriestAction::Scold
        );
        assert_eq!(
            priest_action(false, 0, KIND_ENOUGH, 0),
            PriestAction::Bless { speech: 36 }
        );
        assert_eq!(
            priest_action(false, 0, KIND_ENOUGH, 1),
            PriestAction::Bless { speech: 37 }
        );
        let wrap = PRIEST_BLESSINGS;
        assert_eq!(
            priest_action(false, 0, KIND_ENOUGH, wrap),
            PriestAction::Bless { speech: 36 }
        );
    }

    #[test]
    fn buy_gate_rules() {
        let first = FIRST_MENU_ENTRY;
        assert_eq!(buy_gate(0, BARTENDER_RACE, first, &SHOP), BuyGate::Nothing);
        assert_eq!(
            buy_gate(3, BARTENDER_RACE + 1, first, &SHOP),
            BuyGate::Nothing
        );
        assert_eq!(
            buy_gate(3, BARTENDER_RACE, LAST_BUY_ENTRY + 1, &SHOP),
            BuyGate::NotHandled
        );
        assert_eq!(
            buy_gate(3, BARTENDER_RACE, first, &SHOP),
            BuyGate::Offer { item: 0, price: 3 }
        );
        assert_eq!(
            buy_gate(3, BARTENDER_RACE, first + 1, &SHOP),
            BuyGate::Offer { item: 8, price: 10 }
        );
        // not a bartender wins over an entry past the end
        assert_eq!(
            buy_gate(3, BARTENDER_RACE + 1, LAST_BUY_ENTRY + 1, &SHOP),
            BuyGate::Nothing
        );
    }

    #[test]
    fn price_must_be_strictly_less_than_wealth() {
        assert!(!can_afford(3, 3));
        assert!(can_afford(4, 3));
        assert!(!can_afford(0, 3));
    }

    #[test]
    fn purchases() {
        assert_eq!(purchase(3, 3, 0), Purchase::TooPoor);
        assert_eq!(
            purchase(10, 3, FOOD),
            Purchase::Bought {
                wealth_left: 7,
                what: Bought::Food
            }
        );
        assert_eq!(
            purchase(10, 3, ARROWS),
            Purchase::Bought {
                wealth_left: 7,
                what: Bought::Arrows
            }
        );
        assert_eq!(purchase(10, 10, 12), Purchase::TooPoor);
        assert_eq!(
            purchase(11, 10, 12),
            Purchase::Bought {
                wealth_left: 1,
                what: Bought::Item
            }
        );
    }

    fn target(kind: i8, state: i8, can_talk: bool) -> Target {
        Target {
            kind,
            state,
            race: 7,
            can_talk,
        }
    }

    #[test]
    fn talking() {
        use TalkAction::*;
        let figure = target(SETFIG, 0, true);
        assert_eq!(
            talk_action(&figure, false, 10, 0, false),
            Person {
                starts_talking: true
            }
        );
        assert_eq!(
            talk_action(&target(SETFIG, 0, false), false, 10, 0, false),
            Person {
                starts_talking: false
            }
        );
        // yelling at someone close is refused; far enough is fine; talking is always fine
        assert_eq!(
            talk_action(&figure, true, TOO_CLOSE_TO_YELL - 1, 0, false),
            TooCloseToYell
        );
        assert_eq!(
            talk_action(&figure, true, TOO_CLOSE_TO_YELL, 0, false),
            Person {
                starts_talking: true
            }
        );
        assert_eq!(
            talk_action(&figure, false, 1, 0, false),
            Person {
                starts_talking: true
            }
        );
        // the dead say nothing
        assert_eq!(
            talk_action(&target(SETFIG, DEAD, true), false, 10, 0, false),
            Nothing
        );
        // the turtle gives the shell once
        let turtle = target(CARRIER, 0, false);
        assert_eq!(
            talk_action(&turtle, false, 10, CARRIER_TURTLE, false),
            TurtleGivesShell
        );
        assert_eq!(
            talk_action(&turtle, false, 10, CARRIER_TURTLE, true),
            TurtleHasShell
        );
        assert_eq!(
            talk_action(&turtle, false, 10, CARRIER_TURTLE + 1, false),
            Nothing
        );
        assert_eq!(
            talk_action(&target(ENEMY, 0, false), false, 10, 0, false),
            EnemyShouts(7)
        );
    }

    fn talker() -> Talker {
        Talker {
            kindness: 20,
            fatigue: 0,
            day_period: 3,
            region: 0,
            goal: 2,
            princess_freed: false,
        }
    }

    #[test]
    fn what_people_say() {
        let t = talker();
        assert_eq!(plain_speech(WIZARD, &t), Some(29));
        assert_eq!(
            plain_speech(
                WIZARD,
                &Talker {
                    kindness: KIND_ENOUGH - 1,
                    ..talker()
                }
            ),
            Some(35)
        );
        assert_eq!(plain_speech(GUARD_BACK, &t), Some(15));
        assert_eq!(plain_speech(PRINCESS, &t), None);
        assert_eq!(
            plain_speech(
                PRINCESS,
                &Talker {
                    princess_freed: true,
                    ..talker()
                }
            ),
            Some(16)
        );
        assert_eq!(
            plain_speech(
                KING,
                &Talker {
                    princess_freed: true,
                    ..talker()
                }
            ),
            Some(17)
        );
        assert_eq!(plain_speech(RANGER, &t), Some(55));
        assert_eq!(
            plain_speech(
                RANGER,
                &Talker {
                    region: RANGER_SWAMP_REGION,
                    ..talker()
                }
            ),
            Some(22)
        );
        assert_eq!(plain_speech(PRIEST, &t), None);
        assert_eq!(plain_speech(SORCERESS, &t), None);
    }

    #[test]
    fn the_bartender_notices_tiredness_and_the_hour() {
        let said = |fatigue, day_period| {
            plain_speech(
                BARTENDER,
                &Talker {
                    fatigue,
                    day_period,
                    ..talker()
                },
            )
        };
        assert_eq!(said(TIRED - 1, 0), Some(13));
        assert_eq!(said(TIRED, NIGHT_PERIOD), Some(14));
        assert_eq!(said(TIRED, NIGHT_PERIOD + 1), Some(12));
    }

    #[test]
    fn the_sorceress() {
        use SorceressAction::*;
        assert_eq!(sorceress_action(false, 0, || unreachable!()), AskForHelp);
        assert_eq!(sorceress_action(true, 10, || 11), GiveLuck);
        assert_eq!(sorceress_action(true, 10, || 10), Nothing);
    }
}
