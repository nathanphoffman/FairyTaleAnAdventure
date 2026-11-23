using FaeryTale.Game.Remastered;
using FaeryTale.Gfx;

namespace FaeryTale.Platform;

/// NativeComposer: the control panel.
internal sealed partial class NativeComposer
{
    /// Whether a window pixel is on the text panel (only while the world is shown).
    public bool IsOverPanel(int pixelX, int pixelY)
    {
        (int x, int y) = ToGrid(pixelX, pixelY);

        return ShowsWorld && x >= 0 && x < GRID_WIDTH && y >= PANEL_FIRST_ROW;
    }

    // A piece of the panel's wood joined to the board: strip pixels wide, ending at the board's edge on
    // the left, starting at the board's edge on the right. Made once for each size.
    private void DrawWoodStrip(int width, int height, HotbarGeometry geometry, int stripPixels, int edge, bool onLeft)
    {
        if (stripPixels == 0)
        {
            return;
        }

        int blocks = (stripPixels + Scale - 1) / Scale;
        int rows = (height - geometry.BoardTop + GRID_ROWS_PER_LINE * Scale - 1) / (GRID_ROWS_PER_LINE * Scale);

        if (!hotbarWood.TryGetValue((blocks, rows), out uint[]? wood))
        {
            wood = PanelLayout.BoardPixels(blocks, rows);
            hotbarWood[(blocks, rows)] = wood;
        }

        int left = onLeft ? edge - blocks * Scale : edge;

        for (int rowIndex = 0; rowIndex < rows; rowIndex++)
        {
            for (int block = 0; block < blocks; block++)
            {
                FillPanelBlock(width, height, (left + block * Scale, geometry.BoardTop + rowIndex * GRID_ROWS_PER_LINE * Scale), wood[rowIndex * blocks + block]);
            }
        }
    }

    // A red square round each classic button the new controls are using.
    private void DrawLitButtons(int width, WorldCapture capture)
    {
        if (capture.Game is { } game)
        {
            PanelButtonOutline.Of(game, capture.LitButtons).DrawAt(picture, width, new Overlay.Placement(GridLeft, GridTop, Scale));
        }
    }

    // The text panel, cut into its pieces and put together over the terrain (see PanelLayout): each game
    // pixel a Scale wide and 2 x Scale tall. Pixels the layout leaves empty let the terrain show.
    private void DrawPanel(int width)
    {
        int height = picture.Length / width;
        panelLayout.Prepare(legacyFrame, Scene.FRAME_WIDTH, PANEL_FIRST_LINE);
        DrawPanelWood(width, height);

        for (int line = 0; line < panelLayout.ArrangedLines; line++)
        {
            for (int positionX = 0; positionX < panelLayout.ArrangedWidth; positionX++)
            {
                uint color = panelLayout.Arranged[line * panelLayout.ArrangedWidth + positionX];

                if (color != 0)
                {
                    FillPanelBlock(width, height, (GridLeft + positionX * Scale, GridTop + (PANEL_FIRST_LINE - PanelLayout.TOP_PAD_LINES + line) * GRID_ROWS_PER_LINE * Scale), color);
                }
            }
        }
    }

    // The board of dark wood under the panel's pieces, a little wider than they are, reaching up past
    // the old panel's top and down to the bottom of the window.
    private void DrawPanelWood(int width, int height)
    {
        int boardColumns = panelLayout.ArrangedWidth + 2 * PanelLayout.PAD_SIDE;

        for (int line = 0; line < panelLayout.ArrangedLines; line++)
        {
            for (int positionX = 0; positionX < boardColumns; positionX++)
            {
                FillPanelBlock(width, height, (GridLeft + (positionX - PanelLayout.PAD_SIDE) * Scale, GridTop + (PANEL_FIRST_LINE - PanelLayout.TOP_PAD_LINES + line) * GRID_ROWS_PER_LINE * Scale), panelLayout.BoardPixel(positionX, line));
            }
        }
    }

    // One game pixel of the panel as a block of Scale x (2 x Scale) window pixels, cut to the window.
    private void FillPanelBlock(int width, int height, (int X, int Y) corner, uint color)
    {
        for (int positionY = Math.Max(corner.Y, 0); positionY < Math.Min(corner.Y + GRID_ROWS_PER_LINE * Scale, height); positionY++)
        {
            int left = Math.Max(corner.X, 0);
            int right = Math.Min(corner.X + Scale, width);

            if (right > left)
            {
                Array.Fill(picture, color, positionY * width + left, right - left);
            }
        }
    }
}
