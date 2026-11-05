namespace FaeryTale.Game.Remastered;

/// Finds a way across the map for click to move (A* search on a grid of cells).
///
/// The grid is laid out from the hero: cell (0, 0) is where the hero stands and each cell is CELL
/// pixels. The caller says what each cell costs, so the rules of the game (what blocks, what is
/// water) stay in the game's own code.
public static class PathFinder
{
    /// Size of a grid cell in world pixels.
    public const int CELL = 8;

    /// How far from the hero the search may go, in cells.
    private const int RADIUS = 40;

    /// Gives up (and uses the best cell found) after looking at this many cells.
    private const int MAX_EXPANSIONS = 4000;

    // Costs are whole numbers: a straight step is 10 and a diagonal one 14 (about 10 times root 2).
    private const int STRAIGHT_COST = 10;
    private const int DIAGONAL_COST = 14;

    private static readonly (int X, int Y)[] NEIGHBOURS = [(0, -1), (1, 0), (0, 1), (-1, 0), (-1, -1), (1, -1), (1, 1), (-1, 1)];

    /// The cells to walk through, in order, to get from the hero's cell to goal. The route ends
    /// as near the goal as could be reached when the goal itself cannot be (it is inside a wall).
    /// cellCost gives the extra cost of entering a cell: negative means it is blocked, 0 is open
    /// ground and a positive number makes the cell less attractive (shallow water).
    public static List<(int X, int Y)> Find((int X, int Y) goal, Func<int, int, int> cellCost)
    {
        Dictionary<(int, int), int> costs = [];
        Dictionary<(int, int), (int, int)> cameFrom = [];
        Dictionary<(int, int), int> best = new() { [(0, 0)] = 0 };
        PriorityQueue<(int, int), int> open = new();
        (int X, int Y) closest = (0, 0);

        open.Enqueue((0, 0), Estimate((0, 0), goal));

        for (int looked = 0; looked < MAX_EXPANSIONS && open.TryDequeue(out (int X, int Y) cell, out _); looked++)
        {
            if (Estimate(cell, goal) < Estimate(closest, goal))
            {
                closest = cell;
            }

            if (cell == goal)
            {
                break;
            }

            Expand(cell, goal, (cellCost, costs), (best, cameFrom, open));
        }

        return Rebuild(closest, cameFrom);
    }

    // Looks at the neighbours of a cell and queues the ones that are now reached more cheaply.
    private static void Expand(
        (int X, int Y) cell,
        (int X, int Y) goal,
        (Func<int, int, int> Cost, Dictionary<(int, int), int> Known) costing,
        (Dictionary<(int, int), int> Best, Dictionary<(int, int), (int, int)> From, PriorityQueue<(int, int), int> Open) search)
    {
        foreach ((int dx, int dy) in NEIGHBOURS)
        {
            (int X, int Y) next = (cell.X + dx, cell.Y + dy);
            int extra = CostOf(next, costing);

            if (extra < 0 || !CanCutCorner(cell, (dx, dy), costing))
            {
                continue;
            }

            int through = search.Best[cell] + (dx != 0 && dy != 0 ? DIAGONAL_COST : STRAIGHT_COST) + extra;

            if (!search.Best.TryGetValue(next, out int known) || through < known)
            {
                search.Best[next] = through;
                search.From[next] = cell;
                search.Open.Enqueue(next, through + Estimate(next, goal));
            }
        }
    }

    // A cell's cost, or -1 outside the search area (cached, because asking the game is slow).
    private static int CostOf((int X, int Y) cell, (Func<int, int, int> Cost, Dictionary<(int, int), int> Known) costing)
    {
        if (Math.Abs(cell.X) > RADIUS || Math.Abs(cell.Y) > RADIUS)
        {
            return -1;
        }

        if (!costing.Known.TryGetValue(cell, out int cost))
        {
            cost = costing.Cost(cell.X, cell.Y);
            costing.Known[cell] = cost;
        }

        return cost;
    }

    // A diagonal step may not squeeze between two blocked cells that touch corners.
    private static bool CanCutCorner((int X, int Y) cell, (int X, int Y) step, (Func<int, int, int> Cost, Dictionary<(int, int), int> Known) costing)
    {
        if (step.X == 0 || step.Y == 0)
        {
            return true;
        }

        return CostOf((cell.X + step.X, cell.Y), costing) >= 0 && CostOf((cell.X, cell.Y + step.Y), costing) >= 0;
    }

    // The straight line distance guess: as many diagonal steps as possible, then straight ones.
    private static int Estimate((int X, int Y) from, (int X, int Y) to)
    {
        int across = Math.Abs(to.X - from.X);
        int down = Math.Abs(to.Y - from.Y);

        return STRAIGHT_COST * Math.Max(across, down) + (DIAGONAL_COST - STRAIGHT_COST) * Math.Min(across, down);
    }

    // Walks back from the end to the start, leaving out the start cell itself.
    private static List<(int X, int Y)> Rebuild((int X, int Y) end, Dictionary<(int, int), (int, int)> cameFrom)
    {
        List<(int X, int Y)> route = [];

        for ((int X, int Y) cell = end; cell != (0, 0); cell = cameFrom[cell])
        {
            route.Add(cell);
        }

        route.Reverse();

        return route;
    }
}
