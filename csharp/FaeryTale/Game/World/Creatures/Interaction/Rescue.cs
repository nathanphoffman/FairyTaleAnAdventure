using FaeryTale.Game.Data;
using FaeryTale.Game.Land;
using FaeryTale.Game.Output;
using FaeryTale.Game.Render;
using FaeryTale.Game.Ui;

namespace FaeryTale.Game.Creatures.Interaction;

/// Rescuing the princess: the story page, the reward and the keys.
public static class Rescue
{
    // Each princess has three story placards, starting at 8 for the first; the closing two follow.
    private const int RESCUE_PLACARD_BASE = 8;
    private const int RESCUE_PLACARDS_EACH = 3;
    private const int AFTER_RESCUE_FIRST = 17;
    private const int AFTER_RESCUE_SECOND = 18;
    private const int RESCUE_PAUSE = 380;
    private const int CLEAR_PAUSE = 10;
    private const ushort PRINCESS_HOME_X = 5511;
    private const ushort PRINCESS_HOME_Y = 33780;
    private const short PRINCESS_EXTENT_X = 22205;
    private const short PRINCESS_EXTENT_Y = 21231;

    /// The object list of the interior regions.
    private const int INSIDE_OBJECTS = 8;

    // Entries of the interior object list that change when the princess is rescued.
    private const int PRINCESS_OBJECT = 2;
    private const sbyte PRINCESS_OBJECT_ID = 4;
    private const int CAPTIVE_OBJECT = 9;
    private const int PRINCESS_STUFF = 28;

    // Items 16 to 21 (the keys) get this many extra.
    private const int FIRST_KEY_ITEM = 16;
    private const int LAST_KEY_ITEM = 21;
    private const byte KEYS_REWARD = 3;
    private const int REWARD_MESSAGE = 18;
    private const short REWARD_GOLD = 100;

    /// The scene when a princess is rescued, and her reward.
    ///
    /// Shows the story pages, moves the hero to the princess's home, repositions the extent for
    /// the next princess, changes the princess object, gives 100 gold and three of each key, and
    /// bumps the count (each rescue shows different story pages).
    public static void RescuePrincess(Data.Game game, IOutput output)
    {
        MessagePage.MapMessage(game, output);
        output.UseStoryFont();
        ShowStoryPages(game, output);
        MessagePage.MessageOff(game, output);

        game.Hero.PrincessesRescued++;
        Places.TransferHero(game, output, PRINCESS_HOME_X, PRINCESS_HOME_Y, 0); // do princess transfer
        Extents.CenterExtentOn(game, 0, PRINCESS_EXTENT_X, PRINCESS_EXTENT_Y);
        game.Objects.Region(INSIDE_OBJECTS)[PRINCESS_OBJECT].ObId = PRINCESS_OBJECT_ID;
        game.Hero.Stuff[PRINCESS_STUFF] = 1;
        TextRoutines.Speak(game, output, REWARD_MESSAGE);
        game.Hero.Wealth = (short)(game.Hero.Wealth + REWARD_GOLD);
        game.Objects.Region(INSIDE_OBJECTS)[CAPTIVE_OBJECT].ObStat = 0;

        for (int item = FIRST_KEY_ITEM; item <= LAST_KEY_ITEM; item++)
        {
            game.Hero.Stuff[item] = (byte)(game.Hero.Stuff[item] + KEYS_REWARD);
        }
    }

    // The story placards of the rescue, then the closing ones.
    private static void ShowStoryPages(Data.Game game, IOutput output)
    {
        int line = game.Hero.PrincessesRescued * RESCUE_PLACARDS_EACH;

        TextRoutines.PlacardText(output, RESCUE_PLACARD_BASE + line);
        Messages.Name(game, output);
        TextRoutines.PlacardText(output, RESCUE_PLACARD_BASE + 1 + line);
        Messages.Name(game, output);
        TextRoutines.PlacardText(output, RESCUE_PLACARD_BASE + 2 + line);
        output.DrawPlacardBorder();
        Display.Delay(game, output, RESCUE_PAUSE);
        output.ClearPlacard();
        Display.Delay(game, output, CLEAR_PAUSE);
        TextRoutines.PlacardText(output, AFTER_RESCUE_FIRST);
        Messages.Name(game, output);
        TextRoutines.PlacardText(output, AFTER_RESCUE_SECOND);
        Display.Delay(game, output, RESCUE_PAUSE);
    }
}
