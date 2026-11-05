using System.Text;
using FaeryTale.Game.Data;
using FaeryTale.Game.Output;

namespace FaeryTale.Game.Testing;

/// Test helper: writing the main game state as text so tests can compare state instead of pixels.
/// The format matches the Rust version line for line, because the golden hashes are taken of it.
public static class StateDump
{
    /// Number of inventory slots the dump lists (the weapons and keys, up to the arrows).
    private const int DUMPED_STUFF_SLOTS = 35;
    private const int DUMPED_ACTORS = 20;

    /// Used when FTA_STATE_AFTER is not set.
    private const uint DEFAULT_AFTER_TICKS = 1;

    /// The state as text: the brother, place, stats, clock, random seeds, every actor, and the
    /// inventory.
    public static string StateText(Data.Game game)
    {
        StringBuilder text = new(HeroWorldLines(game));

        text.Append($"seed {game.Rng.Seed1} {game.Rng.Seed2}\n");

        for (int number = 0; number < DUMPED_ACTORS; number++)
        {
            text.Append(ActorLine(number, game.Actors.List[number]));
        }

        text.Append("stuff");

        for (int slot = 0; slot < DUMPED_STUFF_SLOTS; slot++)
        {
            text.Append($" {game.Hero.Stuff[slot]}");
        }

        text.Append('\n');

        return text.ToString();
    }

    /// One line describing actor number.
    public static string ActorLine(int number, Shape a) =>
        $"actor {number} pos {a.AbsX},{a.AbsY} type {a.Kind} race {a.Race} idx {a.Index} wpn {a.Weapon} env {a.Environ} goal {a.Goal} tac {a.Tactic} state {a.State} face {a.Facing} vit {a.Vitality} vel {a.VelX},{a.VelY}\n";

    /// Writes StateText to the file at path.
    public static void Dump(Data.Game game, string path)
    {
        try
        {
            File.WriteAllText(path, StateText(game));
        }
        catch (IOException error)
        {
            Console.Error.WriteLine($"Could not write the state dump {path}: {error.Message}");
        }
    }

    /// FTA_STATE=file writes the game state as text once FTA_STATE_AFTER ticks have passed (then
    /// quits). A test helper; the display only counts the ticks.
    public static void DumpIfDue(Data.Game game, IOutput output)
    {
        string? path = Environment.GetEnvironmentVariable("FTA_STATE");

        if (path is null)
        {
            return;
        }

        uint after = uint.TryParse(Environment.GetEnvironmentVariable("FTA_STATE_AFTER"), out uint given) ? given : DEFAULT_AFTER_TICKS;

        if (output.Ticks < after)
        {
            return;
        }

        Dump(game, path);
        Environment.Exit(0);
    }

    // The first lines of the dump: where the hero is, his stats, and the clock and light.
    private static string HeroWorldLines(Data.Game game) =>
        $"brother {game.Hero.Brother} region {game.World.Region} hero {game.Hero.X},{game.Hero.Y} map {game.World.MapX},{game.World.MapY}\n"
        + $"brave {game.Hero.Brave} luck {game.Hero.Luck} kind {game.Hero.Kindness} wealth {game.Hero.Wealth} hunger {game.Hero.Hunger} fatigue {game.Hero.Fatigue}\n"
        + $"daynight {game.World.DayClock} light {game.World.LightLevel} riding {game.Hero.Riding} flying {game.Hero.Flying} anix {game.Actors.Active},{game.Actors.Drawn}\n";
}
