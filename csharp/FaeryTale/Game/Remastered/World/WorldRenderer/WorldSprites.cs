using FaeryTale.Game.Data;
using FaeryTale.Game.Render;

namespace FaeryTale.Game.Remastered;

/// WorldRenderer: the sprites.
public sealed partial class WorldRenderer
{
    // The sprites in the order the old drawing drew them (back to front).
    private void DrawSprites(uint[] frame, WorldView view, WorldCapture capture, Data.Game game, Video video, double alpha)
    {
        foreach (CapturedSprite sprite in capture.Sprites)
        {
            if (!interior.HidesPosition(sprite.ActorX, sprite.ActorY) && images.Sprite(video.Graphics, sprite.Sheet, sprite.Inum) is { } image)
            {
                DrawSprite(frame, view, game, video, sprite, image, alpha);
            }
        }
    }

    private void DrawSprite(uint[] frame, WorldView view, Data.Game game, Video video, CapturedSprite sprite, SpriteImage image, double alpha)
    {
        double left = sprite.Left + sprite.DeltaX * (1 - alpha);
        double topEdge = sprite.Top + sprite.DeltaY * (1 - alpha);
        int rows = Math.Min(sprite.Rows, image.Height - sprite.SkipRows);
        (double screenLeft, double screenTop) = view.ToScreen(left, topEdge);
        double scale = view.Scale;
        int x0 = Math.Max((int)Math.Floor(screenLeft), 0);
        int x1 = Math.Min((int)Math.Ceiling(screenLeft + image.Width * scale), view.Width);
        int y0 = Math.Max((int)Math.Floor(screenTop), 0);
        int y1 = Math.Min((int)Math.Ceiling(screenTop + rows * scale), view.Height);

        if (rows <= 0 || x0 >= x1 || y0 >= y1)
        {
            return;
        }

        int leftPixel = (int)Math.Floor(left);
        int topPixel = (int)Math.Floor(topEdge);
        SceneryHider hider = SceneryHider.For(game, video.Graphics, sprite, (leftPixel, topPixel), image.Width);

        for (int positionY = y0; positionY < y1; positionY++)
        {
            int rowIndex = (int)Math.Floor((positionY + PIXEL_CENTER - screenTop) / scale);

            if (rowIndex < 0 || rowIndex >= rows)
            {
                continue;
            }

            int imageRow = (rowIndex + sprite.SkipRows) * image.Width;

            for (int positionX = x0; positionX < x1; positionX++)
            {
                int column = (int)Math.Floor((positionX + PIXEL_CENTER - screenLeft) / scale);

                if (column < 0 || column >= image.Width || !image.Opaque[imageRow + column] || hider.Hides(leftPixel + column, topPixel + rowIndex))
                {
                    continue;
                }

                frame[positionY * view.Width + positionX] = palette[image.Pixels[imageRow + column]];
            }
        }
    }

    /// Hides the parts of a sprite that scenery (trees, walls, roofs) is in front of: the same rules
    /// and masks as the old drawing, worked out for each tile the sprite is over.
    private sealed class SceneryHider
    {
        private readonly int firstColumn;
        private readonly int firstRow;
        private readonly int columns;
        private readonly int[] maskOfCell;
        private readonly byte[] shadow;

        private SceneryHider(int firstColumn, int firstRow, int columns, int[] maskOfCell, byte[] shadow)
        {
            this.firstColumn = firstColumn;
            this.firstRow = firstRow;
            this.columns = columns;
            this.maskOfCell = maskOfCell;
            this.shadow = shadow;
        }

        public static SceneryHider For(Data.Game game, Graphics graphics, CapturedSprite sprite, (int X, int Y) corner, int width)
        {
            int ground = corner.Y + sprite.GroundY - sprite.Top;
            int firstColumn = corner.X >> TILE_WIDTH_SHIFT; // shift right = divide by tile width, giving the tile column
            int firstRow = corner.Y >> TILE_HEIGHT_SHIFT; // shift right = divide by tile height, giving the tile row
            int columns = ((corner.X + width - 1) >> TILE_WIDTH_SHIFT) - firstColumn + 1; // last pixel's tile column minus first, plus 1 = tiles covered
            int rows = ((corner.Y + Math.Max(sprite.Rows, 1) - 1) >> TILE_HEIGHT_SHIFT) - firstRow + 1; // same as columns, but vertically
            int[] masks = new int[Math.Max(columns, 1) * Math.Max(rows, 1)];

            Array.Fill(masks, -1);

            if (!sprite.InFront && sprite.Actor is { } actor)
            {
                for (int xm = 0; xm < columns; xm++)
                {
                    for (int ym = 0; ym < rows; ym++)
                    {
                        masks[ym * columns + xm] = MaskOfCell(game, actor, sprite.Slot, ground, firstColumn + xm, firstRow + ym, xm, ym);
                    }
                }
            }

            return new SceneryHider(firstColumn, firstRow, columns, masks, graphics.Shadow);
        }

        /// Whether the world pixel is hidden behind scenery.
        public bool Hides(int worldX, int worldY)
        {
            int column = (worldX >> TILE_WIDTH_SHIFT) - firstColumn; // pixel x to tile column, relative to the first covered column
            int rowIndex = (worldY >> TILE_HEIGHT_SHIFT) - firstRow; // pixel y to tile row, relative to the first covered row

            if (column < 0 || column >= columns || rowIndex < 0 || rowIndex * columns + column >= maskOfCell.Length || maskOfCell[rowIndex * columns + column] < 0)
            {
                return false;
            }

            int offset = maskOfCell[rowIndex * columns + column] * SHADOW_MASK_BYTES + (worldY & (WorldImages.TILE_HEIGHT - 1)) * SHADOW_ROW_BYTES + ((worldX & (WorldImages.TILE_WIDTH - 1)) >> BYTE_SHIFT_FOR_PIXELS); // sizes are powers of 2, so & (size-1) is pixel-in-tile; >> 3 = 8 pixels per byte

            return offset < shadow.Length && ((shadow[offset] >> (LAST_BIT_IN_BYTE - (worldX & LAST_BIT_IN_BYTE))) & 1) != 0; // pick this pixel's bit: bits are stored left pixel in the top bit
        }

        // The mask number of the scenery tile at a cell, or -1 when the tile does not hide the sprite.
        private static int MaskOfCell(Data.Game game, Shape actor, int slot, int ground, int tileColumn, int tileRow, int xm, int ym)
        {
            int tile = TileAt(game, tileColumn, tileRow);

            if (tile < 0)
            {
                return -1;
            }

            int entry = tile * TERRAIN_ENTRY_BYTES;
            short type = (short)(game.Map.TerrainByte(entry + 1) & TERRAIN_TYPE_MASK);

            if (actor.State == ActorState.FALL)
            {
                if (entry <= SceneryMasks.FALL_TILE_LIMIT)
                {
                    return -1;
                }

                type = FALLING_TERRAIN_TYPE;
            }

            bool onBridge = game.Hero.Sector == SceneryMasks.BRIDGE_SECTOR && slot != RAFT_SLOT;
            short feetAboveTile = (short)(ground - (tileRow << TILE_HEIGHT_SHIFT)); // shift left = multiply tile row by tile height, back to pixels

            switch (SceneryMasks.RuleFor(type, xm, ym, feetAboveTile, onBridge))
            {
                case SceneryMasks.SceneryRule.Skip:
                    return -1;

                case SceneryMasks.SceneryRule.UseFullMask:
                    entry = SceneryMasks.TILE_FULL_MASK;

                    break;
            }

            return game.Map.TerrainByte(entry);
        }
    }
}
