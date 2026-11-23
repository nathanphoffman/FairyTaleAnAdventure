using FaeryTale.Game.Data;
using FaeryTale.Game.Output;
using FaeryTale.Game.Render;

namespace FaeryTale.Game.Ui;

/// TakeItems: searching a dead body.
public static partial class TakeItems
{
    /// Whether an actor is a body that has not been searched yet (a dead enemy or person; a searched
    /// body has its weapon set to -1).
    public static bool IsUnsearchedBody(Shape actor) =>
        (actor.Kind == ActorKind.ENEMY || actor.Kind == ActorKind.SETFIG)
        && actor.State == ActorState.DEAD
        && actor.Weapon >= 0
        && actor.Vitality == 0;

    /// The weapon a body yields (0 for none): the unarmed and odd values count as nothing.
    internal static uint BodyWeapon(sbyte raw) => (byte)raw > MAX_BODY_WEAPON ? 0u : (uint)(byte)raw;

    /// The hero keeps a better weapon in hand and only takes up a found one that is better.
    internal static sbyte BetterWeapon(sbyte current, uint found) => (sbyte)found > current ? (sbyte)found : current;

    /// What searching a body turns up: roll draws the arrows or the treasure when asked.
    internal static BodyLoot BodyLootFor(sbyte bodyWeaponRaw, sbyte heroWeapon, Func<LootRoll, int> roll)
    {
        uint weapon = BodyWeapon(bodyWeaponRaw);
        sbyte newHeroWeapon = weapon != 0 ? BetterWeapon(heroWeapon, weapon) : heroWeapon;

        if (weapon == ARCHER_WEAPON)
        {
            return new BodyLoot(weapon, newHeroWeapon, roll(LootRoll.Arrows), 0);
        }

        return new BodyLoot(weapon, newHeroWeapon, null, (uint)roll(LootRoll.Treasure));
    }

    // Searches the dead body NearestObject: takes its weapon and any treasure.
    private static void SearchBody(Data.Game game, IOutput output)
    {
        Shape actor = game.Actors.List[game.Actors.NearestObject];

        Messages.PrintWrapped(game, output, "% searched the body and found"u8);
        output.Print("");

        byte race = actor.Race;
        BodyLoot loot = BodyLootFor(actor.Weapon, game.Actors.List[0].Weapon, roll =>
            roll == LootRoll.Arrows ? game.Rng.Rand8() + EXTRA_ARROWS_MIN : (int)RollBodyTreasure(race, game.Rng));

        actor.Weapon = -1;
        TakeFoundWeapon(game, output, loot);

        if (loot.Arrows is { } arrows)
        {
            output.PrintCont(" and ");
            output.PrintNumber(arrows, 1);
            output.PrintCont(" Arrows.");
            ItemsMenu.AddStuff(game, ItemsMenu.ARROWS, (byte)arrows);

            return;
        }

        TakeTreasure(game, output, loot);
        output.PrintCont(".");
    }

    // Says what weapon was found, adds it to the pack, and arms the hero with it if it is better.
    private static void TakeFoundWeapon(Data.Game game, IOutput output, BodyLoot loot)
    {
        if (loot.Weapon == 0)
        {
            return;
        }

        output.PrintCont("a ");
        output.PrintCont(ItemsMenu.ItemName((int)loot.Weapon - 1));
        ItemsMenu.AddStuff(game, (int)loot.Weapon - 1, 1);
        game.Actors.List[0].Weapon = loot.HeroWeapon;
    }
}
