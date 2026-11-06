using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Data;
using FaeryTale.Game.Input;
using FaeryTale.Game.Output;
using FaeryTale.Game.Ui.Options;

namespace FaeryTale.Game.Remastered;

/// Talking to a person by clicking on them (Remastered and Reimagined): a bubble with their face
/// and the three choices of the Talk menu, Yell, Say and Ask. It stays open while the hero is
/// near the person and goes away on any other click.
public static class Conversation
{
    /// The names of the buttons, in the order of the Talk menu's entries from Yell.
    public static readonly string[] CHOICES = ["Yell", "Say", "Ask"];

    private static int? personNumber;
    private static ushort startRegion;

    /// The actor number of the person the bubble is open for, if it is open.
    public static int? Person => personNumber;

    /// Opens the bubble for a person.
    public static void Open(Data.Game game, int actorNumber)
    {
        personNumber = actorNumber;
        startRegion = game.World.Region;
    }

    /// Takes the bubble away.
    public static void Close() => personNumber = null;

    /// Closes the bubble when the person died, left, or is out of reach, or the hero went elsewhere.
    public static void CloseIfLost(Data.Game game)
    {
        if (personNumber is not { } number)
        {
            return;
        }

        bool gone = game.World.Region != startRegion || number >= game.Actors.Drawn || !PersonTargets.IsTalkable(game.Actors.List[number]);

        if (gone || Blocking.ActorDistance(game, number, 0) >= TalkOptions.TALK_RANGE)
        {
            Close();
        }
    }

    /// Does what a click on the bubble means. Returns false when the click was not on a button
    /// (the bubble is closed and the click is for the world as usual).
    public static bool TakeButtonClick(Data.Game game, IOutput output, InputHandler.Click click, BubbleLayout layout)
    {
        if (personNumber is not { } number)
        {
            return false;
        }

        Close();

        if (click.Native && layout.ButtonAt(click.X, click.Y) is { } button)
        {
            TalkOptions.TalkToActor(game, output, number, (short)(TalkOptions.YELL + button));

            return true;
        }

        return false;
    }
}
