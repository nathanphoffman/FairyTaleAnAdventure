using FaeryTale.Game.Data;
using FaeryTale.Game.Files;
using FaeryTale.Game.Input;
using FaeryTale.Game.Remastered;
using FaeryTale.Game.Render;
using FaeryTale.Game.Ui;
using FaeryTale.Platform;

namespace FaeryTale.Game.Startup;

/// Launch: the intro and the first brother.
public static partial class Launch
{
    // Remembers the chosen mode. Remastered and Reimagined let the pointer roam the whole screen.
    private static void ChooseExperience(Data.Game game, Video video, Experience experience)
    {
        game.Experience = experience;
        game.Difficulty = Difficulty.Chosen;
        InputHandler.RemasteredControls = experience != Experience.Original;

        // Remastered and Reimagined draw the world with the new world drawer at the window's own size.
        video.Capture.Enabled = experience != Experience.Original;
        WindowControls.NativeRendering = experience != Experience.Original;
    }

    // Points the text port at the panel with the story font and the paper colours for play.
    private static void SetUpStoryText(Screen screen)
    {
        screen.CurrentPort = PortId.Text;
        screen.GetPort(PortId.Text).Target = BitmapId.TextPlane0;
        screen.SetFontKind(PortId.Text, FontKind.Story);
        screen.SetAPen(PortId.Text, STORY_PEN);
        screen.SetBPen(PortId.Text, STORY_BACKGROUND_PEN);
    }

    // How long each pause on the title screen lasts, in ticks.
    private static int TitleWait() => LaunchSkipped ? TITLE_WAIT : LAUNCH_TITLE_WAIT;

    // Starts the intro tune, and flips the pages twice so both show the same thing.
    internal static void PrepareIntro(Data.Game game, Video video)
    {
        video.Screen.VpPage.Ras = 0;
        StartMenuMusic();
        video.Screen.LoadColors(ViewPortId.Text, Tables.BlackColors);
        video.Screen.Pages[1].Ras = 1;
        ScreenEffects.SetIntroZoom(game, video, 0);

        Display.FlipPages(game, video); // "prime the pump"
        Display.FlipPages(game, video);
    }

    // Switches to the normal playing screen: loads the shadow masks and the panel picture, and
    // shows the copyright placard.
    internal static void ShowPlayScreenBackground(Data.Game game, Video video)
    {
        video.Screen.Pages[1].Ras = 1;
        ClearBothPages(video.Screen);
        video.Screen.LoadColors(ViewPortId.Text, Tables.BlackColors);
        ScreenEffects.SetIntroZoom(game, video, ZOOM_NORMAL);
        SetPageColor(video.Screen, BACKGROUND_COLOR, LOADING_BLUE);

        Disk.LoadTrackRange(SHADOW_FIRST_BLOCK, SHADOW_BLOCKS, video.Graphics.Shadow);

        SetPageColor(video.Screen, BACKGROUND_COLOR, TITLE_BLUE);
        IffPicture.LoadPicture("hiscreen", video.Screen.GetBitmap(BitmapId.Text), 0, 0);
        SetPageColor(video.Screen, TEXT_COLOR, WHITE_RGB);

        video.Screen.CurrentPort = PortId.Map;
        video.Screen.GetPort(PortId.Map).Target = video.Screen.DrawingBitmapId;
        MessagePage.ShowStillScreen(game, video);

        // The copyright placard is a flash of text between the menu and the game: only the
        // original game has it.
        if (game.Experience == Experience.Original)
        {
            video.Screen.SetAPen(PortId.Map, TITLE_PEN);
            TextRoutines.PlacardText(video, PLACARD_COPYRIGHT);
        }

        InputHandler.ResetKeyBuffer();
    }

    // Starts the first brother and sets up the text page and the colours for play.
    internal static void StartFirstBrother(Data.Game game, Video video)
    {
        for (int ras = 0; ras < 2; ras++)
        {
            video.Screen.Ras[ras].X = 0;
            video.Screen.Ras[ras].Y = 0;
        }

        StopMusic();
        Creatures.Revive.Revival.Revive(game, video, 1);

        video.Screen.GetPort(PortId.Map).Target = video.Screen.DrawingBitmapId;
        video.Screen.VpText.Y = PAGE_HEIGHT;
        video.Screen.VpText.Height = TEXT_HEIGHT;
        video.Screen.VpPage.Height = PAGE_VIEW_HEIGHT;
        video.Screen.VpPage.X = PAGE_VIEW_X;
        video.Screen.VpPage.Width = LOWRES_WIDTH;
        video.Screen.VpPage.Y = 0;
        video.Screen.LoadColors(ViewPortId.Text, Tables.TextColors);

        foreach (FPage page in video.Screen.Pages)
        {
            page.ObCount = 0;
            page.WFlag = 0;
        }

        game.Ui.ViewStatus = VIEW_REDRAW;
        game.Ui.CurrentMenu = 0;
        Menus.PrintOptions(game, video);
    }
}
