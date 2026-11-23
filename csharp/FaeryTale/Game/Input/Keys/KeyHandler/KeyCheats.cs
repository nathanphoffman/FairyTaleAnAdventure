using FaeryTale.Game.Creatures;
using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Data;
using FaeryTale.Game.Output;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Input.Keys;

/// KeyHandler: the cheat keys.
public static partial class KeyHandler
{
    // Debug keys that give the hero a carrier or items. Returns true if the key was one.
    private static bool CheatItems(Data.Game game, IOutput output, sbyte key)
    {
        if (game.Ui.CheatEnabled == 0)
        {
            return false;
        }

        if (key == CHEAT_CARRIER)
        {
            if (game.Actors.ActiveCarrier == CARRIER_BIRD)
            {
                game.Hero.Stuff[RAFT_STONE_SLOT] = 1;
            }

            Extents.CenterExtentOn(game, 0, (short)((short)game.Hero.X + CARRIER_OFFSET), (short)((short)game.Hero.Y + CARRIER_OFFSET));
            Encounters.LoadCarrier(game, output, CARRIER_BIRD);

            return true;
        }

        if (key == CHEAT_STUFF)
        {
            int slot = game.Rng.Rnd(GOLDBASE);

            game.Hero.Stuff[slot] = unchecked((byte)(game.Hero.Stuff[slot] + CHEAT_STUFF_COUNT));
            Menus.UpdateMenuAvailability(game);
            game.Hero.Stuff[VIAL_SLOT] = 0;

            return true;
        }

        return false;
    }

    // Debug keys that rescue, redraw or move the hero about. Returns true if the key was one.
    private static bool CheatMove(Data.Game game, IOutput output, sbyte key) =>
        game.Ui.CheatEnabled != 0 && (CheatShow(game, output, key) || CheatNudge(game, key));

    // The debug keys that rescue the princess, print debug lines or move the clock on.
    private static bool CheatShow(Data.Game game, IOutput output, sbyte key)
    {
        switch (key)
        {
            case CHEAT_RESCUE:
                Rescue.RescuePrincess(game, output);

                return true;

            case CHEAT_PRQ_2:
                TextRoutines.QueuePrint(game.Ui, PRQ_DEBUG_COORDS);

                return true;

            case CHEAT_PRQ_3:
                TextRoutines.QueuePrint(game.Ui, PRQ_DEBUG_SECTOR);

                return true;

            case CHEAT_TIME:
                game.World.DayClock = unchecked((ushort)(game.World.DayClock + CHEAT_TIME_STEP));

                return true;

            default:
                return false;
        }
    }

    // The debug keys that move the hero (and the map with him) a step in a direction.
    private static bool CheatNudge(Data.Game game, sbyte key)
    {
        (int dx, int dy) = key switch
        {
            CHEAT_NORTH => (0, -CHEAT_STEP_Y),
            CHEAT_SOUTH => (0, CHEAT_STEP_Y),
            CHEAT_EAST => (CHEAT_STEP_X, 0),
            CHEAT_WEST => (-CHEAT_STEP_X, 0),
            _ => (0, 0),
        };

        if (dx == 0 && dy == 0)
        {
            return false;
        }

        Shape hero = game.Actors.List[0];

        hero.AbsX = unchecked((ushort)(hero.AbsX + dx));
        hero.AbsY = unchecked((ushort)(hero.AbsY + dy));
        game.World.MapX = unchecked((ushort)(game.World.MapX + dx));
        game.World.MapY = unchecked((ushort)(game.World.MapY + dy));

        return true;
    }
}
