//! Going through the doors between the outdoors and the buildings and caves (from the main loop in
//! csrc/fmain.c). Doors that need a key are in `key_doors`.

mod entering;

mod leaving;

use self::entering::enter_door;
use self::leaving::leave_door;
use crate::game::data::state::Game;
use crate::game::output::Output;

const DOORCOUNT: i32 = 86;
const CAVE: i8 = 18;
const FIRST_INSIDE_REGION: u16 = 8;

/// The hero is lined up with a door when these bits of the position are in range: a tile is 16
/// pixels wide and 32 high, so bit 4 of y says which half of the tile row.
const LOWER_HALF_BIT: u16 = 0x10;
const TILE_COLUMN_MASK: u16 = 15;

/// Snaps the hero's position to a tile (16 wide, 32 high).
const TILE_X_MASK: u16 = 0xfff0;
const TILE_Y_MASK: u16 = 0xffe0;
const TILE_WIDTH: i32 = 16;

/// Is the hero standing at a door? Outdoors the doors are in sorted order and found by binary
/// search; indoors they are searched in turn. If so, the hero is moved to the other side.
/// Riding disables doors.
pub fn check_door(game: &mut Game, out: &mut dyn Output) {
    if game.hero.riding != 0 {
        return;
    }

    let xtest = game.hero.x & TILE_X_MASK;

    let ytest = game.hero.y & TILE_Y_MASK;

    if game.world.region < FIRST_INSIDE_REGION {
        enter_door(game, out, xtest, ytest);
    } else {
        leave_door(game, out, xtest, ytest);
    }
}
