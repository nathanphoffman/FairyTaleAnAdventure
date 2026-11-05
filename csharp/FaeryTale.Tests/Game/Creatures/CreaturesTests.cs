using FaeryTale.Game.Creatures;
using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Data;
using FaeryTale.Game.Land.TravelDoors;

namespace FaeryTale.Tests;

public class CreaturesTests
{
    private static Shape Actor(short vitality, sbyte state) => new() { Vitality = vitality, State = state };

    private static Hero HeroWith(short brave, short luck, short kindness) => new() { Brave = brave, Luck = luck, Kindness = kindness };

    [Fact]
    public void SteersTowardATarget()
    {
        // Actor is right of and level with the target: direction 7.
        Assert.Equal(new Steering.Course(false, 7), Steering.Steer(100, 0, 0, () => false));

        // Below the target: direction 1.
        Assert.Equal(new Steering.Course(false, 1), Steering.Steer(0, 100, 0, () => false));

        // Exactly on the target: stop.
        Assert.True(Steering.Steer(0, 0, 0, () => false).Stop);
    }

    [Fact]
    public void SteeringAwayReversesDirection()
    {
        Assert.Equal(new Steering.Course(false, 3), Steering.Steer(100, 0, 3, () => false));
    }

    [Fact]
    public void CloseTargetsWobbleInModes1To2()
    {
        Assert.Equal(0, Steering.Steer(10, 0, 1, () => true).Facing);
        Assert.Equal(6, Steering.Steer(10, 0, 1, () => false).Facing);

        // 35 is close enough for mode 1 (under 40) but not for mode 2 (under 30).
        Assert.Equal(0, Steering.Steer(35, 0, 1, () => true).Facing);
        Assert.Equal(7, Steering.Steer(35, 0, 2, () => true).Facing);
    }

    [Fact]
    public void ADiagonalNeedsBothAxesToMatter()
    {
        // A 3:1 slope is treated as straight along x.
        Assert.Equal(7, Steering.Steer(90, 30, 0, () => false).Facing);
        Assert.Equal(0, Steering.Steer(60, 60, 0, () => false).Facing);
    }

    [Fact]
    public void TheRandomBitIsOnlyUsedWhenTheActorMoves()
    {
        bool asked = false;

        Steering.Steer(0, 0, 0, () =>
        {
            asked = true;

            return true;
        });

        Assert.False(asked);
    }

    [Fact]
    public void NearnessOfTheCarrier()
    {
        Assert.Equal(2, Carriers.Nearness(0, 0));
        Assert.Equal(2, Carriers.Nearness(8, 0));
        Assert.Equal(1, Carriers.Nearness(9, 0));
        Assert.Equal(1, Carriers.Nearness(-15, 15));
        Assert.Equal(0, Carriers.Nearness(16, 0));
        Assert.Equal(0, Carriers.Nearness(0, -16));
    }

    [Fact]
    public void ALivingActorDoesNotDie()
    {
        Shape actor = Actor(1, 0);

        Assert.Null(Encounters.StartDying(5, actor, HeroWith(10, 10, 10)));
        Assert.Equal(0, actor.State);
    }

    [Fact]
    public void FinishedActorsAreNotKilledTwice()
    {
        Hero hero = HeroWith(10, 10, 10);

        Assert.Null(Encounters.StartDying(5, Actor(0, ActorState.DYING), hero));
        Assert.Null(Encounters.StartDying(5, Actor(0, ActorState.DEAD), hero));
        Assert.Equal(10, hero.Brave);
    }

    [Fact]
    public void KillingAnEnemyStartsTheDeathGivingBravery()
    {
        Shape actor = Actor(-4, 0);
        Hero hero = HeroWith(10, 10, 10);

        Assert.Equal(new Encounters.Death(false), Encounters.StartDying(5, actor, hero));
        Assert.Equal((0, ActorState.DYING, ActorGoal.DEATH, Encounters.DYING_COUNTDOWN), (actor.Vitality, actor.State, actor.Goal, actor.Tactic));
        Assert.Equal((11, 10, 10), (hero.Brave, hero.Luck, hero.Kindness));
    }

    [Fact]
    public void KillingAPersonCostsKindnessButNotBelowZero()
    {
        Hero hero = HeroWith(10, 10, 5);

        Encounters.StartDying(5, new Shape { Kind = ActorKind.SETFIG }, hero);
        Assert.Equal(2, hero.Kindness);

        Encounters.StartDying(5, new Shape { Kind = ActorKind.SETFIG }, hero);
        Assert.Equal(0, hero.Kindness);
    }

    [Fact]
    public void TheLassoKeeperIsNotAMurderAndTheDarkKnightSpeaks()
    {
        Hero hero = HeroWith(10, 10, 5);

        Encounters.StartDying(5, new Shape { Kind = ActorKind.SETFIG, Race = 0x89 }, hero);
        Assert.Equal(5, hero.Kindness);

        Assert.Equal(new Encounters.Death(true), Encounters.StartDying(5, new Shape { Kind = ActorKind.SETFIG, Race = 7 }, hero));
        Assert.Equal(5, hero.Kindness);
    }

    [Fact]
    public void TheHerosOwnDeathCostsLuckNotBravery()
    {
        Hero hero = HeroWith(10, 20, 10);

        Encounters.StartDying(0, Actor(0, 0), hero);

        Assert.Equal((10, 15), (hero.Brave, hero.Luck));
    }

    [Fact]
    public void SpiritsNeedARealWeaponAndTheGhostKeeperTheItem()
    {
        Assert.True(Hits.TooWeakToHurt(9, Hits.MIN_HURT_WEAPON - 1, true));
        Assert.False(Hits.TooWeakToHurt(9, Hits.MIN_HURT_WEAPON, false));
        Assert.True(Hits.TooWeakToHurt(0x89, 0, false));
        Assert.False(Hits.TooWeakToHurt(0x89, 0, true));
        Assert.False(Hits.TooWeakToHurt(0, 0, false));
    }

    [Fact]
    public void AHitNeverTakesVitalityBelowZero()
    {
        Assert.Equal(6, Hits.VitalityAfterHit(10, 4));
        Assert.Equal(0, Hits.VitalityAfterHit(3, 9));
    }

    [Fact]
    public void HitSoundsByWhoHitWhom()
    {
        Assert.Equal(Hits.HitSound.Arrow, Hits.HitSoundFor(-1, 0));
        Assert.Equal(Hits.HitSound.Rock, Hits.HitSoundFor(-2, 5));
        Assert.Equal(Hits.HitSound.HeroHurt, Hits.HitSoundFor(3, 0));
        Assert.Equal(Hits.HitSound.FoeHurt, Hits.HitSoundFor(0, 3));
    }

    [Fact]
    public void DragonsAndPeopleAreNotPushedBack()
    {
        Assert.False(Hits.CanBePushed(ActorKind.DRAGON));
        Assert.False(Hits.CanBePushed(ActorKind.SETFIG));
        Assert.True(Hits.CanBePushed(2));
    }

    [Fact]
    public void WhatIsSaidAfterAFight()
    {
        ushort calm = Hits.PEACEFUL_EXTENT - 1;

        Assert.IsType<Hits.BattleReport.Silent>(Hits.BattleReportFor(0, 3, 0, calm));
        Assert.IsType<Hits.BattleReport.Bravely>(Hits.BattleReportFor(Hits.BRAVE_LOW_VITALITY - 1, 1, 0, calm));

        // Nearly dead but nobody died: the normal report.
        Assert.Equal(new Hits.BattleReport.Foes(0, 2), Hits.BattleReportFor(Hits.BRAVE_LOW_VITALITY - 1, 0, 2, calm));
        Assert.Equal(new Hits.BattleReport.Foes(2, 1), Hits.BattleReportFor(50, 2, 1, calm));

        // Peaceful places say nothing.
        Assert.IsType<Hits.BattleReport.Silent>(Hits.BattleReportFor(50, 2, 1, Hits.PEACEFUL_EXTENT));
    }

    private static Door DoorOf(sbyte kind, sbyte secs) => new() { Xc1 = 100, Yc1 = 200, Xc2 = 300, Yc2 = 400, Kind = kind, Secs = secs };

    [Fact]
    public void EnteringNeedsTheHeroLinedUp()
    {
        // Horizontal doors (odd kind) go by y, vertical ones by x.
        Assert.True(EnteringDoors.MayEnter(DoorOf(1, 1), 0, 0x00, 0));
        Assert.False(EnteringDoors.MayEnter(DoorOf(1, 1), 0, 0x10, 0));
        Assert.True(EnteringDoors.MayEnter(DoorOf(2, 1), 6, 0, 0));
        Assert.False(EnteringDoors.MayEnter(DoorOf(2, 1), 7, 0, 0));
    }

    [Fact]
    public void TheDesertDoorNeedsFiveStatues()
    {
        Assert.False(EnteringDoors.MayEnter(DoorOf(EnteringDoors.DESERT, 1), 0, 0, EnteringDoors.STATUES_NEEDED - 1));
        Assert.True(EnteringDoors.MayEnter(DoorOf(EnteringDoors.DESERT, 1), 0, 0, EnteringDoors.STATUES_NEEDED));
        Assert.True(EnteringDoors.MayEnter(DoorOf(2, 1), 0, 0, 0));
    }

    [Fact]
    public void DoorsLeadToTwoIndoorRegions()
    {
        Assert.Equal(EnteringDoors.REGION_INSIDE, EnteringDoors.RegionInside(DoorOf(1, 1)));
        Assert.Equal(EnteringDoors.REGION_INSIDE_OTHER, EnteringDoors.RegionInside(DoorOf(1, 2)));
    }

    // Doors whose entrance the binary search cannot reach, because a neighbouring door's x range
    // (16 pixels) overlaps theirs. This is how the original data behaves; they are listed so a
    // change to the search is noticed.
    private static readonly int[] UNREACHABLE = [38, 67];

    [Fact]
    public void EveryOtherDoorIsFoundAtItsOwnEntrance()
    {
        for (int index = 0; index < TravelDoors.DOOR_COUNT; index++)
        {
            Door door = EnteringDoors.DoorAt(index);
            int? found = EnteringDoors.FindOutdoorDoor(door.Xc1, door.Yc1);

            if (UNREACHABLE.Contains(index))
            {
                Assert.Null(found);

                continue;
            }

            Assert.NotNull(found);
            Assert.Equal((door.Xc1, door.Yc1), (EnteringDoors.DoorAt(found.Value).Xc1, EnteringDoors.DoorAt(found.Value).Yc1));
        }
    }

    [Fact]
    public void EmptyGroundHasNoDoor()
    {
        Assert.Null(EnteringDoors.FindOutdoorDoor(1, 1));
    }

    [Fact]
    public void LeavingIsTheMirrorOfEntering()
    {
        Assert.True(LeavingDoors.MayLeave(DoorOf(1, 1), 0, 0x10));
        Assert.False(LeavingDoors.MayLeave(DoorOf(1, 1), 0, 0));
        Assert.True(LeavingDoors.MayLeave(DoorOf(2, 1), 2, 0));
        Assert.False(LeavingDoors.MayLeave(DoorOf(2, 1), 1, 0));
    }

    [Fact]
    public void ExitsAreFoundByTile()
    {
        Assert.True(LeavingDoors.IsExitHere(DoorOf(2, 1), 300, 400));
        Assert.False(LeavingDoors.IsExitHere(DoorOf(2, 1), 316, 400));
        Assert.True(LeavingDoors.IsExitHere(DoorOf(1, 1), 316, 400)); // horizontal: also one tile right
        Assert.False(LeavingDoors.IsExitHere(DoorOf(1, 1), 300, 416));
    }

    [Fact]
    public void ArrivalSpotsDependOnTheDoorKind()
    {
        Assert.Equal(((ushort)324, (ushort)416), EnteringDoors.InsideSpot(DoorOf(TravelDoors.CAVE, 1)));
        Assert.Equal(((ushort)316, (ushort)400), EnteringDoors.InsideSpot(DoorOf(1, 1)));
        Assert.Equal(((ushort)299, (ushort)416), EnteringDoors.InsideSpot(DoorOf(2, 1)));
        Assert.Equal(((ushort)96, (ushort)216), LeavingDoors.OutsideSpot(DoorOf(TravelDoors.CAVE, 1)));
        Assert.Equal(((ushort)116, (ushort)234), LeavingDoors.OutsideSpot(DoorOf(1, 1)));
        Assert.Equal(((ushort)120, (ushort)216), LeavingDoors.OutsideSpot(DoorOf(2, 1)));
    }
}
