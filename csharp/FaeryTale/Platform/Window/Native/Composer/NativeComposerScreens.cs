using FaeryTale.Game.Input;
using FaeryTale.Game.Remastered;
using FaeryTale.Game.Render;
using FaeryTale.Gfx;

namespace FaeryTale.Platform;

/// NativeComposer: conversation, inventory, journal and hotbar.
internal sealed partial class NativeComposer
{
    // The conversation bubble over the person the hero is talking to.
    private void DrawConversation(int width, Video video, (int X, int Y) pointer)
    {
        if (video.Capture.TalkingTo is not { } number || FaceOf(video, number) is not { } face)
        {
            return;
        }

        FaeryTale.Game.Data.Game game = video.Capture.Game!;
        BubbleLayout layout = Conversation.Layout(game);
        List<Row> rows = Conversation.Rows(game);
        Overlay bubble = ConversationPicture.Draw(face, rows, layout.RowAt(pointer.X, pointer.Y, rows));

        bubble.DrawAt(picture, width, new Overlay.Placement(layout.Left, layout.Top, layout.Scale));
    }

    // The face of the person with an actor number, from the sprite just drawn.
    private Image? FaceOf(Video video, int number)
    {
        CapturedSprite? sprite = video.Capture.Sprites.Find(found => found.Slot == number && found.Pass == 0);

        return sprite is not null && renderer.PictureOf(video, sprite) is { } sheetPicture
            ? FacePicture.Face(renderer.Palette, sheetPicture)
            : null;
    }

    // The inventory screen over the world, while it is open (the game goes on behind it).
    private void DrawInventory(int width, Video video)
    {
        if (!Inventory.IsOpen || video.Capture.Game is not { } game)
        {
            return;
        }

        InventoryPicture.Of(game, number => renderer.ObjectPicture(video, number), InputHandler.GridPointer)
            .DrawAt(picture, width, new Overlay.Placement(GridLeft, GridTop, Scale));
    }

    // The journal over the world, while it is open (the game goes on behind it).
    private void DrawJournal(int width)
    {
        if (Journal.IsOpen)
        {
            JournalPicture.Of(Journal.Entries()).DrawAt(picture, width, new Overlay.Placement(GridLeft, GridTop, Scale));
        }
    }

    // The hotbar: wood growing out of the text panel's board to each side, the weapons the hero carries
    // on the left and the magic stones on the right, each in a square with a black border.
    private void DrawHotbar(int width, Video video, (int X, int Y) pointer)
    {
        int height = picture.Length / width;

        if (video.Capture.Game is not { } game || HotbarLayout.For(width, height) is not { } geometry)
        {
            return;
        }

        List<int> weapons = Hotbar.Weapons(game);
        List<int> spells = Hotbar.Spells(game);

        DrawWoodStrip(width, height, geometry, HotbarLayout.StripWidth(geometry, weapons.Count, HotbarLayout.WEAPON_COLUMNS), geometry.BoardLeft, true);
        DrawWoodStrip(width, height, geometry, HotbarLayout.StripWidth(geometry, spells.Count, HotbarLayout.SPELL_COLUMNS), geometry.BoardRight, false);

        List<HotbarSlot> slots = HotbarLayout.Slots(geometry, weapons, spells);

        HotbarPicture.Of(game, geometry, slots, number => renderer.ObjectPicture(video, number), pointer).Draw(picture, width);
    }

    // The "< Back" button above the classic buttons while a submenu is open.
    private void DrawBackButton(int width, WorldCapture capture)
    {
        if (capture.Game is { } game && PanelBackButton.IsShown(game))
        {
            PanelBackButton.Of(PanelBackButton.Contains(InputHandler.GridPointer)).DrawAt(picture, width, new Overlay.Placement(GridLeft, GridTop, Scale));
        }
    }
}
