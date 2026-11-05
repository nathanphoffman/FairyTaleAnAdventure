using FaeryTale.Game.Output;

namespace FaeryTale.Game.Render;

/// Switching between the play view and the map page that shows messages.
public static class MessagePage
{
    // ViewStatus values: showing a map message, and showing the normal view again.
    private const sbyte VIEW_MESSAGE = 2;
    private const sbyte VIEW_NORMAL_AGAIN = 3;

    /// Shows the page without scrolling.
    public static void ShowStillScreen(Data.Game game, IOutput output)
    {
        output.ResetPageScroll();
        Display.FlipPages(game, output);
    }

    /// Switches to the map page for a message (the text panel is hidden) and fades in.
    public static void MapMessage(Data.Game game, IOutput output)
    {
        Fading.FadeDown(game, output);

        output.PrepareMessagePage();
        output.SetPanelVisible(false);
        ShowStillScreen(game, output);
        output.LoadPageColors(game.PageColors);

        game.Ui.ViewStatus = VIEW_MESSAGE;
    }

    /// Goes back from the map message to the normal view.
    public static void MessageOff(Data.Game game, IOutput output)
    {
        Fading.FadeDown(game, output);

        output.SelectMessageArea();
        output.SetPanelVisible(true);
        Display.FlipPages(game, output);
        game.Ui.ViewStatus = VIEW_NORMAL_AGAIN;
    }
}
