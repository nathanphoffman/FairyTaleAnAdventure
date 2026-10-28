//! Hiding the part of a sprite that scenery (trees, walls, roofs) is in front of.

use super::{CARRIER, FALL, OBJECTS, RAFT_SLOT};
use crate::game::data::state::ActorRef;
use crate::game::data::state::Game;
use crate::game::render::blit::{add_scenery_mask, clear_memory};

/// The mask buffer (the mask region) is cleared before each sprite: 96 lines of 64 words, plus 5.
const MASK_CLEAR_SIZE: i32 = (96 << 6) + 5;

/// Terrain flag bits `terrain_mem[tile*4+1] & 15` says how a tile hides what is behind it.
pub(super) const TILE_FULL_MASK: i16 = 64 * 4;

/// Fall animation: tiles at or below this offset do not hide the actor.
pub(super) const FALL_TILE_LIMIT: i16 = 220 * 4;

/// The bridge sector, where actors other than the raft walk in front of the bridge.
pub(super) const BRIDGE_SECTOR: i16 = 48;

/// Races that are never hidden behind scenery.
const RACE_NO_MASK_A: u8 = 0x85;
const RACE_NO_MASK_B: u8 = 0x87;

/// The `extent_type` of the room where actors are drawn in front of everything.
const NO_MASK_XTYPE: u16 = 84;

/// The hero rides the swan when `riding` is this.
const RIDING_SWAN: i16 = 11;

/// The area (x and y bounds, exclusive) where the hero is always drawn in front: the bridge.
const FRONT_AREA_X: (u16, u16) = (22833, 26428);
const FRONT_AREA_Y: (u16, u16) = (26425, 26527);

/// Objects with numbers from 100 to 101 (the two flying sparks) are drawn in front.
const FRONT_OBJECTS: std::ops::RangeInclusive<i16> = 100..=101;

/// Map tiles are 32 pixels high, and 6 tile numbers make up one minimap column.
const TILE_ROW_SHIFT: u32 = 5;
const MINIMAP_COLUMN_TILES: i32 = 6;
const TERRAIN_ENTRY_BYTES: i32 = 4;
const TERRAIN_TYPE_MASK: u8 = 15;

/// Terrain masking types (the low nibble of terrain byte 1) and the heights, in pixels above the
/// tile bottom, that matter to them.
const TYPE_ALWAYS_SKIP: i16 = 0;
const TYPE_LEFT_COLUMN: i16 = 1;
const TYPE_TALL: i16 = 2;
const TYPE_BRIDGE: i16 = 3;
const TYPE_TALL_OR_LEFT: i16 = 4;
const TYPE_TALL_AND_LEFT: i16 = 5;
const TYPE_FULL_ABOVE: i16 = 6;
const TYPE_LOW: i16 = 7;
const TALL_HEIGHT: i16 = 35;
const LOW_HEIGHT: i16 = 20;

/// Zeroes the scenery mask buffer, ready for the next sprite.
pub(super) fn clear_scenery_mask(game: &mut Game) {
    clear_memory(game, MASK_CLEAR_SIZE);
}

/// True for sprites drawn in front of everything: carriers, the hero when riding the swan or in
/// certain places, arrows and some special races.
pub(super) fn drawn_in_front(
    game: &mut Game,
    actor: ActorRef,
    i: i32,
    atype: i16,
    inum: i16,
) -> bool {
    if atype == CARRIER
        || (i == 0
            && (game.hero.riding == RIDING_SWAN
                || game.world.extent_type == NO_MASK_XTYPE
                || (game.hero.x > FRONT_AREA_X.0
                    && game.hero.x < FRONT_AREA_X.1
                    && game.hero.y > FRONT_AREA_Y.0
                    && game.hero.y < FRONT_AREA_Y.1)))
    {
        return true;
    }

    if atype == OBJECTS && FRONT_OBJECTS.contains(&inum) {
        return true;
    }

    game.actors.list[actor.slot()].race == RACE_NO_MASK_A
        || game.actors.list[actor.slot()].race == RACE_NO_MASK_B
}

/// What a terrain tile does to a sprite drawn behind it.
#[derive(Debug, PartialEq)]
pub(super) enum SceneryRule {
    /// The sprite is drawn over the tile.
    Skip,
    /// The tile's own mask hides part of the sprite.
    UseTileMask,
    /// The whole tile hides the sprite.
    UseFullMask,
}

/// How the tile of masking type `k` (from the terrain table) treats a sprite. `xm` and `ym` are
/// the tile's column and row within the sprite, `ystop` is the sprite's feet height above the
/// tile bottom, and `on_bridge` is true when the sprite is on the bridge.
pub(super) fn scenery_rule(k: i16, xm: i32, ym: i32, ystop: i16, on_bridge: bool) -> SceneryRule {
    let skip = match k {
        TYPE_ALWAYS_SKIP => true,
        TYPE_LEFT_COLUMN => xm == 0,
        TYPE_TALL => ystop > TALL_HEIGHT,
        TYPE_BRIDGE => on_bridge,
        TYPE_TALL_OR_LEFT => xm == 0 || ystop > TALL_HEIGHT,
        TYPE_TALL_AND_LEFT => xm == 0 && ystop > TALL_HEIGHT,
        TYPE_LOW => ystop > LOW_HEIGHT,
        _ => false,
    };

    if skip {
        SceneryRule::Skip
    } else if k == TYPE_FULL_ABOVE && ym != 0 {
        SceneryRule::UseFullMask // full if above
    } else {
        SceneryRule::UseTileMask
    }
}

/// The sprite whose tiles are being masked: which actor, where its feet are and which minimap
/// tiles it covers.
struct SpriteTiles {
    actor: ActorRef,
    i: i32,
    ground: i16,
    xbw: i8,
    ym1: u8,
}

/// Cuts out the parts of the sprite that scenery (trees, walls, roofs) should hide, one map tile at a
/// time, by marking them in the mask buffer.
#[allow(clippy::too_many_arguments)] // the tile loop of the original, kept as one call
pub(super) fn mask_against_scenery(
    game: &mut Game,
    actor: ActorRef,
    i: i32,
    ground: i16,
    xbw: i8,
    ystart1: i16,
    ystop1: i16,
    blitwide: i8,
) {
    // Shifts by TILE_ROW_SHIFT, converting between pixels, tiles, sectors or bytes.
    let ym1 = (ystart1 >> TILE_ROW_SHIFT) as u8;

    // Shifts by TILE_ROW_SHIFT, converting between pixels, tiles, sectors or bytes.
    let ym2 = ((ystop1 >> TILE_ROW_SHIFT) as u8).wrapping_sub(ym1);
    let tiles = SpriteTiles {
        actor,
        i,
        ground,
        xbw,
        ym1,
    };

    for xm in 0..blitwide.max(0) as i32 {
        for ym in 0..=ym2 as i32 {
            if let Some(mask) = tiles.mask_of_tile(game, xm, ym) {
                add_scenery_mask(game, xm, ym, blitwide as i32, mask);
            }
        }
    }
}

impl SpriteTiles {
    /// The mask byte to cut out of the sprite for the tile at column `xm` and row `ym` of the
    /// sprite, or `None` when that tile does not hide it.
    fn mask_of_tile(&self, game: &mut Game, xm: i32, ym: i32) -> Option<i32> {
        // Back from tile rows to pixels.
        let ystop = self.ground - (((ym + self.ym1 as i32) << TILE_ROW_SHIFT) as i16);
        let mut cm = ((xm + self.xbw as i32) * MINIMAP_COLUMN_TILES + ym + self.ym1 as i32) as i16;
        cm = (game.memory.minimap[cm as usize] as i32 * TERRAIN_ENTRY_BYTES) as i16;
        let mut k =
            // Masks with TERRAIN_TYPE_MASK, keeping only the bits that matter here.
            (game.memory.terrain_byte(cm as usize + 1) & TERRAIN_TYPE_MASK) as i16;

        if game.actors.list[self.actor.slot()].state == FALL {
            if cm <= FALL_TILE_LIMIT {
                return None;
            }

            k = TYPE_BRIDGE;
        }

        let on_bridge = game.hero.sector == BRIDGE_SECTOR && self.i != RAFT_SLOT;

        match scenery_rule(k, xm, ym, ystop, on_bridge) {
            SceneryRule::Skip => return None,
            SceneryRule::UseFullMask => cm = TILE_FULL_MASK,
            SceneryRule::UseTileMask => {}
        }

        Some(game.memory.terrain_byte(cm as usize) as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scenery_rules_by_tile_type() {
        use SceneryRule::*;
        assert_eq!(scenery_rule(0, 1, 0, 0, false), Skip);
        assert_eq!(scenery_rule(1, 0, 0, 0, false), Skip);
        assert_eq!(scenery_rule(1, 1, 0, 0, false), UseTileMask);
        assert_eq!(scenery_rule(2, 0, 0, 36, false), Skip);
        assert_eq!(scenery_rule(2, 0, 0, 35, false), UseTileMask);
        assert_eq!(scenery_rule(3, 0, 0, 0, true), Skip);
        assert_eq!(scenery_rule(3, 0, 0, 0, false), UseTileMask);
        assert_eq!(scenery_rule(4, 0, 0, 0, false), Skip);
        assert_eq!(scenery_rule(5, 0, 0, 36, false), Skip);
        assert_eq!(scenery_rule(5, 0, 0, 10, false), UseTileMask);
        assert_eq!(scenery_rule(6, 0, 0, 0, false), UseTileMask);
        assert_eq!(scenery_rule(6, 0, 1, 0, false), UseFullMask);
        assert_eq!(scenery_rule(7, 0, 0, 21, false), Skip);
        assert_eq!(scenery_rule(8, 0, 0, 99, false), UseTileMask);
    }
}
