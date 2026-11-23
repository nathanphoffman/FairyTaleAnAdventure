using FaeryTale.Game.Data;
using FaeryTale.Game.Remastered;
using FaeryTale.Game.Render;
using FaeryTale.Game.Ui;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Startup;

/// Launch: music and what the player chose.
public static partial class Launch
{
    // Starts the menu's tune.
    private static void StartMenuMusic()
    {
        Music.Player.PlayScore(INTRO_TUNE);
        menuMusicPlaying = true;
    }

    // Stops whatever music is playing.
    private static void StopMusic()
    {
        Music.Player.StopScore();
        menuMusicPlaying = false;
    }

    // Shows the launch menu until the player picks a version to play (true) or Quit (false). The
    // book and the prologue are replayed on the way, and the menu comes back after each.
    private static bool AskWhatToDo(Data.Game game, Video video)
    {
        while (true)
        {
            if (!menuMusicPlaying)
            {
                StartMenuMusic();
            }

            (LaunchAction action, Experience experience) = LaunchMenu.Choose(game, video);

            switch (action)
            {
                case LaunchAction.Quit:
                    return false;

                case LaunchAction.ShowBook:
                    ReplayBook(game, video);

                    break;

                case LaunchAction.ShowPrologue:
                    ReplayPrologue(game, video);

                    break;

                default:
                    ChooseExperience(game, video, experience);

                    return true;
            }
        }
    }

    // The story book of the intro again (Space or Escape skips it).
    private static void ReplayBook(Data.Game game, Video video)
    {
        game.Ui.IntroSkipRequested = 0;
        PrepareBluePage(game, video);
        video.SetOverlay(Overlay.CornerHint(BOOK_HINT));
        PrepareIntro(game, video);
        StoryPages.PlayIntro(game, video);
        BackToBluePage(game, video);
    }

    // The text that flashes up when the original game starts: the copyright notice and the first
    // brother's story page, drawn the way the original game draws them. Each screen stays until the
    // player clicks (or presses a key); Escape leaves.
    private static void ReplayPrologue(Data.Game game, Video video)
    {
        game.Experience = Experience.Original;
        ShowPlayScreenBackground(game, video);

        if (WaitForClick(game, video))
        {
            Creatures.Revive.Brothers.ShowOpeningPlacard(game, video);
            WaitForClick(game, video);
        }

        BackToBluePage(game, video);
    }
}
