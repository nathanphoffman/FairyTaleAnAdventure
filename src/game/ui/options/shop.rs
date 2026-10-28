//! The Buy menu: what the bartender sells and whether the hero can pay.

use super::{ARROWS, PRQ_VITALITY};
use crate::game::actors::interaction::eat_food;
use crate::game::data::object_tables::jtrans;
use crate::game::data::state::Game;
use crate::game::ui::items_menu::{add_stuff, item_name};
use crate::game::ui::messages::{print, print_cont, print_wrapped};
use crate::game::ui::text::{queue_print, show_event};

// Buy menu: the bartender is race 0x88; entries 5-11 pick a pair in `jtrans`.
const BARTENDER_RACE: u8 = 0x88;
const LAST_BUY_ENTRY: i16 = 11;
pub(super) const FIRST_MENU_ENTRY: i16 = 5;
const FOOD: usize = 0;
const BOUGHT_FOOD_EVENT: i32 = 22;
const BOUGHT_ARROWS_EVENT: i32 = 23;
const FOOD_HUNGER: i32 = 50;
const ARROWS_BOUGHT: u8 = 10;

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
pub(super) fn buy_option(game: &mut Game, chosen_entry: i16) -> bool {
    game.actors.nearest_object = game.actors.nearest_person;
    let near = game.actors.nearest_object;
    let race = if near == 0 {
        0
    } else {
        game.actors.list[near as usize].race
    };

    let (item, price) = match buy_gate(near, race, chosen_entry, &jtrans) {
        BuyGate::Nothing => return true,
        BuyGate::NotHandled => return false,
        BuyGate::Offer { item, price } => (item, price),
    };

    match purchase(game.hero.wealth, price, item) {
        Purchase::TooPoor => print(game, "Not enough money!"),
        Purchase::Bought { wealth_left, what } => {
            game.hero.wealth = wealth_left;
            queue_print(&mut game.ui, PRQ_VITALITY);
            receive_purchase(game, item, what);
        }
    }

    true
}

/// Hands over what was bought: food is eaten at once, arrows come in a bundle, anything else is
/// added to the pack with a message.
fn receive_purchase(game: &mut Game, item: usize, what: Bought) {
    match what {
        Bought::Food => {
            show_event(game, BOUGHT_FOOD_EVENT);
            eat_food(game, FOOD_HUNGER);
        }

        Bought::Arrows => {
            add_stuff(game, item, ARROWS_BOUGHT);
            show_event(game, BOUGHT_ARROWS_EVENT);
        }

        Bought::Item => {
            add_stuff(game, item, 1);
            print_wrapped(game, "% bought a ");
            print_cont(game, item_name(item));
            print_cont(game, ".");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::ARROWS;
    use super::*;

    const SHOP: [i8; 14] = [0, 3, 8, 10, 11, 15, 1, 30, 2, 45, 3, 75, 13, 20];

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
}
