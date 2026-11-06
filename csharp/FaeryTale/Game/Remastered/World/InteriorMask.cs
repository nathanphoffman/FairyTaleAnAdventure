using FaeryTale.Game.Data;
using FaeryTale.Game.Land;
using FaeryTale.Game.Land.Map;

namespace FaeryTale.Game.Remastered;

/// Indoors, the interiors of many buildings lie side by side in the map, with black between them.
/// Zoomed far out the world drawer would show the next buildings as well, so this works out which
/// tiles belong to the building the hero is in (all the tiles joined to the hero's one that are not
/// black), and the drawer leaves the rest black. The set is worked out again when the hero is in a
/// building that was not the last one.
public sealed class InteriorMask
{
    // The black outside the rooms is tile 0.
    private const int BLACK_TILE = 0;

    // Tiles are 16 pixels wide and 32 high.
    private const int TILE_COLUMN_SHIFT = 4;
    private const int TILE_ROW_SHIFT = 5;

    // Never spreads further than this from the hero's tile (a cave can be large, a building is not),
    // so a map with no black round it cannot make the search run on.
    private const int MAX_REACH_COLUMNS = 320;
    private const int MAX_REACH_ROWS = 160;

    // A tile of the world as one number, to keep them in a set.
    private const int ROW_KEY_SHIFT = 16;

    private static readonly (int Column, int Row)[] STEPS = [(1, 0), (-1, 0), (0, 1), (0, -1)];

    private HashSet<int>? shown;

    /// Works out which tiles to show for the hero's building (before the tiles are drawn); outdoors
    /// there is no mask.
    public void Prepare(Data.Game game)
    {
        if (game.World.Region < Regions.FIRST_INDOOR_REGION)
        {
            shown = null;

            return;
        }

        Shape hero = game.Actors.List[0];
        (int column, int row) = (hero.AbsX >> TILE_COLUMN_SHIFT, hero.AbsY >> TILE_ROW_SHIFT);

        if (shown is null || !shown.Contains(Key(column, row)))
        {
            shown = Spread(game, column, row);
        }
    }

    /// Whether a tile of the world is left black (it is in another building). False when there is
    /// no mask.
    public bool Hides(int column, int row) => shown is not null && !shown.Contains(Key(column, row));

    /// Whether a world position is left black.
    public bool HidesPosition(int worldX, int worldY) => Hides(worldX >> TILE_COLUMN_SHIFT, worldY >> TILE_ROW_SHIFT);

    // The tiles joined to a start tile without crossing black. A start that is itself black means
    // there is nothing to go by, and every tile is shown.
    private static HashSet<int>? Spread(Data.Game game, int startColumn, int startRow)
    {
        if (TileAt(game, startColumn, startRow) == BLACK_TILE)
        {
            return null;
        }

        HashSet<int> found = [Key(startColumn, startRow)];
        Queue<(int Column, int Row)> waiting = [];

        waiting.Enqueue((startColumn, startRow));

        while (waiting.TryDequeue(out (int Column, int Row) tile))
        {
            foreach ((int stepColumn, int stepRow) in STEPS)
            {
                (int column, int row) = (tile.Column + stepColumn, tile.Row + stepRow);

                if (IsNewLand(game, found, (column, row), (startColumn, startRow)))
                {
                    found.Add(Key(column, row));
                    waiting.Enqueue((column, row));
                }
            }
        }

        return found;
    }

    // Whether a tile is inside reach, not black and not found yet.
    private static bool IsNewLand(Data.Game game, HashSet<int> found, (int Column, int Row) tile, (int Column, int Row) start) =>
        Math.Abs(tile.Column - start.Column) <= MAX_REACH_COLUMNS
        && Math.Abs(tile.Row - start.Row) <= MAX_REACH_ROWS
        && tile.Column >= 0 && tile.Row >= 0
        && !found.Contains(Key(tile.Column, tile.Row))
        && TileAt(game, tile.Column, tile.Row) != BLACK_TILE;

    private static int TileAt(Data.Game game, int column, int row) =>
        MapLookup.TileNumber(game, (ushort)column, (ushort)row);

    private static int Key(int column, int row) => (row << ROW_KEY_SHIFT) | column;
}
