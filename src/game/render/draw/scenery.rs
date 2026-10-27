//! Hiding the part of a sprite that scenery (trees, walls, roofs) is in front of.

use super::*;

/// The mask buffer (`bmask_mem`) is cleared before each sprite: 96 lines of 64 words, plus 5.
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

/// Zeroes the scenery mask buffer, ready for the next sprite.
pub(super) unsafe fn clear_scenery_mask() {
    unsafe {
        OwnBlitter();
        WaitBlit();
        clear_memory(the_memory.bmask_mem, MASK_CLEAR_SIZE);
        DisownBlitter();
    }
}

/// True for sprites drawn in front of everything: carriers, the hero when riding the swan or in
/// certain places, arrows and some special races.
pub(super) unsafe fn drawn_in_front(an: *const Shape, i: i32, atype: i16, inum: i16) -> bool {
    unsafe {
        if atype == CARRIER
            || (i == 0
                && (the_hero.riding == 11
                    || the_world.extent_type == NO_MASK_XTYPE
                    || (the_hero.x > 22833
                        && the_hero.x < 26428
                        && the_hero.y > 26425
                        && the_hero.y < 26527)))
        {
            return true;
        }
        if atype == OBJECTS && inum > 99 && inum < 102 {
            return true;
        }
        (*an).race == RACE_NO_MASK_A || (*an).race == RACE_NO_MASK_B
    }
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
        0 => true,
        1 => xm == 0,
        2 => ystop > 35,
        3 => on_bridge,
        4 => xm == 0 || ystop > 35,
        5 => xm == 0 && ystop > 35,
        7 => ystop > 20,
        _ => false,
    };
    if skip {
        SceneryRule::Skip
    } else if k == 6 && ym != 0 {
        SceneryRule::UseFullMask // full if above
    } else {
        SceneryRule::UseTileMask
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
