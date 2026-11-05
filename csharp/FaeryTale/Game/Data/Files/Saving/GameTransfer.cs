using FaeryTale.Game.Data;

namespace FaeryTale.Game.Files;

/// Writes or reads the whole game state through a Transfer: the misc variables, the region, the
/// actors, the inventories and missiles, the extents and the object lists. The order of the calls
/// is the save file format, so it must not change.
public static class GameTransfer
{
    /// Entries of the extent list that are saved, and entries in the per-region object tables.
    private const int SAVED_EXTENTS = 2;
    private const int OBJECT_REGIONS = 10;

    public static void TransferAll(Data.Game game, Transfer transfer)
    {
        TransferVariables(game, transfer);
        transfer.Word(ref game.World.Region);

        transfer.Word(ref game.Actors.Active);
        transfer.Word(ref game.Actors.Drawn);
        transfer.Word(ref game.Actors.NextMissile);

        for (int position = 0; position < game.Actors.Active; position++)
        {
            TransferActor(transfer, game.Actors.List[position]);
        }

        TransferParty(game.Actors, game.Hero, transfer);
        TransferObjects(game, transfer);
    }

    // The 40 variables the original saved as one 80 byte block, in that order: the map and hero
    // positions, the stats (bravery, luck, kindness, wealth, hunger, fatigue), which brother is
    // playing, the day/night clock, the magic timers and so on.
    private static void TransferVariables(Data.Game game, Transfer transfer)
    {
        transfer.Word(ref game.World.MapX);
        transfer.Word(ref game.World.MapY);
        transfer.Word(ref game.Hero.X);
        transfer.Word(ref game.Hero.Y);
        transfer.Word(ref game.Hero.SafeX);
        transfer.Word(ref game.Hero.SafeY);
        transfer.Word(ref game.Hero.SafeR);
        transfer.Word(ref game.World.TileX);
        transfer.Word(ref game.World.TileY);
        transfer.Word(ref game.Ui.CheatEnabled);
        transfer.Word(ref game.Hero.Riding);
        transfer.Word(ref game.Hero.Flying);
        transfer.Word(ref game.Actors.CarrierSlot);
        transfer.Word(ref game.Actors.TurtleNearness);
        transfer.Word(ref game.Actors.RaftNearness);
        transfer.Word(ref game.Hero.Brave);
        transfer.Word(ref game.Hero.Luck);
        transfer.Word(ref game.Hero.Kindness);
        transfer.Word(ref game.Hero.Wealth);
        transfer.Word(ref game.Hero.Hunger);
        transfer.Word(ref game.Hero.Fatigue);
        transfer.Word(ref game.Hero.Brother);
        transfer.Word(ref game.Hero.PrincessesRescued);
        transfer.Word(ref game.Hero.Sector);
        transfer.Word(ref game.Hero.Place);
        transfer.Word(ref game.World.DayClock);
        transfer.Word(ref game.World.LightLevel);
        transfer.Word(ref game.Actors.LoadedActorFile);
        transfer.Word(ref game.Actors.LoadedSetfigFile);
        transfer.Word(ref game.Actors.ActiveCarrier);
        transfer.Word(ref game.World.ExtentType);
        transfer.Word(ref game.Actors.Leader);
        transfer.Word(ref game.World.SecretTimer);
        transfer.Word(ref game.World.LightTimer);
        transfer.Word(ref game.World.FreezeTimer);
        transfer.Word(ref game.Ui.CurrentMenu);
        transfer.Word(ref game.Encounter.Kind);
        transfer.Word(ref game.SavePadding[0]);
        transfer.Word(ref game.SavePadding[1]);
        transfer.Word(ref game.SavePadding[2]);
    }

    private static void TransferActor(Transfer transfer, Shape actor)
    {
        transfer.Word(ref actor.AbsX);
        transfer.Word(ref actor.AbsY);
        transfer.Word(ref actor.RelX);
        transfer.Word(ref actor.RelY);
        transfer.Byte(ref actor.Kind);
        transfer.Byte(ref actor.Race);
        transfer.Byte(ref actor.Index);
        transfer.Byte(ref actor.Visible);
        transfer.Byte(ref actor.Weapon);
        transfer.Byte(ref actor.Environ);
        transfer.Byte(ref actor.Goal);
        transfer.Byte(ref actor.Tactic);
        transfer.Byte(ref actor.State);
        transfer.Byte(ref actor.Facing);
        transfer.Word(ref actor.Vitality);
        transfer.Byte(ref actor.VelX);
        transfer.Byte(ref actor.VelY);
    }

    private static void TransferExtent(Transfer transfer, ref Extent extent)
    {
        transfer.Word(ref extent.X1);
        transfer.Word(ref extent.Y1);
        transfer.Word(ref extent.X2);
        transfer.Word(ref extent.Y2);
        transfer.Byte(ref extent.Etype);
        transfer.Byte(ref extent.V1);
        transfer.Byte(ref extent.V2);
        transfer.Byte(ref extent.V3);
    }

    private static void TransferObject(Transfer transfer, ref WorldObject item)
    {
        transfer.Word(ref item.Xc);
        transfer.Word(ref item.Yc);
        transfer.Byte(ref item.ObId);
        transfer.Byte(ref item.ObStat);
    }

    private static void TransferMissile(Transfer transfer, Missile missile)
    {
        transfer.Word(ref missile.AbsX);
        transfer.Word(ref missile.AbsY);
        transfer.Byte(ref missile.MissileType);
        transfer.Byte(ref missile.TimeOfFlight);
        transfer.Byte(ref missile.Speed);
        transfer.Byte(ref missile.Direction);
        transfer.Byte(ref missile.Archer);
    }

    // The three brothers' item lists and the missiles (the count NextMissile is saved elsewhere).
    private static void TransferParty(Actors actors, Hero hero, Transfer transfer)
    {
        transfer.Bytes(hero.Julstuff.AsSpan(0, Hero.SAVED_STUFF_SLOTS));
        transfer.Bytes(hero.Philstuff.AsSpan(0, Hero.SAVED_STUFF_SLOTS));
        transfer.Bytes(hero.Kevstuff.AsSpan(0, Hero.SAVED_STUFF_SLOTS));

        foreach (Missile missile in actors.Missiles)
        {
            TransferMissile(transfer, missile);
        }
    }

    // The extents, then the object lists: the global one, how many each region uses, and the
    // objects each region uses.
    private static void TransferObjects(Data.Game game, Transfer transfer)
    {
        for (int index = 0; index < SAVED_EXTENTS; index++)
        {
            TransferExtent(transfer, ref game.Extents[index]);
        }

        for (int index = 0; index < ObjectTables.Glbobs; index++)
        {
            TransferObject(transfer, ref game.Objects.Global[index]);
        }

        for (int region = 0; region < OBJECT_REGIONS; region++)
        {
            transfer.Word(ref game.Objects.Used[region]);
        }

        for (int region = 0; region < OBJECT_REGIONS; region++)
        {
            transfer.Word(ref game.Objects.Distributed[region]);
        }

        for (int region = 0; region < OBJECT_REGIONS; region++)
        {
            WorldObject[] list = game.Objects.Region(region);

            for (int index = 0; index < Math.Min((int)game.Objects.Used[region], list.Length); index++)
            {
                TransferObject(transfer, ref list[index]);
            }
        }
    }
}
