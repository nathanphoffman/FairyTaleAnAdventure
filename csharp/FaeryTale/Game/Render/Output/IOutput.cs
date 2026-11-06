using FaeryTale.Game.Data;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Output;

/// What the rules ask of the screen. The rules say what should be shown (a line of text, new page
/// colours, the next frame) and an output decides how: the video one draws it, the recorder just
/// writes it down so tests can run without a screen.
public interface IOutput
{
    /// Starts a new line in the text panel with these characters (the old lines scroll up).
    void PrintBytes(ReadOnlySpan<byte> text);

    /// Adds these characters to the last line of the text panel.
    void PrintContBytes(ReadOnlySpan<byte> text);

    /// Adds a number to the last line, right aligned in length characters, plus a trailing space.
    void PrintNumber(int value, int length);

    /// Starts a new line in the text panel.
    void Print(string text) => PrintBytes(TextEncoding.ToBytes(text));

    /// Adds to the last line of the text panel.
    void PrintCont(string text) => PrintContBytes(TextEncoding.ToBytes(text));

    /// Makes these (12 bit, 0x0RGB) colours the page palette.
    void LoadPageColors(ReadOnlySpan<ushort> colors);

    /// Swaps the page being drawn with the one being shown.
    void SwapPages();

    /// Shows the page that is on screen now.
    void Present();

    /// Waits one tick of 1/50 second, showing the screen as it is now.
    void WaitFrame();

    /// How long a picture of an animation lasts, and showing the next one (see IDisplayDevice).
    double AnimationFrameMs { get; }

    void ShowAnimationFrame();

    /// Ticks waited since the game started.
    uint Ticks { get; }

    /// Sets the text and dark panels drawn over the picture (an empty overlay removes them).
    void SetOverlay(Overlay overlay);

    /// Forgets all sprite sheets: the next one loads at the start of sprite memory.
    void ForgetSprites();

    /// Makes the next sprite load replace the sheet in this slot and everything loaded after it.
    void RewindSpritesTo(byte slot);

    /// The sheet in this slot starts where the next sprite load goes.
    void StartSpriteSheet(byte slot);

    /// Starts loading sprite file number file into the next free sprite memory.
    void LoadSpriteFile(int file);

    /// Waits for the sprite load to finish and builds the masks of the sheet in this slot.
    void FinishSpriteLoad(short slot);

    /// Loads image bank bank (a quarter of the tile images, 5 planes) from disk block firstBlock.
    void LoadImageBank(int bank, short firstBlock);

    // ---- the text cursor and the story pages ----

    /// Moves the text position of the area being written to.
    void SetTextPosition(int x, int y);

    /// Writes in the story font from now on.
    void UseStoryFont();

    /// Writes in the font of the text panel from now on.
    void UsePanelFont();

    /// Makes the message area of the text panel the place Print and PrintCont write to.
    void SelectMessageArea();

    /// Clears the area of a story page to black and goes back to the story text colour.
    void ClearPlacard();

    /// Draws the decorative border around a story page.
    void DrawPlacardBorder();

    // ---- the menu in the text panel ----

    /// Draws menu entry slot (0-11, two columns) with a label, highlighted if selected, on a box of boxPen.
    void DrawMenuEntry(int slot, ReadOnlySpan<byte> label, bool selected, byte boxPen);

    /// Blanks menu entry slot: nothing is shown there.
    void BlankMenuEntry(int slot);

    // ---- whole screens and pictures ----

    /// Lights direction dir on the compass (9 and above light none).
    void DrawCompass(short dir);

    /// Draws the pictures of everything the hero holds on a black page.
    void DrawItemList(Hero hero);

    /// Draws the overview map of the whole world around the map position (mx, my) on the page being
    /// drawn, and says in world which sector is at its top left.
    void DrawOverviewMap(MapData map, World world, int mx, int my);

    /// Marks the hero's place on the overview map, if it is inside the map.
    void MarkOverviewMap((int X, int Y)? at);

    /// Clears the page being drawn to black.
    void ClearDrawingPage();

    /// Shows the page being drawn from its top left corner (no scrolling).
    void ResetPageScroll();

    /// Gets the play field ready for a full page message: draws on the page being drawn, in the
    /// message colour, and clears it.
    void PrepareMessagePage();

    /// Shows or hides the text panel.
    void SetPanelVisible(bool visible);

    /// Makes the play field 2 * x wide and 2 * y high, centred on the screen, and the text panel the
    /// height that is left over (the intro zoom).
    void SetZoomSize(int x, int y);

    /// Loads the picture file with this name onto the page being drawn.
    void LoadPictureToDrawingPage(string name);

    /// Makes these (12 bit, 0x0RGB) colours the text panel palette.
    void LoadPanelColors(ReadOnlySpan<ushort> colors);
}
