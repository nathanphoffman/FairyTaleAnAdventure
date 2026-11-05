using FaeryTale.Game.Data;
using FaeryTale.Game.Input;
using FaeryTale.Game.Input.Keys;
using FaeryTale.Game.Ui;
using FaeryTale.Game.Ui.Options;

namespace FaeryTale.Tests;

public class UiAndInputTests
{
    [Fact]
    public void CompassRegions()
    {
        Assert.Equal(9, InputHandler.CompassDirection(200, 150));
        Assert.Equal(0, InputHandler.CompassDirection(280, 150));
        Assert.Equal(7, InputHandler.CompassDirection(280, 170));
        Assert.Equal(6, InputHandler.CompassDirection(280, 190));
        Assert.Equal(1, InputHandler.CompassDirection(296, 150));
        Assert.Equal(9, InputHandler.CompassDirection(296, 170));
        Assert.Equal(5, InputHandler.CompassDirection(296, 190));
        Assert.Equal(2, InputHandler.CompassDirection(310, 150));
        Assert.Equal(3, InputHandler.CompassDirection(310, 170));
        Assert.Equal(4, InputHandler.CompassDirection(310, 190));
    }

    [Fact]
    public void MenuKeys()
    {
        Assert.Equal(0x61, InputHandler.MenuKeyAt(225, 148));
        Assert.Equal(0x62, InputHandler.MenuKeyAt(250, 148));
        Assert.Equal(0x65, InputHandler.MenuKeyAt(225, 166));
        Assert.Null(InputHandler.MenuKeyAt(100, 148));
        Assert.Null(InputHandler.MenuKeyAt(225, 100));
    }

    private static Func<int, int> Dice(bool trips, bool right) => n => n == 4 ? (trips ? 0 : 1) : (right ? 1 : 0);

    [Fact]
    public void OnlyAStarvingHeroStumbles()
    {
        int drawn = 0;

        Assert.Equal(3, HeroControls.Stumble(3, HeroControls.STUMBLE_HUNGER, _ =>
        {
            drawn++;

            return 0;
        }));
        Assert.Equal(0, drawn);

        short starving = HeroControls.STUMBLE_HUNGER + 1;

        Assert.Equal(3, HeroControls.Stumble(3, starving, Dice(false, true)));
        Assert.Equal(4, HeroControls.Stumble(3, starving, Dice(true, true)));
        Assert.Equal(2, HeroControls.Stumble(3, starving, Dice(true, false)));
        Assert.Equal(0, HeroControls.Stumble(7, starving, Dice(true, true)));
        Assert.Equal(7, HeroControls.Stumble(0, starving, Dice(true, false)));
    }

    [Fact]
    public void TimersCountDownStoppingAtZeroAndWeaponsDecideHowTheHeroFights()
    {
        Assert.Equal(2, HeroControls.TimerStep(3));
        Assert.Equal(0, HeroControls.TimerStep(1));
        Assert.Equal(0, HeroControls.TimerStep(0));

        Assert.Equal(ActorState.SHOOT1, HeroControls.FightingStateFor(ActorState.WALKING, HeroControls.WEAPON_BOW));
        Assert.Equal(ActorState.SHOOT1, HeroControls.FightingStateFor(ActorState.WALKING, HeroControls.WEAPON_WAND));
        Assert.Equal(ActorState.SHOOT1 + 3, HeroControls.FightingStateFor((sbyte)(ActorState.SHOOT1 + 3), HeroControls.WEAPON_WAND));
        Assert.Equal(ActorState.FIGHTING, HeroControls.FightingStateFor(ActorState.WALKING, 1));
    }

    [Fact]
    public void TheFairyCountdown()
    {
        Assert.Equal(((byte)1, HeroControls.FairyAction.Revive), HeroControls.FairyStep(1, 50, false));
        Assert.Equal(HeroControls.FairyAction.Wait, HeroControls.FairyStep(HeroControls.FAIRY_ARRIVES + 5, 50, false).Action);
        Assert.Equal(((byte)(HeroControls.FAIRY_ARRIVES - 1), HeroControls.FairyAction.FlyIn), HeroControls.FairyStep(HeroControls.FAIRY_ARRIVES, 50, false));
        Assert.Equal(HeroControls.FairyAction.Revive, HeroControls.FairyStep(HeroControls.FAIRY_GIVES_UP, 50, true).Action);
        Assert.Equal(HeroControls.FairyAction.NextBrother, HeroControls.FairyStep(HeroControls.FAIRY_GIVES_UP, 0, false).Action);
        Assert.Equal(HeroControls.FairyAction.Wait, HeroControls.FairyStep(HeroControls.FAIRY_GIVES_UP, 1, false).Action);
        Assert.Equal(HeroControls.FairyAction.Wait, HeroControls.FairyStep(HeroControls.FAIRY_RESURRECT, 0, true).Action);
    }

    [Fact]
    public void TakeClasses()
    {
        Assert.IsType<TakeItems.Take.Gold>(TakeItems.ClassifyTake(0x0d, 0, 0));
        Assert.Equal(new TakeItems.Take.Paper(false), TakeItems.ClassifyTake(0x14, 0, 7));
        Assert.Equal(new TakeItems.Take.Paper(true), TakeItems.ClassifyTake(0x14, 0, 8));
        Assert.Equal(new TakeItems.Take.Fruit(true), TakeItems.ClassifyTake(148, 14, 0));
        Assert.Equal(new TakeItems.Take.Fruit(false), TakeItems.ClassifyTake(148, 15, 0));
        Assert.IsType<TakeItems.Take.Eggs>(TakeItems.ClassifyTake(102, 0, 0));
        Assert.IsType<TakeItems.Take.Bones>(TakeItems.ClassifyTake(28, 0, 0));
        Assert.Equal(new TakeItems.Take.Container("a chest"), TakeItems.ClassifyTake(0x0f, 0, 0));
        Assert.Equal(new TakeItems.Take.Container("a brass urn"), TakeItems.ClassifyTake(0x0e, 0, 0));
        Assert.Equal(new TakeItems.Take.Container("some sacks"), TakeItems.ClassifyTake(0x10, 0, 0));
        Assert.IsType<TakeItems.Take.Left>(TakeItems.ClassifyTake(0x1d, 0, 0));
        Assert.IsType<TakeItems.Take.Plain>(TakeItems.ClassifyTake(1, 0, 0));
    }

    [Fact]
    public void ItemSlotLookupStopsAtTheZeroPair()
    {
        byte[] table = [5, 35, 7, 9, 0, 0, 9, 1];

        Assert.Equal(9, TakeItems.ItemSlot(7, table));
        Assert.Equal(35, TakeItems.ItemSlot(5, table));
        Assert.Null(TakeItems.ItemSlot(9, table));
        Assert.Null(TakeItems.ItemSlot(3, table));
    }

    [Fact]
    public void BodyLootRules()
    {
        Func<TakeItems.LootRoll, int> treasure(int value) => roll => roll == TakeItems.LootRoll.Treasure ? value : throw new InvalidOperationException();

        TakeItems.BodyLoot loot = TakeItems.BodyLootFor(2, 1, treasure(7));
        Assert.Equal((2u, (sbyte)2, (int?)null, 7u), (loot.Weapon, loot.HeroWeapon, loot.Arrows, loot.Treasure));

        Assert.Equal(3, TakeItems.BodyLootFor(2, 3, treasure(0)).HeroWeapon);

        loot = TakeItems.BodyLootFor(4, 1, roll => roll == TakeItems.LootRoll.Arrows ? 5 : throw new InvalidOperationException());
        Assert.Equal((4u, (int?)5, 0u, (sbyte)4), (loot.Weapon, loot.Arrows, loot.Treasure, loot.HeroWeapon));

        Assert.Equal(0u, TakeItems.BodyLootFor(-1, 2, treasure(0)).Weapon);
        Assert.Equal(0u, TakeItems.BodyWeapon(6));
        Assert.Equal(5u, TakeItems.BodyWeapon(5));
        Assert.Equal(3, TakeItems.BetterWeapon(1, 3));
        Assert.Equal(4, TakeItems.BetterWeapon(4, 3));
    }

    private static ContainerContents.Contents Rolls(int kind, params int[] items)
    {
        Queue<int> next = new(items);

        return ContainerContents.RollContents(n => n == 4 ? kind : next.Dequeue());
    }

    [Fact]
    public void ContainerContentsFollowTheRolls()
    {
        Assert.IsType<ContainerContents.Contents.Nothing>(Rolls(0));
        Assert.Equal(new ContainerContents.Contents.One(ItemsMenu.ARROWBASE), Rolls(1, 0));
        Assert.Equal(new ContainerContents.Contents.One(11), Rolls(1, 3));
        Assert.Equal(new ContainerContents.Contents.Two(10, 13), Rolls(2, 2, 5));
        Assert.Equal(new ContainerContents.Contents.Two(ContainerContents.HUNDRED_GOLD, ItemsMenu.ARROWBASE), Rolls(2, 0, 0));
        Assert.Equal(new ContainerContents.Contents.Two(ContainerContents.HUNDRED_GOLD, 12), Rolls(2, 0, 4));
        Assert.Equal(new ContainerContents.Contents.Two(11, ItemsMenu.ARROWBASE), Rolls(2, 3, 3, 0));
        Assert.Equal(new ContainerContents.Contents.ThreeOf(12), Rolls(3, 4));
        Assert.Equal([16, 21, 16], ((ContainerContents.Contents.ThreeKeys)Rolls(3, 0, 0, 5, 6)).Keys);
        Assert.Equal(16, ContainerContents.ContainerKey(0));
        Assert.Equal(20, ContainerContents.ContainerKey(7));
    }

    [Fact]
    public void SpellGateChecksInOrder()
    {
        Assert.Equal(Magic.SpellGate.NoSpell, Magic.GateFor(4, 1, 0, 0, 0, 0));
        Assert.Equal(Magic.SpellGate.NoSpell, Magic.GateFor(6, 0, 0, 0, 0, 0));
        Assert.Equal(Magic.SpellGate.QuietZone, Magic.GateFor(6, 1, 9, 0, 0, 0));
        Assert.Equal(Magic.SpellGate.Refused, Magic.GateFor(10, 1, 0, 2, 0, 0));
        Assert.Equal(Magic.SpellGate.Proceed, Magic.GateFor(10, 1, 0, 1, 0, 0));
        Assert.Equal(Magic.SpellGate.Proceed, Magic.GateFor(6, 1, 0, 5, 0, 9));
        Assert.Equal(Magic.SpellGate.Refused, Magic.GateFor(9, 1, 0, 0, 0, 8));
        Assert.Equal(Magic.SpellGate.Proceed, Magic.GateFor(9, 1, 0, 0, 1, 8));
        Assert.Equal(Magic.SpellGate.Proceed, Magic.GateFor(9, 1, 0, 0, 0, 7));
    }

    [Fact]
    public void StoneJumpRules()
    {
        byte[] list = [54, 43, 71, 77, 78, 102, 66, 121, 12, 85, 79, 40, 107, 38, 73, 21, 12, 26, 26, 53, 84, 60];
        ushort x = (54 << 8) + 100;
        ushort y = (43 << 8) + 70;

        Assert.IsType<Magic.StoneJump.Refused>(Magic.StoneTarget(143, x, y, 0, list));
        Assert.IsType<Magic.StoneJump.Refused>(Magic.StoneTarget(144, (ushort)(x + 100), y, 0, list));
        Assert.IsType<Magic.StoneJump.Refused>(Magic.StoneTarget(144, x, (ushort)(y + 100), 0, list));
        Assert.IsType<Magic.StoneJump.NoRing>(Magic.StoneTarget(144, 0x0500 + 100, 0x0500 + 70, 0, list));
        Assert.Equal(new Magic.StoneJump.To((71 << 8) + 100, (77 << 8) + 70), Magic.StoneTarget(144, x, y, 0, list));
        Assert.Equal(new Magic.StoneJump.To((73 << 8) + 90, (21 << 8) + 65), Magic.StoneTarget(144, (84 << 8) + 90, (60 << 8) + 65, 7, list));
        Assert.Equal(new Magic.StoneJump.To((84 << 8) + 90, (60 << 8) + 65), Magic.StoneTarget(144, (54 << 8) + 90, (43 << 8) + 65, 9, list));
    }

    [Fact]
    public void HealAndFear()
    {
        Assert.Equal(((short)12, false), Magic.HealedVitality(5, 0, 3));
        Assert.Equal((Magic.HEAL_CAP_BASE, true), Magic.HealedVitality(14, 0, 7));
        Assert.Equal(((short)21, false), Magic.HealedVitality(10, 40, 7));
        Assert.Equal(((short)(Magic.HEAL_CAP_BASE + 10), true), Magic.HealedVitality(25, 40, 7));

        Shape actor = new() { Kind = ActorKind.ENEMY, Vitality = 5, Race = 6 };

        Assert.True(Magic.IsScaredByFearSpell(actor));
        actor.Race = 7;
        Assert.False(Magic.IsScaredByFearSpell(actor));
        actor.Race = 0;
        actor.Vitality = 0;
        Assert.False(Magic.IsScaredByFearSpell(actor));
    }

    [Fact]
    public void BuyAndTalkRules()
    {
        sbyte[] shop = [0, 3, 8, 10, 11, 15, 1, 30, 2, 45, 3, 75, 13, 20];

        Assert.IsType<ShopOptions.BuyGate.Nothing>(ShopOptions.BuyGateFor(0, 0x88, 5, shop));
        Assert.IsType<ShopOptions.BuyGate.Nothing>(ShopOptions.BuyGateFor(3, 0x89, 5, shop));
        Assert.IsType<ShopOptions.BuyGate.NotHandled>(ShopOptions.BuyGateFor(3, 0x88, 12, shop));
        Assert.Equal(new ShopOptions.BuyGate.Offer(0, 3), ShopOptions.BuyGateFor(3, 0x88, 5, shop));
        Assert.Equal(new ShopOptions.BuyGate.Offer(8, 10), ShopOptions.BuyGateFor(3, 0x88, 6, shop));
        Assert.False(ShopOptions.CanAfford(3, 3));
        Assert.True(ShopOptions.CanAfford(4, 3));
        Assert.IsType<ShopOptions.Purchase.TooPoor>(ShopOptions.PurchaseFor(3, 3, 0));
        Assert.Equal(new ShopOptions.Purchase.Made(7, ShopOptions.Bought.Food), ShopOptions.PurchaseFor(10, 3, 0));
        Assert.Equal(new ShopOptions.Purchase.Made(7, ShopOptions.Bought.Arrows), ShopOptions.PurchaseFor(10, 3, 8));
        Assert.Equal(new ShopOptions.Purchase.Made(1, ShopOptions.Bought.Item), ShopOptions.PurchaseFor(11, 10, 12));

        TalkOptions.Target figure = new(ActorKind.SETFIG, 0, 7, true);

        Assert.Equal(new TalkOptions.TalkAction.Person(true), TalkOptions.TalkActionFor(figure, false, 10, 0, false));
        Assert.IsType<TalkOptions.TalkAction.TooCloseToYell>(TalkOptions.TalkActionFor(figure, true, TalkOptions.TOO_CLOSE_TO_YELL - 1, 0, false));
        Assert.Equal(new TalkOptions.TalkAction.Person(true), TalkOptions.TalkActionFor(figure, true, TalkOptions.TOO_CLOSE_TO_YELL, 0, false));
        Assert.IsType<TalkOptions.TalkAction.Nothing>(TalkOptions.TalkActionFor(figure with { State = ActorState.DEAD }, false, 10, 0, false));

        TalkOptions.Target turtle = new(ActorKind.CARRIER, 0, 0, false);

        Assert.IsType<TalkOptions.TalkAction.TurtleGivesShell>(TalkOptions.TalkActionFor(turtle, false, 10, 5, false));
        Assert.IsType<TalkOptions.TalkAction.TurtleHasShell>(TalkOptions.TalkActionFor(turtle, false, 10, 5, true));
        Assert.Equal(new TalkOptions.TalkAction.EnemyShouts(7), TalkOptions.TalkActionFor(new(ActorKind.ENEMY, 0, 7, false), false, 10, 0, false));
    }

    [Fact]
    public void SpeechesDependOnWhoIsTalking()
    {
        TalkOptions.Talker talker = new(20, 0, 3, 0, 2, false);

        Assert.Equal(29, TalkOptions.PlainSpeech(0, talker));
        Assert.Equal(35, TalkOptions.PlainSpeech(0, talker with { Kindness = TalkOptions.KIND_ENOUGH - 1 }));
        Assert.Equal(15, TalkOptions.PlainSpeech(3, talker));
        Assert.Null(TalkOptions.PlainSpeech(4, talker));
        Assert.Equal(16, TalkOptions.PlainSpeech(4, talker with { PrincessFreed = true }));
        Assert.Equal(55, TalkOptions.PlainSpeech(12, talker));
        Assert.Equal(22, TalkOptions.PlainSpeech(12, talker with { Region = TalkOptions.RANGER_SWAMP_REGION }));
        Assert.Null(TalkOptions.PlainSpeech(1, talker));
        Assert.Null(TalkOptions.PlainSpeech(7, talker));
        Assert.Equal(13, TalkOptions.PlainSpeech(8, talker with { Fatigue = TalkOptions.TIRED - 1 }));
        Assert.Equal(14, TalkOptions.PlainSpeech(8, talker with { Fatigue = TalkOptions.TIRED, DayPeriod = TalkOptions.NIGHT_PERIOD }));
        Assert.Equal(12, TalkOptions.PlainSpeech(8, talker with { Fatigue = TalkOptions.TIRED, DayPeriod = TalkOptions.NIGHT_PERIOD + 1 }));

        Assert.IsType<TalkOptions.PriestAction.WritReward>(TalkOptions.PriestActionFor(true, 0, 0, 0));
        Assert.IsType<TalkOptions.PriestAction.WritAgain>(TalkOptions.PriestActionFor(true, 1, 99, 0));
        Assert.IsType<TalkOptions.PriestAction.Scold>(TalkOptions.PriestActionFor(false, 0, TalkOptions.KIND_ENOUGH - 1, 0));
        Assert.Equal(new TalkOptions.PriestAction.Bless(36), TalkOptions.PriestActionFor(false, 0, TalkOptions.KIND_ENOUGH, 0));
        Assert.Equal(new TalkOptions.PriestAction.Bless(37), TalkOptions.PriestActionFor(false, 0, TalkOptions.KIND_ENOUGH, 1));
        Assert.Equal(new TalkOptions.PriestAction.Bless(36), TalkOptions.PriestActionFor(false, 0, TalkOptions.KIND_ENOUGH, TalkOptions.PRIEST_BLESSINGS));

        Assert.Equal(TalkOptions.SorceressAction.AskForHelp, TalkOptions.SorceressActionFor(false, 0, () => throw new InvalidOperationException()));
        Assert.Equal(TalkOptions.SorceressAction.GiveLuck, TalkOptions.SorceressActionFor(true, 10, () => 11));
        Assert.Equal(TalkOptions.SorceressAction.Nothing, TalkOptions.SorceressActionFor(true, 10, () => 10));
    }
}
