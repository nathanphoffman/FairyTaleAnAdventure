using System.Text;
using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Data;
using FaeryTale.Game.Input;
using FaeryTale.Game.Output;
using FaeryTale.Game.Ui.Options;

namespace FaeryTale.Game.Remastered;

/// What a row of the bubble does: say something, open or close the list of things for sale, or buy one.
public enum RowKind
{
    Talk,
    Buy,
    Item,
}

/// One board of the bubble: its words, what it does, which Talk entry or which thing for sale it is,
/// and whether it can be used now (an item that costs too much is greyed out).
public readonly record struct Row(string Label, RowKind Kind, int Number, bool Enabled);

/// Talking to a person by clicking on them (Remastered and Reimagined): a bubble with their face
/// and the choices of the Talk menu, Yell, Say and Ask, and for a bartender Buy, which opens the
/// list of things for sale. It stays open while the hero is near the person and goes away on any
/// other click.
public static class Conversation
{
    private static readonly string[] TALK_LABELS = ["Yell", "Say", "Ask"];

    // The names of the things for sale are five letters each in the game's own table.
    private const int NAME_WIDTH = 5;
    private const int PRICE_WIDTH = 4;
    private const int LABEL_NAME_WIDTH = 7;

    private static int? personNumber;
    private static bool listOpen;
    private static ushort startRegion;

    /// The actor number of the person the bubble is open for, if it is open.
    public static int? Person => personNumber;

    /// Opens the bubble for a person.
    public static void Open(Data.Game game, int actorNumber)
    {
        personNumber = actorNumber;
        listOpen = false;
        startRegion = game.World.Region;
    }

    /// Takes the bubble away.
    public static void Close() => personNumber = null;

    /// The boards of the bubble from the top: the Talk choices, then Buy for a bartender and, once
    /// that is clicked, the things for sale under it.
    public static List<Row> Rows(Data.Game game)
    {
        List<Row> rows = [];

        for (int index = 0; index < TALK_LABELS.Length; index++)
        {
            rows.Add(new Row(TALK_LABELS[index], RowKind.Talk, TalkOptions.YELL + index, true));
        }

        if (personNumber is { } number && ShopOptions.IsBartender(game.Actors.List[number]))
        {
            rows.Add(new Row("Buy", RowKind.Buy, 0, true));
            rows.AddRange(listOpen ? ItemRows(game) : []);
        }

        return rows;
    }

    /// Where the bubble is for the person it is open for (the top left corner is all that moves).
    public static BubbleLayout Layout(Data.Game game)
    {
        Shape person = game.Actors.List[personNumber ?? 0];

        return BubbleLayout.For(WorldView.Current, (person.AbsX, person.AbsY), Rows(game).Count);
    }

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

    /// Does what a click on the bubble means. Returns false when the click was not on a board (the
    /// bubble is closed and the click is for the world as usual).
    public static bool TakeButtonClick(Data.Game game, IOutput output, InputHandler.Click click)
    {
        if (personNumber is not { } number)
        {
            return false;
        }

        List<Row> rows = Rows(game);

        if (!click.Native || Layout(game).RowAt(click.X, click.Y, rows) is not { } index)
        {
            Close();

            return false;
        }

        Use(game, output, number, rows[index]);

        return true;
    }

    // Does what a row says. Talking ends the conversation; opening the list and buying do not, so
    // several things can be bought one after another.
    private static void Use(Data.Game game, IOutput output, int number, Row row)
    {
        switch (row.Kind)
        {
            case RowKind.Talk:
                Close();
                TalkOptions.TalkToActor(game, output, number, (short)row.Number);
                break;

            case RowKind.Buy:
                listOpen = !listOpen;
                break;

            case RowKind.Item when row.Enabled:
                ShopOptions.BuyFromActor(game, output, number, row.Number);
                break;
        }
    }

    // The things for sale, each with its price; those the hero cannot pay for are not enabled.
    private static IEnumerable<Row> ItemRows(Data.Game game)
    {
        for (int item = 0; item < ShopOptions.ITEM_COUNT; item++)
        {
            short price = ShopOptions.PriceOf(item);
            string name = ObjectTables.Jtrans[item * 2] == MagicSwap.BIRD_TOTEM ? "Magic" : Encoding.Latin1.GetString(Tables.Label5, item * NAME_WIDTH, NAME_WIDTH).Trim();

            yield return new Row(name.PadRight(LABEL_NAME_WIDTH) + price.ToString().PadLeft(PRICE_WIDTH), RowKind.Item, item, ShopOptions.CanAfford(game.Hero.Wealth, price));
        }
    }
}
