namespace FaeryTale.Game.Remastered;

/// What the drawn cursor icons share.
internal static class CursorPixels
{
    /// Every pixel of an icon, asking pixelAt for the colour at each column and row.
    public static uint[] Build(int width, int height, Func<int, int, uint> pixelAt)
    {
        uint[] result = new uint[width * height];

        for (int positionY = 0; positionY < height; positionY++)
        {
            for (int positionX = 0; positionX < width; positionX++)
            {
                result[positionY * width + positionX] = pixelAt(positionX, positionY);
            }
        }

        return result;
    }
}
