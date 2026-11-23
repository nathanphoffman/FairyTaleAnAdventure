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
        double top = sprite.Top + sprite.DeltaY * (1 - alpha);
        int rows = Math.Min(sprite.Rows, image.Height - sprite.SkipRows);
        (double screenLeft, double screenTop) = view.ToScreen(left, top);
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
        int topPixel = (int)Math.Floor(top);
        SceneryHider hider = SceneryHider.For(game, video.Graphics, sprite, (leftPixel, topPixel), image.Width);

        for (int y = y0; y < y1; y++)
        {
            int row = (int)Math.Floor((y + 0.5 - screenTop) / scale);

            if (row < 0 || row >= rows)
            {
                continue;
            }

            int imageRow = (row + sprite.SkipRows) * image.Width;

            for (int x = x0; x < x1; x++)
            {
                int column = (int)Math.Floor((x + 0.5 - screenLeft) / scale);

                if (column < 0 || column >= image.Width || !image.Opaque[imageRow + column] || hider.Hides(leftPixel + column, topPixel + row))
                {
                    continue;
                }

                frame[y * view.Width + x] = palette[image.Pixels[imageRow + column]];
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
            int firstColumn = corner.X >> TILE_WIDTH_SHIFT;
            int firstRow = corner.Y >> TILE_HEIGHT_SHIFT;
            int columns = ((corner.X + width - 1) >> TILE_WIDTH_SHIFT) - firstColumn + 1;
            int rows = ((corner.Y + Math.Max(sprite.Rows, 1) - 1) >> TILE_HEIGHT_SHIFT) - firstRow + 1;
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
            int column = (worldX >> TILE_WIDTH_SHIFT) - firstColumn;
            int row = (worldY >> TILE_HEIGHT_SHIFT) - firstRow;

            if (column < 0 || column >= columns || row < 0 || row * columns + column >= maskOfCell.Length || maskOfCell[row * columns + column] < 0)
            {
                return false;
            }

            int at = maskOfCell[row * columns + column] * SHADOW_MASK_BYTES + (worldY & (WorldImages.TILE_HEIGHT - 1)) * SHADOW_ROW_BYTES + ((worldX & (WorldImages.TILE_WIDTH - 1)) >> 3);

            return at < shadow.Length && ((shadow[at] >> (7 - (worldX & 7))) & 1) != 0;
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

                type = 3;
            }

            bool onBridge = game.Hero.Sector == SceneryMasks.BRIDGE_SECTOR && slot != RAFT_SLOT;
            short feetAboveTile = (short)(ground - (tileRow << TILE_HEIGHT_SHIFT));

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
