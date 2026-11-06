using FaeryTale.Game.Data;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Remastered;

/// The pictures of the items, taken from the game's object pictures and blown up, for the inventory
/// screen. Each is made once, in the colours of full daylight, so they look the same at night.
public static class ItemIcons
{
    /// Each picture pixel is this many overlay units across, so the icons are large.
    public const int ZOOM = 3;

    private const uint OPAQUE = 0xff00_0000;

    private static readonly uint[] palette = ObjectTables.InitialPageColors.Select(color => OPAQUE | Scene.Rgb4(color)).ToArray();
    private static readonly Dictionary<int, Image> made = [];
    private static readonly Dictionary<(int Slot, int Width), Image> fitted = [];

    /// The picture of an item of the pack (null for one that has no picture, such as gold), made from
    /// the object picture sprite. pictureOf gives the sprite (colour numbers, and which pixels are
    /// part of the figure) of an object picture number.
    public static Image? Of(int slot, Func<int, SpriteImage?> pictureOf)
    {
        if (made.TryGetValue(slot, out Image? icon))
        {
            return icon;
        }

        InvItem entry = Tables.ItemList[slot];

        if (entry.ImgHeight == 0 || pictureOf(entry.ImageNumber) is not { } sprite)
        {
            return null;
        }

        icon = Enlarge(sprite, entry.ImgOff, entry.ImgHeight);
        made[slot] = icon;

        return icon;
    }

    /// The picture of an item fitted to a width in pixels (each picture pixel a block of whole or part
    /// pixels, nearest first), for the small squares of the hotbar. Made once for each width.
    public static Image? Fitted(int slot, Func<int, SpriteImage?> pictureOf, int widthPixels)
    {
        if (fitted.TryGetValue((slot, widthPixels), out Image? cached))
        {
            return cached;
        }

        InvItem entry = Tables.ItemList[slot];

        if (entry.ImgHeight == 0 || widthPixels <= 0 || pictureOf(entry.ImageNumber) is not { } sprite)
        {
            return null;
        }

        int height = Math.Max(1, entry.ImgHeight * widthPixels / sprite.Width);
        uint[] pixels = new uint[widthPixels * height];

        for (int y = 0; y < height; y++)
        {
            for (int x = 0; x < widthPixels; x++)
            {
                int source = (entry.ImgOff + y * entry.ImgHeight / height) * sprite.Width + x * sprite.Width / widthPixels;

                pixels[y * widthPixels + x] = source < sprite.Opaque.Length && sprite.Opaque[source] ? palette[sprite.Pixels[source]] : 0;
            }
        }

        Image icon = new(0, 0, widthPixels, height, pixels);

        fitted[(slot, widthPixels)] = icon;

        return icon;
    }

    // The rows of the picture an item uses (the object pictures hold two items one over the other),
    // each pixel a block of ZOOM x ZOOM units.
    private static Image Enlarge(SpriteImage sprite, int firstRow, int rows)
    {
        int width = sprite.Width * ZOOM;
        int height = rows * ZOOM;
        uint[] pixels = new uint[width * height];

        for (int y = 0; y < height; y++)
        {
            for (int x = 0; x < width; x++)
            {
                int source = (firstRow + y / ZOOM) * sprite.Width + x / ZOOM;

                pixels[y * width + x] = source < sprite.Opaque.Length && sprite.Opaque[source] ? palette[sprite.Pixels[source]] : 0;
            }
        }

        return new Image(0, 0, width, height, pixels);
    }
}
