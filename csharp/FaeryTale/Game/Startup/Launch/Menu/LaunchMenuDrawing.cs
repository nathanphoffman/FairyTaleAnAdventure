using FaeryTale.Game.Render;
using FaeryTale.Gfx;
using static FaeryTale.Gfx.MenuStyle;

namespace FaeryTale.Game.Startup;

/// LaunchMenu: the planks and the overlay.
internal static partial class LaunchMenu
{
    /// The wooden boards behind the heading, each option and Quit, made once. Each entry has a
    /// plain board and a lit one (for when the pointer is on it).
    private sealed record Planks(Image Heading, Image[][] Options, Image[] Quit, Image[] Book, Image[] Prologue);

    // Makes a board for each area of the menu.
    private static Planks MakePlanks(List<Block> blocks)
    {
        Image[][] options = blocks.Select((block, index) => Board(BlockArea(block, OPTIONS[index].Label), 21u + (uint)index)).ToArray();

        return new Planks(Board(HeadingArea(), 11)[0], options, Board(QuitArea(), 31), Board(BookArea(), 32), Board(PrologueArea(), 33));
    }

    // The heading, the options and Quit as text on wooden boards, the hovered one lit.
    private static Overlay MenuOverlay(List<Block> blocks, Planks planks, int? hovered)
    {
        Overlay overlay = new();

        overlay.Images.Add(planks.Heading);
        AddCentered(overlay, HEADING, HEADING_Y, HEADING_COLOR, TITLE_SCALE);

        for (int index = 0; index < blocks.Count; index++)
        {
            bool lit = hovered == index;
            Block block = blocks[index];

            overlay.Images.Add(planks.Options[index][lit ? 1 : 0]);
            AddCentered(overlay, OPTIONS[index].Label, block.TitleY, lit ? HOVER_COLOR : TITLE_COLOR, TITLE_SCALE);

            for (int number = 0; number < block.Lines.Count; number++)
            {
                AddCentered(overlay, block.Lines[number], block.TitleY + DESCRIPTION_OFFSET + number * LINE_SPACING, lit ? LIT_DESCRIPTION_COLOR : DESCRIPTION_COLOR, BODY_SCALE);
            }
        }

        bool quitLit = hovered == QUIT;

        overlay.Images.Add(planks.Quit[quitLit ? 1 : 0]);
        AddText(overlay, QUIT_LABEL, (QuitLeft(), QUIT_Y), quitLit ? HOVER_COLOR : TITLE_COLOR, TITLE_SCALE);
        AddReplayEntry(overlay, (planks.Book, BOOK_LABEL, BookArea().Left), hovered == BOOK);
        AddReplayEntry(overlay, (planks.Prologue, PROLOGUE_LABEL, PrologueArea().Left), hovered == PROLOGUE);

        return overlay;
    }
}
