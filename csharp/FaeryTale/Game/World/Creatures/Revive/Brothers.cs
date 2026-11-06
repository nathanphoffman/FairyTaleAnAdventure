using FaeryTale.Game.Data;
using FaeryTale.Game.Files;
using FaeryTale.Game.Land.Map;
using FaeryTale.Game.Output;
using FaeryTale.Game.Render;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Creatures.Revive;

/// Handing over from a fallen brother to the next: his body, the placards and the new stats.
public static class Brothers
{
    // Where the raft and the actor in slot 2 start.
    private const ushort RAFT_START_X = 13668;
    private const ushort RAFT_START_Y = 14470;
    private const ushort SETFIG_START_X = 13668;
    private const ushort SETFIG_START_Y = 15000;

    // Where the first brother starts, and the region it is in.
    internal const ushort FIRST_SAFE_SPOT_X = 19036;
    internal const ushort FIRST_SAFE_SPOT_Y = 15755;
    internal const ushort FIRST_REGION = 3;

    /// The number of inventory slots that are goods (the rest are gold and arrows).
    internal const int GOLDBASE = 31;

    /// The last brother is number 3; one past that is the end of the game.
    public const short LAST_BROTHER = 3;

    // The brothers in order; their numbers go up with each fallen one.
    private const short JULIAN = 1;
    private const short PHILLIP = 2;
    private const short KEVIN = 3;

    // Placard text numbers: the first brother's, the second's, ... the game-over one, then the
    // follow-up placards that tell why the next brother sets out.
    private const int PLACARD_JULIAN = 0;
    private const int PLACARD_PHILLIP = 1;
    private const int PLACARD_PHILLIP_FOLLOW_UP = 2;
    private const int PLACARD_KEVIN = 3;
    private const int PLACARD_KEVIN_FOLLOW_UP = 4;
    private const int PLACARD_GAME_OVER = 5;

    // Event messages for setting out: always the first, then one for the second and third brother.
    private const int EVENT_SETTING_OUT = 9;
    private const int EVENT_PHILLIP_SETS_OUT = 10;
    private const int EVENT_KEVIN_SETS_OUT = 11;

    // Object list slots: where the fallen brother's person entry is and the spectre.
    private const int DEAD_BROTHER_PERSON = 2;
    private const int SPECTRE_SLOT = 9;
    private const int INSIDE_OBJECTS = 8;
    private const sbyte PERSON = 3;
    private const sbyte ON_GROUND = 1;

    /// Place message number: "You are in..." for the start village.
    private const ushort START_PLACE = 2;

    // Delays, in 50ths of a second.
    private const int SHOW_PLACARD = 120;
    private const int SHORT_PAUSE = 80;
    private const int TINY_PAUSE = 10;
    private const int GAME_OVER_PAUSE = 500;

    // Sprite file for the hero and its set number at the start.
    private const short HERO_SPRITE_FILE = 6;
    private const short HERO_SET_FILE = 13;

    /// Puts the raft and the first person back where they start.
    public static void ResetRaftSetfig(Actors actors)
    {
        PlaceAtStart(actors.List[1], ActorKind.RAFT, RAFT_START_X, RAFT_START_Y);
        actors.List[1].Environ = 0;

        PlaceAtStart(actors.List[2], ActorKind.SETFIG, SETFIG_START_X, SETFIG_START_Y);
    }

    /// The fallen brother's body is left where he died, and his ghost appears.
    public static void LeaveFallenBrother(ObjectLists objects, Hero hero)
    {
        if (hero.Brother > 0 && hero.Brother < KEVIN)
        {
            objects.Global[hero.Brother].Xc = hero.X;
            objects.Global[hero.Brother].Yc = hero.Y;
            objects.Global[hero.Brother].ObStat = ON_GROUND;
            objects.Global[hero.Brother + DEAD_BROTHER_PERSON].ObStat = PERSON;
        }

        objects.Regions[INSIDE_OBJECTS][SPECTRE_SLOT].ObStat = PERSON;
    }

    /// The next brother takes over: his stats and item list, nothing in the timers, and the first
    /// safe spot.
    public static void TakeOverNextBrother(Data.Game game)
    {
        Bro stats = Tables.BrotherStats[game.Hero.Brother];

        StartNextBrother(game.Hero, game.Actors.List[0], stats);
        ResetForNextBrother(game.Hero, game.World);
        MapCamera.MoveCameraTo(game.World, game.Hero.SafeX, game.Hero.SafeY);

        game.Ui.ViewStatus = ViewStatus.REBUILD_ALL;
        game.Actors.OnScreen = 1;
        game.Actors.Loading = 0;
    }

    /// No spells running, and the next brother starts from the first safe spot in the first region.
    internal static void ResetForNextBrother(Hero hero, World world)
    {
        world.SecretTimer = 0;
        world.LightTimer = 0;
        world.FreezeTimer = 0;
        hero.SafeX = FIRST_SAFE_SPOT_X;
        hero.SafeY = FIRST_SAFE_SPOT_Y;
        hero.SafeR = FIRST_REGION;
        world.Region = FIRST_REGION;
    }

    /// The next brother takes over: his starting stats, an empty pack except for a dirk
    /// (inventory slot 0, which he also holds), and he counts as the next brother.
    internal static void StartNextBrother(Hero hero, Shape body, Bro next)
    {
        hero.Brave = next.Brave;
        hero.Luck = next.Luck;
        hero.Kindness = next.Kind;
        hero.Wealth = next.Wealth;
        hero.Brother++;

        byte[] stuff = hero.Stuff;

        Array.Clear(stuff, 0, GOLDBASE); // has no stuff
        stuff[0] = 1; // okay, a dirk, then
        body.Weapon = 1;
    }

    /// The first brother's opening story page on its own, as the original shows it when a game
    /// starts but without the wait after it (for the launch menu's Show Prologue).
    public static void ShowOpeningPlacard(Data.Game game, IOutput output)
    {
        MessagePage.MapMessage(game, output);
        output.UseStoryFont();
        TextRoutines.PlacardText(output, PLACARD_JULIAN);
        output.ClearDrawingPage();
        output.DrawPlacardBorder();
    }

    /// The story pages between brothers (and the game over page after the last).
    public static void ShowBrotherPlacards(Data.Game game, IOutput output)
    {
        MessagePage.MapMessage(game, output);
        output.UseStoryFont();

        if (game.Hero.Brother == JULIAN && game.Experience != Experience.Original)
        {
            output.ClearDrawingPage(); // Remastered starts the game without the opening placard

            return;
        }

        if (game.Hero.Brother == JULIAN)
        {
            TextRoutines.PlacardText(output, PLACARD_JULIAN);
            output.ClearDrawingPage();
        }
        else if (game.Hero.Brother == PHILLIP)
        {
            TextRoutines.PlacardText(output, PLACARD_PHILLIP);
        }
        else if (game.Hero.Brother == KEVIN)
        {
            TextRoutines.PlacardText(output, PLACARD_KEVIN);
        }
        else
        {
            TextRoutines.PlacardText(output, PLACARD_GAME_OVER);
        }

        output.DrawPlacardBorder();
        Display.Delay(game, output, SHOW_PLACARD);

        if (game.Hero.Brother > LAST_BROTHER)
        {
            game.Ui.QuitRequested = 1;
            Display.Delay(game, output, GAME_OVER_PAUSE);
        }
        else if (game.Hero.Brother > JULIAN)
        {
            ShowFollowUpPlacard(game, output);
        }
    }

    /// Loads the brother's sprites and tells where the story starts.
    public static void AnnounceBrother(Data.Game game, IOutput output)
    {
        output.UsePanelFont();
        output.SelectMessageArea();

        game.Actors.LoadedActorFile = HERO_SPRITE_FILE;
        game.Actors.LoadedSetfigFile = HERO_SET_FILE;
        SpriteLoading.LoadStartSprites(game, output);

        if (game.Hero.Brother > LAST_BROTHER)
        {
            return;
        }

        MessagePage.MessageOff(game, output);
        game.Hero.Place = START_PLACE;
        TextRoutines.ShowEvent(game, output, EVENT_SETTING_OUT);

        if (game.Hero.Brother == JULIAN)
        {
            output.PrintCont(".");
        }
        else if (game.Hero.Brother == PHILLIP)
        {
            TextRoutines.ShowEvent(game, output, EVENT_PHILLIP_SETS_OUT);
        }
        else if (game.Hero.Brother == KEVIN)
        {
            TextRoutines.ShowEvent(game, output, EVENT_KEVIN_SETS_OUT);
        }
    }

    // Puts one actor back at its start: its kind, its position and the first picture with no weapon.
    private static void PlaceAtStart(Shape actor, sbyte kind, ushort x, ushort y)
    {
        actor.Kind = kind;
        actor.AbsX = x;
        actor.AbsY = y;
        actor.Index = 0;
        actor.Weapon = 0;
    }

    // After the first story page the panel is cleared and the follow-up page for this brother is shown.
    private static void ShowFollowUpPlacard(Data.Game game, IOutput output)
    {
        Display.Delay(game, output, SHORT_PAUSE);
        output.ClearPlacard();
        Display.Delay(game, output, TINY_PAUSE);
        TextRoutines.PlacardText(output, game.Hero.Brother == PHILLIP ? PLACARD_PHILLIP_FOLLOW_UP : PLACARD_KEVIN_FOLLOW_UP);
        Display.Delay(game, output, SHOW_PLACARD);
    }
}
