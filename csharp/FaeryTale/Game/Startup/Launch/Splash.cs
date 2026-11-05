using FaeryTale.Game.Input;
using FaeryTale.Game.Render;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Startup;

/// The splash screens at the very start: a note from the remaster's author, then who made the
/// original game, what the copyright situation is (in David Joiner's own words, from the README
/// of his source release) and under which license this remaster is shared.
internal static class Splash
{
    /// One screen: a heading and paragraphs of text under it. A paragraph that starts with # is a
    /// small heading for the paragraphs after it.
    internal sealed record Page(string Heading, string[] Paragraphs);

    /// The words: Nathan Hoffman's note, then the legal details. The quoted sentences are David
    /// Joiner's (Talin's) own words from the README of his source release on GitHub; the rest is
    /// short and plain.
    internal static readonly Page[] PAGES =
    [
        new("A note from Nathan Hoffman",
        [
            "When I was just a few years old I would gaze over my sister's shoulder watching her play Faery Tale Adventure in the early-90s on one of my dad's Commodore Amiga computers. My childhood memories of the Amiga and this game would drive me into a programming career into a time in the early 2020s wondering how I might convert the game's original source code (released by David Joiner the creator) into a remastered version. Today, with the advent of AI I am at a special place and time that has allowed me to achieve that dream.",
            "I hope you enjoy,",
            "-Nathan Hoffman",
            "#About the license",
            "Fairy Tale: An Adventure is an adaptation by Nathan Hoffman of The Faery Tale Adventure, written by David Joiner (Talin) and published by MicroIllusions in 1987. In 2022 he sent a \"termination of transfer\" notice to Hollyware, Inc., the successors to MicroIllusions' rights, to reclaim his copyright, and has had no response. In his words: \"I cannot say for certain exactly what the copyright status of this code is. However, whatever rights I do have, I hereby make freely available under an MIT-style permissive license.\"",
            "Nathan Hoffman grants the MIT License on this adaptation, along with whatever rights the license above allows. The original game's art, music and data come from David Joiner's GitHub release, under his license above.",
        ]),
    ];

    // Everything here is in overlay pixels (640 x 400). The heading is drawn at the size of the
    // classic game text and the rest at half that.
    private const int HEADING_SCALE = 2;
    private const int BODY_SCALE = 1;

    // Where the text goes: the heading's baseline, the first body line, and the left margin. Lines
    // are this many characters wide, this far apart, with this much extra between paragraphs.
    private const int HEADING_Y = 40;
    private const int BODY_Y = 72;
    internal const int LINE_WIDTH = 76;
    private const int LINE_SPACING = 12;
    private const int PARAGRAPH_GAP = 8;

    // The bottom line of the screen holds the prompt.
    private const int PROMPT_X = 16;
    private const int PROMPT_Y = 390;

    // Colours of the heading, the text and the prompt.
    private const uint HEADING_COLOR = 0xffff_d060;
    private const uint TEXT_COLOR = Overlay.WHITE;
    private const uint PROMPT_COLOR = 0xffa0_a0c0;

    /// A page that nobody reacts to moves on by itself after this many ticks (40 seconds).
    private const int AUTO_ADVANCE_TICKS = 2000;

    /// The Return key's code.
    private const int RETURN_KEY = 13;

    /// Shows the pages one after another. Space, Return or a click goes to the next one, Escape
    /// skips the rest of the splash (the intro comes next). hint is the message for the bottom right.
    public static void Show(Data.Game game, Video video, string hint)
    {
        Launch.PrepareBluePage(game, video);
        InputHandler.ResetKeyBuffer();

        for (int number = 0; number < PAGES.Length; number++)
        {
            video.SetOverlay(PageOverlay(PAGES[number], number, hint));

            // Escape only leaves the splash; it takes a second press to skip the intro after it.
            if (WaitForPlayer(game, video))
            {
                break;
            }
        }

        video.SetOverlay(new Overlay());
    }

    /// A page as an overlay: the heading, the text, the prompt and the skip message.
    internal static Overlay PageOverlay(Page page, int number, string hint)
    {
        Overlay overlay = Overlay.CornerHint(hint);

        overlay.Texts.Add(new Text(page.Heading, (Overlay.WIDTH - Overlay.TextWidth(page.Heading, HEADING_SCALE)) / 2, HEADING_Y, HEADING_COLOR, HEADING_SCALE));
        overlay.Texts.AddRange(BodyLines(page));
        overlay.Texts.Add(new Text(Prompt(number), PROMPT_X, PROMPT_Y, PROMPT_COLOR, BODY_SCALE));

        return overlay;
    }

    /// The wrapped lines of a page's paragraphs, with their positions: a block of text in the
    /// middle of the screen, its lines starting at the same left edge.
    internal static List<Text> BodyLines(Page page)
    {
        int left = (Overlay.WIDTH - LINE_WIDTH * Overlay.GLYPH_SIZE * BODY_SCALE) / 2;
        List<Text> lines = [];
        int y = BODY_Y;

        foreach (string paragraph in page.Paragraphs)
        {
            bool small = paragraph.StartsWith('#');
            uint color = small ? HEADING_COLOR : TEXT_COLOR;

            foreach (string line in LaunchMenu.Wrap(small ? paragraph[1..] : paragraph, LINE_WIDTH))
            {
                lines.Add(new Text(line, left, y, color, BODY_SCALE));
                y += LINE_SPACING;
            }

            y += PARAGRAPH_GAP;
        }

        return lines;
    }

    // What the bottom line says: how to go on.
    private static string Prompt(int number) =>
        number + 1 == PAGES.Length ? "Space or click: continue" : $"Space: next page {number + 1}/{PAGES.Length}";

    // Waits until the player moves on, or the page has been up long enough. Returns true if the
    // player skipped the rest of the splash.
    private static bool WaitForPlayer(Data.Game game, Video video)
    {
        for (int tick = 0; tick < AUTO_ADVANCE_TICKS; tick++)
        {
            int key = InputHandler.NextKey();

            if (key == Launch.ESCAPE_KEY)
            {
                return true;
            }

            if (key == ' ' || key == RETURN_KEY)
            {
                return false;
            }

            if (InputHandler.LeftButtonDown)
            {
                // Wait for the button to go up, so one click does not turn two pages.
                while (InputHandler.LeftButtonDown)
                {
                    Display.Delay(game, video, 1);
                }

                return false;
            }

            Display.Delay(game, video, 1);
        }

        return false;
    }
}
