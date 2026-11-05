using FaeryTale.Game.Data;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Output;

/// An output that draws nothing and writes down what the rules asked for, so the rules can run
/// (and be tested) without a screen. It counts a tick for each WaitFrame.
public sealed class Recorder : IOutput
{
    private readonly List<Call> calls = new();
    private readonly List<string> printedLines = new();

    public IReadOnlyList<Call> Calls => calls;

    public uint Ticks { get; private set; }

    /// The lines started with PrintBytes, each with what PrintContBytes added.
    public IReadOnlyList<string> PrintedLines => printedLines;

    public void PrintBytes(ReadOnlySpan<byte> text)
    {
        string line = TextEncoding.FromBytes(text);

        printedLines.Add(line);
        Record("PrintBytes", Quote(line));
    }

    public void PrintContBytes(ReadOnlySpan<byte> text)
    {
        string added = TextEncoding.FromBytes(text);

        if (printedLines.Count > 0)
        {
            printedLines[^1] += added;
        }

        Record("PrintContBytes", Quote(added));
    }

    public void PrintNumber(int value, int length) => Record("PrintNumber", value, length);

    public void LoadPageColors(ReadOnlySpan<ushort> colors) => Record("LoadPageColors", Hex(colors));

    public void SwapPages() => Record("SwapPages");

    public void Present() => Record("Present");

    public void WaitFrame()
    {
        Ticks++;
        Record("WaitFrame");
    }

    public void SetOverlay(Overlay overlay) =>
        Record("SetOverlay", $"{overlay.Shades.Count} shades", $"{overlay.Images.Count} images", $"{overlay.Texts.Count} texts");

    public void ForgetSprites() => Record("ForgetSprites");

    public void RewindSpritesTo(byte slot) => Record("RewindSpritesTo", slot);

    public void StartSpriteSheet(byte slot) => Record("StartSpriteSheet", slot);

    public void LoadSpriteFile(int file) => Record("LoadSpriteFile", file);

    public void FinishSpriteLoad(short slot) => Record("FinishSpriteLoad", slot);

    public void LoadImageBank(int bank, short firstBlock) => Record("LoadImageBank", bank, firstBlock);

    public void SetTextPosition(int x, int y) => Record("SetTextPosition", x, y);

    public void UseStoryFont() => Record("UseStoryFont");

    public void UsePanelFont() => Record("UsePanelFont");

    public void SelectMessageArea() => Record("SelectMessageArea");

    public void ClearPlacard() => Record("ClearPlacard");

    public void DrawPlacardBorder() => Record("DrawPlacardBorder");

    public void DrawMenuEntry(int slot, ReadOnlySpan<byte> label, bool selected, byte boxPen) =>
        Record("DrawMenuEntry", slot, Quote(TextEncoding.FromBytes(label)), selected, boxPen);

    public void BlankMenuEntry(int slot) => Record("BlankMenuEntry", slot);

    public void DrawCompass(short dir) => Record("DrawCompass", dir);

    public void DrawItemList(Hero hero) => Record("DrawItemList");

    public void DrawOverviewMap(MapData map, World world, int mx, int my) => Record("DrawOverviewMap", mx, my);

    public void MarkOverviewMap((int X, int Y)? at) =>
        Record("MarkOverviewMap", at is { } place ? $"({place.X}, {place.Y})" : "none");

    public void ClearDrawingPage() => Record("ClearDrawingPage");

    public void ResetPageScroll() => Record("ResetPageScroll");

    public void PrepareMessagePage() => Record("PrepareMessagePage");

    public void SetPanelVisible(bool visible) => Record("SetPanelVisible", visible);

    public void SetZoomSize(int x, int y) => Record("SetZoomSize", x, y);

    public void LoadPictureToDrawingPage(string name) => Record("LoadPictureToDrawingPage", Quote(name));

    public void LoadPanelColors(ReadOnlySpan<ushort> colors) => Record("LoadPanelColors", Hex(colors));

    private void Record(string name, params object[] arguments) =>
        calls.Add(new Call(name, string.Join(", ", arguments)));

    private static string Quote(string text) => $"\"{text}\"";

    private static string Hex(ReadOnlySpan<ushort> colors)
    {
        string[] words = new string[colors.Length];

        for (int index = 0; index < colors.Length; index++)
        {
            words[index] = $"0x{colors[index]:X4}";
        }

        return $"[{string.Join(", ", words)}]";
    }
}
