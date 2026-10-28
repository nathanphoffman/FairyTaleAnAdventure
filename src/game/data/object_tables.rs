//! The tables for what lies where and how things are drawn (from the end of csrc/fmain2.c): item
//! ids, the objects in each region, colours, the witch's beam points, bow offsets and loot
//! probabilities. (The tables describing kinds of things are in `tables`.) Names and element types match the C originals, because the C
//! that is left reads many of them.
//!
//! The names stay as in the C source for now (the C that is left refers to them); they are
//! renamed in the cleanup pass after the port.

// The names stay as in the C source for now (the C that is left refers to them); they are
// renamed in the cleanup pass after the port.
#![allow(non_upper_case_globals, dead_code)]

use crate::game::data::types::WorldObject;

// ---- item ids (the C `enum obytes`) ----
pub const QUIVER: u8 = 11;
pub const MONEY: u8 = 13;
pub const URN: u8 = 14;
pub const CHEST: u8 = 15;
pub const SACKS: u8 = 16;
pub const G_RING: u8 = 17;
pub const B_STONE: u8 = 18;
pub const G_JEWEL: u8 = 19;
pub const SCRAP: u8 = 20;
pub const C_ORB: u8 = 21;
pub const VIAL: u8 = 22;
pub const B_TOTEM: u8 = 23;
pub const J_SKULL: u8 = 24;
pub const GOLD_KEY: u8 = 25;
pub const GREY_KEY: u8 = 26;
pub const FOOTSTOOL: u8 = 31;
pub const TURTLE: u8 = 102;
pub const BLUE_KEY: u8 = 114;
pub const M_WAND: u8 = 145;
pub const MEAL: u8 = 146;
pub const ROSE: u8 = 147;
pub const FRUIT: u8 = 148;
pub const STATUE: u8 = 149;
pub const BOOK: u8 = 150;
pub const SHELL: u8 = 151;
pub const GREEN_KEY: u8 = 153;
pub const WHITE_KEY: u8 = 154;
pub const RED_KEY: u8 = 242;

/// What the bartender sells: pairs of (inventory item number, price). Item 0 is food (eaten
/// at once), item 8 is a bundle of 10 arrows, the others are single items. The buy menu choice
/// (from 5) picks the pair.
pub static jtrans: [i8; 14] = [
    0,3, 8,10, 11,15, 1,30, 2,45, 3,75, 13,20
];

/// What a dead enemy drops: 8 entries per treasure class (the enemy's class times 8, plus a
/// random 0-7 picks the entry). The entry is an inventory item number (gold amounts for the high
/// numbers) or 0 for nothing.
pub static treasure_probs: [i8; 40] = [
    0, 0, 0, 0, 0, 0, 0, 0, // no treasure
    9,11,13,31,31,17,17,32, // stone,vial,totem,gold,keys
    12,14,20,20,20,31,33,31, // keys,skull,gold,nothing
    10,10,16,16,11,17,18,19, // magic and keys
    15,21,0,0,0,0,0,0 // jade skull and white key
];

/// The weapon an enemy carries: 4 entries per weapon class (the enemy type's `arms` times 4, plus
/// a random 0-3). 1 dirk, 2 mace, 3 sword, 4 bow, 5 magic wand, 8 touch attack, 0 none.
pub static weapon_probs: [i8; 32] = [
    0,0,0,0, // no weapons
    1,1,1,1, // dirks only
    1,2,1,2, // dirks and maces
    1,2,3,2, // mostly maces
    4,4,3,2, // swords and bows
    5,5,5,5, // magic wand
    8,8,8,8, // touch attack!
    3,3,3,3, // swords only
];

/// Sprite frames for the falling animation: 6 per brother, picked by how far into the fall the actor is.
pub static fallstates: [u8; 24] = [
    0,0,0,0,0,0,
    0x20,0x22,0x3a,0x6f,0x70,0x71,
    0x24,0x27,0x3c,0x6f,0x70,0x71,
    0x37,0x38,0x3d,0x6f,0x70,0x71
];

/// Where the bow is drawn relative to the shooter, x, for each of the 32 shooting animation frames.
pub static bow_x: [i8; 32] = [
    1,2,3,4,3,2,1,0, 3,2,0,-2,-3,-2,0,2,
    -3,-3,-3,-3,-3,-3,-3,-2, 0,1,1,1,0,-2,-3,-2
];

/// Where the bow is drawn relative to the shooter, y, for each of the 32 shooting animation frames.
pub static bow_y: [i8; 32] = [
    8,8,8,7,8,8,8,8, 11,12,13,13,13,13,13,12,
    8,7,6,5,6,7,8,9, 12,12,12,12,12,12,11,12
];

/// Where an arrow starts relative to the shooter, x, for each of the 8 facings.
pub static bowshotx: [i8; 8] = [
    0, 0, 3, 6,-3,-3,-3,-6
];

/// Where an arrow starts relative to the shooter, y, for each of the 8 facings.
pub static bowshoty: [i8; 8] = [
    -6,-6,-1, 0, 6, 8, 0,-1
];

/// Where a fireball (non-bow missile) starts relative to the shooter, y, for each of the 8 facings.
pub static gunshoty: [i8; 8] = [
    2, 0, 4, 7, 9, 4, 7, 8
];

/// The witch beam: 64 directions of 4 numbers (x1, y1, x2, y2), the near and far end of each edge,
/// going around a circle of radius 100. See `witch_fx`.
pub static witchpoints: [i8; 256] = [
    // 64 points - 256 entries
    0,100,0,10, 9,99,0,9, 19,98,1,9, 29,95,2,9,
    38,92,3,9, 47,88,4,8, 55,83,5,8, 63,77,6,7,
    70,70,7,7, 77,63,7,6, 83,55,8,5, 88,47,8,4,
    92,38,9,3, 95,29,9,2, 98,19,9,1, 99,9,9,0,
    100,0,10,0, 99,-10,9,-1, 98,-20,9,-2, 95,-30,9,-3,
    92,-39,9,-4, 88,-48,8,-5, 83,-56,8,-6, 77,-64,7,-7,
    70,-71,7,-8, 63,-78,6,-8, 55,-84,5,-9, 47,-89,4,-9,
    38,-93,3,-10, 29,-96,2,-10, 19,-99,1,-10, 9,-100,0,-10,
    0,-100,0,-10, -10,-100,-1,-10,-20,-99,-2,-10, -30,-96,-3,-10,
    -39,-93,-4,-10, -48,-89,-5,-9, -56,-84,-6,-9, -64,-78,-7,-8,
    -71,-71,-8,-8, -78,-64,-8,-7, -84,-56,-9,-6, -89,-48,-9,-5,
    -93,-39,-10,-4, -96,-30,-10,-3, -99,-20,-10,-2, -100,-10,-10,-1,
    -100,-1,-10,-1, -100,9,-10,0, -99,19,-10,1, -96,29,-10,2,
    -93,38,-10,3, -89,47,-9,4, -84,55,-9,5, -78,63,-8,6,
    -71,70,-8,7, -64,77,-7,7, -56,83,-6,8, -48,88,-5,8,
    -39,92,-4,9, -30,95,-3,9, -20,98,-2,9, -10,99,-1,9,
];

/// Colours of the sky as the sun rises in the win sequence (53 steps from night to day).
pub static sun_colors: [i16; 53] = [
    0x000, 0x000, 0x001, 0x002, 0x002, 0x012, 0x112, 0x113,
    0x113, 0x214, 0x224, 0x225, 0x326, 0x326, 0x337, 0x338,
    0x438, 0x439, 0x449, 0x54A, 0x54B, 0x54B, 0x55C, 0x65C,
    0x65D, 0x66E, 0x76E, 0x76F, 0x86F, 0x97E, 0xA7E, 0xB7E,
    0xB8D, 0xC8D, 0xD8D, 0xD8D, 0xE9C, 0xE9C, 0xE9B, 0xE9B,
    0xFAA, 0xFAA, 0xF99, 0xF98, 0xF98, 0xF97, 0xF86, 0xF85,
    0xF84, 0xF84, 0xF93, 0xF92, 0x76F
];

/// The 32 page colours at full brightness (colour 31 changes with the region).
/// Each is 12 bits: red, green and blue in 4 bits each.
pub static mut page_colors: [u16; 32] = [
    0x0000,0x0FFF,0x0E96,0x0B63,0x0631,0x07BF,0x0333,0x0DB8,
    0x0223,0x0445,0x0889,0x0BBC,0x0521,0x0941,0x0F82,0x0FC7,
    0x0040,0x0070,0x00B0,0x06F6,0x0005,0x0009,0x000D,0x037F,
    0x0C00,0x0F50,0x0FA0,0x0FF6,0x0EB6,0x0EA5,0x000F,0x0BDF
];

/// Item translation: pairs of (object id in the world, inventory slot). When the hero picks up
/// an object, the pair with its id says which inventory count to raise. Ended by 0, 0.
pub static itrans: [u8; 64] = [
    QUIVER, 35,
    B_STONE, 9,
    G_JEWEL, 10,
    VIAL, 11,
    C_ORB, 12,
    B_TOTEM, 13,
    G_RING, 14,
    J_SKULL, 15,
    M_WAND, 4,
    27, 5,
    8, 2,
    9, 1,
    12, 0,
    10, 3,
    ROSE, 23,
    FRUIT, 24,
    STATUE, 25,
    BOOK, 26,
    SHELL, 6,
    155, 7,
    136, 27,
    137, 28,
    138, 29,
    139, 22,
    140, 30,
    GOLD_KEY, 16,
    GREEN_KEY, 17,
    BLUE_KEY, 18,
    RED_KEY, 19,
    GREY_KEY, 20,
    WHITE_KEY, 21,
    0, 0,
];

/// Random treasure scattered over a region on first visit, picked with 4 random bits.
pub static rand_treasure: [u8; 16] = [
    SACKS, SACKS, SACKS, SACKS, CHEST, MONEY, GOLD_KEY, QUIVER, GREY_KEY, GREY_KEY, GREY_KEY, RED_KEY, B_TOTEM, VIAL, WHITE_KEY, CHEST,
];

/// One lit part of the compass: where it is in the bitmap and how big (`struct compvars`).
#[repr(C)]
pub struct CompassPart {
    pub xrect: u8,
    pub yrect: u8,
    pub xsize: u8,
    pub ysize: u8,
}

const fn part(xrect: u8, yrect: u8, xsize: u8, ysize: u8) -> CompassPart {
    CompassPart { xrect, yrect, xsize, ysize }
}

/// The compass parts for the 8 directions; 8 and 9 mean "not walking" and light nothing.
pub static comptable: [CompassPart; 10] = [
    part(0, 0, 16, 8),
    part(16, 0, 16, 9),
    part(32, 0, 16, 8),
    part(30, 8, 18, 8),
    part(32, 16, 16, 8),
    part(16, 13, 16, 11),
    part(0, 16, 16, 8),
    part(0, 8, 18, 8),
    part(0, 0, 1, 1),
    part(0, 0, 1, 1),
];

/// Builds an object list entry: position, item id and state.
const fn obj(x: u16, y: u16, id: u8, stat: i8) -> WorldObject {
    WorldObject { xc: x, yc: y, ob_id: id as i8, ob_stat: stat }
}

/// An unused entry, kept so a region has room for the random treasure added when it is visited.
const BLANK: WorldObject = obj(0, 0, 0, 0);

// Object states (`ob_stat`): 0 does not exist, 1 on the ground, 2 in the inventory, 3 a person,
// 4 a dead person, 5 hidden until looked for, 6 cabinet items.

/// Global objects.
pub static mut ob_listg: [WorldObject; 11] = [
   // global objects
    obj(0, 0, 0, 0), // special item (for 'give')
    obj(0, 0, 28, 0), // dead brother 1 - to be filled in later
    obj(0, 0, 28, 0), // dead brother 2 - to be filled in later
    obj(19316, 15747, 11, 0), // ghost brother 1 - to be filled in later
    obj(18196, 15735, 11, 0), // ghost brother 2 - to be filled in later
    obj(12439, 36202, 10, 3), // spectre - can be disabled
    obj(11092, 38526, STATUE, 1), // gold statues (6 = seahold)
    obj(25737, 10662, STATUE, 1), // (7 = ogre den)
    obj(2910, 39023, STATUE, 1), // (8 = octal room)
    obj(12025, 37639, STATUE, 0), // (9 = sorceress)
    obj(6700, 33766, STATUE, 0), // (10 = priest)
];

/// Snow land objects.
pub static mut ob_list0: [WorldObject; 13] = [
   // snow land objects
    obj(3340, 6735, 12, 3), // ranger west
    obj(9678, 7035, 12, 3), // ranger east
    obj(4981, 6306, 12, 3), // ranger north
    BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK,
];

/// Maze forest (north) objects.
pub static mut ob_list1: [WorldObject; 11] = [
   // maze forest (north) objects
    // {23297,5797,TURTLE,1},
    obj(23087, 5667, TURTLE, 1),
    BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK,
];

/// Swamp land objects.
pub static mut ob_list2: [WorldObject; 15] = [
   // swamp land objects
    obj(13668, 15000, 0, 3), // wizard
    obj(10627, 13154, 0, 3), // wizard
    obj(4981, 10056, 12, 3), // ranger
    obj(13950, 11087, SACKS, 1),
    obj(10344, 36171, SHELL, 1), // shell
    BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK,
];

/// Maze forest (south) and manor objects (and tambry).
pub static mut ob_list3: [WorldObject; 22] = [
   // maze forest (south) and manor objects (and tambry)
    obj(19298, 16128, CHEST, 1),
    obj(18310, 15969, 13, 3), // beggar
    obj(20033, 14401, 0, 3), // wizard
    obj(24794, 13102, 13, 3), // beggar
    obj(21626, 15446, B_STONE, 1), // 3 items at stone ring

    obj(21616, 15456, MONEY, 1),
    obj(21636, 15456, G_RING, 1),
    obj(20117, 14222, G_JEWEL, 1),
    obj(24185, 9840, SACKS, 1),
    obj(25769, 10617, MONEY, 1),

    obj(25678, 10703, B_STONE, 1),
    obj(17177, 10599, SCRAP, 1),
    BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK,
];

/// Desert objects.
pub static mut ob_list4: [WorldObject; 13] = [
   // desert objects
    obj(0, 0, 0, 0), // dummy
    obj(0, 0, 0, 0), // dummy
    obj(6817, 19693, 13, 3), // beggar - must be 3rd object
    BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK,
];

/// Farm and city objects.
pub static mut ob_list5: [WorldObject; 15] = [
   // farm and city objects
    obj(22184, 21156, 13, 3), // beggar
    obj(18734, 17595, G_RING, 1),
    obj(21294, 22648, CHEST, 1),
    obj(22956, 19955, 0, 3), // wizard
    obj(28342, 22613, 0, 3), // wizard
    BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK,
];

/// Lava plain objects.
pub static mut ob_list6: [WorldObject; 11] = [
   // lava plain objects
    obj(24794, 13102, 13, 3), // DUMMY OBJECT
    BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK,
];

/// Southern mountain land objects.
pub static mut ob_list7: [WorldObject; 11] = [
   // southern mountain land objects
    obj(23297, 5797, TURTLE, 1), // dummy object
    BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK, BLANK,
];

/// Interiors of buildings.
pub static mut ob_list8: [WorldObject; 78] = [
   // interiors of buildings
    obj(6700, 33756, 1, 3), // priest in chapel
    obj(5491, 33780, 5, 3), // king on throne - never mask
    obj(5592, 33764, 6, 3), // noble
    obj(5514, 33668, 2, 3), // guard
    obj(5574, 33668, 2, 3), // guard
    obj(8878, 38995, 0, 3), // wizard
    obj(7776, 34084, 0, 3), // wizard
    obj(5514, 33881, 3, 3), // guard
    obj(5574, 33881, 3, 3), // guard
    obj(10853, 35656, 4, 3), // princess
    obj(12037, 37614, 7, 3), // sorceress

    obj(11013, 36804, 9, 3), // witch
    obj(9631, 38953, 8, 3), // bartender
    obj(10191, 38953, 8, 3), // bartender
    obj(10649, 38953, 8, 3), // bartender
    obj(2966, 33964, 8, 3), // bartender
    obj(9532, 40002, FOOTSTOOL, 1),
    obj(6747, 33751, FOOTSTOOL, 1),
    obj(11410, 36169, 27+128, 1), // sunstone
    obj(9550, 39964, B_TOTEM, 1), // cabinet item
    obj(9552, 39964, B_TOTEM, 1),

    obj(9682, 39964, B_TOTEM, 1), // cabinet item
    obj(9684, 39964, B_TOTEM, 1),
    obj(9532, 40119, B_TOTEM, 1), // on table
    obj(9575, 39459, URN, 1),
    obj(9590, 39459, URN, 1),
    obj(9605, 39459, URN, 1),
    obj(9680, 39453, VIAL, 1),
    obj(9682, 39453, VIAL, 1),
    obj(9784, 39453, VIAL, 1),
    obj(9668, 39554, CHEST, 1),

    obj(11090, 39462, MONEY, 1),
    obj(11108, 39458, B_TOTEM, 1),
    obj(11118, 39459, B_TOTEM, 1),
    obj(11128, 39459, B_TOTEM, 1),
    obj(11138, 39458, B_TOTEM, 1),
    obj(11148, 39459, B_TOTEM, 1),
    obj(11158, 39459, B_TOTEM, 1),
    obj(11855, 36206, FOOTSTOOL, 1),
    obj(11909, 36198, CHEST, 1),
    obj(11918, 36246, B_TOTEM, 1), // cabinet items?

    obj(11928, 36246, B_TOTEM, 1),
    obj(11938, 36246, B_TOTEM, 1),
    obj(12212, 38481, CHEST, 1),
    obj(11652, 38481, RED_KEY, 1),
    obj(10427, 39977, FOOTSTOOL, 1),
    obj(10323, 40071, URN, 1),
    obj(10059, 38472, SACKS, 1),
    obj(10344, 36171, SHELL, 1), // shell
    obj(11936, 36207, SCRAP, 1), // spectre note
    obj(9674, 35687, URN, 1),
    obj(5473, 38699, ROSE, 1),
    obj(7185, 34342, FRUIT, 1),
    obj(7190, 34342, FRUIT, 1),
    obj(7195, 34342, FRUIT, 1),
    obj(7185, 34347, FRUIT, 1),
    obj(7190, 34347, FRUIT, 1),
    obj(7195, 34347, FRUIT, 1),
    obj(6593, 34085, FRUIT, 1),
    obj(6598, 34085, FRUIT, 1),
    obj(6593, 34090, FRUIT, 1),
    obj(6598, 34090, FRUIT, 1),

   // 'look' items
    obj(3872, 33546, GOLD_KEY, 5),
    obj(3887, 33510, B_TOTEM, 5),
    obj(4495, 33510, VIAL, 5),
    obj(3327, 33383, J_SKULL, 5),
    obj(4221, 34119, QUIVER, 5),
    obj(7610, 33604, VIAL, 5),
    obj(7616, 33522, MONEY, 5),
    obj(9570, 35768, B_STONE, 5),
    obj(9668, 35769, QUIVER, 5),
    obj(9553, 38951, G_RING, 5),
    obj(10062, 39005, J_SKULL, 5),
    obj(10577, 38951, VIAL, 5),
    obj(11062, 39514, MONEY, 5),
    obj(8845, 39494, WHITE_KEY, 5),
    obj(6542, 39494, G_JEWEL, 5),
    obj(7313, 38992, RED_KEY, 5),
];

/// Underground areas.
pub static mut ob_list9: [WorldObject; 9] = [
   // underground areas
    obj(7540, 38528, M_WAND, 1),
    obj(9624, 36559, M_WAND, 1),
    obj(9624, 37459, M_WAND, 1),
    obj(8337, 36719, M_WAND, 1),
    obj(8154, 34890, CHEST, 1),
    obj(7826, 35741, CHEST, 1),
    obj(3460, 37260, 0, 3), // wizard
    obj(8485, 35725, MONEY, 1),
    obj(3723, 39340, 128+10, 1), // king's bone
];

/// The object list of region `region` (the global list is separate). The lists have room to grow:
/// random treasure is added after the objects in use.
pub fn region_objects(region: usize) -> &'static mut [WorldObject] {
    unsafe {
        match region {
            0 => &mut *(&raw mut ob_list0),
            1 => &mut *(&raw mut ob_list1),
            2 => &mut *(&raw mut ob_list2),
            3 => &mut *(&raw mut ob_list3),
            4 => &mut *(&raw mut ob_list4),
            5 => &mut *(&raw mut ob_list5),
            6 => &mut *(&raw mut ob_list6),
            7 => &mut *(&raw mut ob_list7),
            8 => &mut *(&raw mut ob_list8),
            9 => &mut *(&raw mut ob_list9),
            _ => &mut [],
        }
    }
}

/// The global object list.
pub fn global_objects() -> &'static mut [WorldObject] {
    unsafe { &mut *(&raw mut ob_listg) }
}

/// How many entries of each region's list are in use.
pub static mut mapobs: [i16; 10] = [3, 1, 5, 12, 3, 5, 1, 1, 61 + 16, 9];

/// Whether random treasure has been scattered over each region yet (1 = done).
pub static mut dstobs: [i16; 10] = [0, 0, 0, 0, 0, 0, 0, 0, 1, 1];

/// Entries in the global object list.
pub static glbobs: i16 = 11;

// ---- shared work areas ----

/// The 32 colours being faded to (scratch space for `fade_page`).
pub static mut fader: [u16; 32] = [0; 32];

/// The print queue: requests for the text panel, filled by `queue_print` and emptied by `handle_print_request`.
pub static mut print_que: [i8; 32] = [0; 32];
pub static mut prec: i16 = 0;
pub static mut pplay: i16 = 0;

/// The tile numbers of the visible map (19 columns of 6), plus spare room.
pub static mut minimap: [i16; 120] = [0; 120];

/// The typed answer and where it is shown (the copy protection riddle, now skipped).
pub static mut answr: [i8; 10] = [0; 10];
pub static mut xx: i16 = 0;
pub static mut yy: i16 = 0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_region_has_room_for_its_used_objects() {
        let capacity = [13, 11, 15, 22, 13, 15, 11, 11, 78, 9];
        unsafe {
            for (region, room) in capacity.iter().enumerate() {
                assert!(mapobs[region] <= *room, "region {region}");
            }
        }
    }

    #[test]
    fn regions_with_random_treasure_have_ten_spare_entries() {
        // regions not yet distributed (dstobs 0) get 10 new objects when first visited
        let capacity = [13, 11, 15, 22, 13, 15, 11, 11];
        unsafe {
            for (region, room) in capacity.iter().enumerate() {
                assert!(mapobs[region] + 10 <= *room, "region {region}");
            }
        }
    }

    #[test]
    fn item_translation_table_ends_with_a_zero_pair() {
        assert_eq!(&itrans[itrans.len() - 2..], &[0, 0]);
        assert_eq!(itrans.len() % 2, 0);
    }

    #[test]
    fn witch_beam_has_64_directions_of_4_numbers() {
        assert_eq!(witchpoints.len(), 64 * 4);
    }

    #[test]
    fn loot_tables_have_the_expected_shape() {
        assert_eq!(treasure_probs.len(), 5 * 8);
        assert_eq!(weapon_probs.len(), 8 * 4);
        assert_eq!(rand_treasure.len(), 16);
    }
}
