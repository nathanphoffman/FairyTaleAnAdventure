using FaeryTale.Game.Data;

namespace FaeryTale.Game.Render;

// Hiding the part of a sprite that scenery (trees, walls, roofs) is in front of.
internal static class SceneryMasks
{
    /// What a terrain tile does to a sprite drawn behind it.
    internal enum SceneryRule
    {
        /// The sprite is drawn over the tile.
        Skip,

        /// The tile's own mask hides part of the sprite.
        UseTileMask,

        /// The whole tile hides the sprite.
        UseFullMask,
    }

    /// The mask buffer (the mask region) is cleared before each sprite: 96 lines of 64 words, plus 5.
    private const int MASK_CLEAR_SIZE = (96 << 6) + 5;

    /// Terrain mask offset of the tile that hides everything.
    internal const short TILE_FULL_MASK = 64 * 4;

    /// Fall animation: tiles at or below this offset do not hide the actor.
    internal const short FALL_TILE_LIMIT = 220 * 4;

    /// The bridge sector, where actors other than the raft walk in front of the bridge.
    internal const short BRIDGE_SECTOR = 48;

    // Races that are never hidden behind scenery.
    private const byte RACE_NO_MASK_A = 0x85;
    private const byte RACE_NO_MASK_B = 0x87;

    /// The extent type of the room where actors are drawn in front of everything.
    private const ushort NO_MASK_XTYPE = 84;

    /// The hero rides the swan when Riding is this.
    private const short RIDING_SWAN = 11;

    // The area (x and y bounds, exclusive) where the hero is always drawn in front: the bridge.
    private const ushort FRONT_AREA_X_MIN = 22833;
    private const ushort FRONT_AREA_X_MAX = 26428;
    private const ushort FRONT_AREA_Y_MIN = 26425;
    private const ushort FRONT_AREA_Y_MAX = 26527;

    // Objects with numbers from 100 to 101 (the two flying sparks) are drawn in front.
    private const short FIRST_FRONT_OBJECT = 100;
    private const short LAST_FRONT_OBJECT = 101;

    // Map tiles are 32 pixels high, and 6 tile numbers make up one minimap column.
    private const int TILE_ROW_SHIFT = 5;
    private const int MINIMAP_COLUMN_TILES = 6;
    private const int TERRAIN_ENTRY_BYTES = 4;
    private const byte TERRAIN_TYPE_MASK = 15;

    // Terrain masking types (the low nibble of terrain byte 1) and the heights, in pixels above
    // the tile bottom, that matter to them.
    private const short TYPE_ALWAYS_SKIP = 0;
    private const short TYPE_LEFT_COLUMN = 1;
    private const short TYPE_TALL = 2;
    private const short TYPE_BRIDGE = 3;
    private const short TYPE_TALL_OR_LEFT = 4;
    private const short TYPE_TALL_AND_LEFT = 5;
    private const short TYPE_FULL_ABOVE = 6;
    private const short TYPE_LOW = 7;
    private const short TALL_HEIGHT = 35;
    private const short LOW_HEIGHT = 20;

    private const int RAFT_SLOT = 1;

    /// Zeroes the scenery mask buffer, ready for the next sprite.
    internal static void ClearSceneryMask(Video video) => SpriteBlitter.ClearMemory(video, MASK_CLEAR_SIZE);

    /// True for sprites drawn in front of everything: carriers, the hero when riding the swan or
    /// in certain places, arrows and some special races.
    internal static bool DrawnInFront(World world, Hero hero, Shape actor, int slot, Sprite sprite)
    {
        bool heroInFront = slot == 0
            && (hero.Riding == RIDING_SWAN
                || world.ExtentType == NO_MASK_XTYPE
                || (hero.X > FRONT_AREA_X_MIN && hero.X < FRONT_AREA_X_MAX && hero.Y > FRONT_AREA_Y_MIN && hero.Y < FRONT_AREA_Y_MAX));

        if (sprite.Atype == SpriteImages.CARRIER || heroInFront)
        {
            return true;
        }

        if (sprite.Atype == SpriteImages.OBJECTS && sprite.Inum >= FIRST_FRONT_OBJECT && sprite.Inum <= LAST_FRONT_OBJECT)
        {
            return true;
        }

        return actor.Race == RACE_NO_MASK_A || actor.Race == RACE_NO_MASK_B;
    }

    /// How the tile of masking type k (from the terrain table) treats a sprite. xm and ym are the
    /// tile's column and row within the sprite, yStop is the sprite's feet height above the tile
    /// bottom, and onBridge is true when the sprite is on the bridge.
    internal static SceneryRule RuleFor(short k, int xm, int ym, short yStop, bool onBridge)
    {
        bool skip = k switch
        {
            TYPE_ALWAYS_SKIP => true,
            TYPE_LEFT_COLUMN => xm == 0,
            TYPE_TALL => yStop > TALL_HEIGHT,
            TYPE_BRIDGE => onBridge,
            TYPE_TALL_OR_LEFT => xm == 0 || yStop > TALL_HEIGHT,
            TYPE_TALL_AND_LEFT => xm == 0 && yStop > TALL_HEIGHT,
            TYPE_LOW => yStop > LOW_HEIGHT,
            _ => false,
        };

        if (skip)
        {
            return SceneryRule.Skip;
        }

        return k == TYPE_FULL_ABOVE && ym != 0 ? SceneryRule.UseFullMask : SceneryRule.UseTileMask;
    }

    /// Cuts out the parts of the sprite that scenery (trees, walls, roofs) should hide, one map
    /// tile at a time, by marking them in the mask buffer.
    internal static void MaskAgainstScenery(Data.Game game, Video video, Shape actor, int slot, Placed placed, BlitExtent extent)
    {
        // First tile row of the sprite: pixel row divided by the tile height via a shift.
        byte firstRow = (byte)(placed.Clip.YStart >> TILE_ROW_SHIFT);

        // Last tile row, then made relative to the first by subtracting it (wrapping like the original byte math).
        byte rows = unchecked((byte)((byte)(placed.Clip.YStop >> TILE_ROW_SHIFT) - firstRow));

        for (int xm = 0; xm < Math.Max((int)extent.BlitWide, 0); xm++)
        {
            for (int ym = 0; ym <= rows; ym++)
            {
                if (MaskOfTile(game, actor, slot, placed.Sprite.Ground, extent.Xbw, firstRow, xm, ym) is { } mask)
                {
                    SpriteBlitter.AddSceneryMask(video, xm, ym, extent.BlitWide, mask);
                }
            }
        }
    }

    // The mask byte to cut out of the sprite for the tile at column xm and row ym of the sprite,
    // or null when that tile does not hide it.
    private static int? MaskOfTile(Data.Game game, Shape actor, int slot, short ground, sbyte xbw, byte firstRow, int xm, int ym)
    {
        // Back from tile rows to pixels.
        short yStop = (short)(ground - (short)((ym + firstRow) << TILE_ROW_SHIFT));
        short cm = (short)((xm + xbw) * MINIMAP_COLUMN_TILES + ym + firstRow);

        cm = (short)(game.Map.Minimap[cm] * TERRAIN_ENTRY_BYTES);
        short k = (short)(game.Map.TerrainByte(cm + 1) & TERRAIN_TYPE_MASK);

        if (actor.State == ActorState.FALL)
        {
            if (cm <= FALL_TILE_LIMIT)
            {
                return null;
            }

            k = TYPE_BRIDGE;
        }

        bool onBridge = game.Hero.Sector == BRIDGE_SECTOR && slot != RAFT_SLOT;

        switch (RuleFor(k, xm, ym, yStop, onBridge))
        {
            case SceneryRule.Skip:
                return null;

            case SceneryRule.UseFullMask:
                cm = TILE_FULL_MASK;
                break;
        }

        return game.Map.TerrainByte(cm);
    }
}
