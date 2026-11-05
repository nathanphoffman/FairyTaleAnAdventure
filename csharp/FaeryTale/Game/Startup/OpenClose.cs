using FaeryTale.Game.Data;
using FaeryTale.Game.Files;
using FaeryTale.Game.Render;
using FaeryTale.Game.Ui;
using FaeryTale.Gfx;

namespace FaeryTale.Game.Startup;

/// Opening everything the game needs (the screen, fonts, memory, music and the disk image) and
/// closing it again.
public static class OpenClose
{
    // The error number ShowError shows when a start-up step fails.
    private const int ERROR_NO_FONT = 15;

    /// The story font's file.
    private const string STORY_FONT_FILE = "fonts/Amber/9";

    // Memory areas are given a little slack at the end: the sprite blitter reads a word past some of them.
    private const int AREA_SLACK = 128;
    private const string WAVEFORM_FILE = "v6";
    private const int WAVE_BUFFER_SIZE = 128 * 8;
    private const int VOLUME_BUFFER_SIZE = 10 * 256;
    private const int IMAGE_PLANE_SIZE = 16384;
    private const int IMAGE_SIZE = IMAGE_PLANE_SIZE * 5;
    private const int SHAPE_SIZE = 78000;
    private const int SHADOW_SIZE = 8192 + 4096; // background masks

    /// Where the saved backgrounds start in the blitter's work memory (after the shape queue).
    private const int BACKSAVE_OFFSET = 962;

    /// Opens everything the game needs. Returns 0, or the error number ShowError shows.
    public static int OpenAll(Data.Game game, Video video)
    {
        video.Screen.SetUp([StoryText.HINOR, StoryText.HIVAR]);

        if (Disk.Open() is var error && error != 0)
        {
            return error;
        }

        if (!OpenFonts(video.Screen))
        {
            return ERROR_NO_FONT;
        }

        video.Screen.Pages[0].BackSave = new Loc(Area.Queue, BACKSAVE_OFFSET);
        video.Screen.Pages[1].BackSave = new Loc(Area.Mask, BACKSAVE_OFFSET);
        AllocateGameMemory(game.Map, video.Graphics);
        video.Screen.PointerImage = Tables.PointerSpriteData;
        LoadMusicWaveforms();

        return 0;
    }

    /// Closes the disk image.
    public static void CloseAll() => Disk.Close();

    // Reads the waveforms and volume envelopes for the music from the file v6 (the waveforms,
    // then as many bytes that are skipped, then the envelopes) and gives them to the music
    // driver with the instrument table.
    private static void LoadMusicWaveforms()
    {
        string? path = Paths.Resolve(WAVEFORM_FILE, false);
        byte[] data = path is not null && File.Exists(path) ? File.ReadAllBytes(path) : [];
        byte[] waves = new byte[WAVE_BUFFER_SIZE];
        byte[] volumes = new byte[VOLUME_BUFFER_SIZE];

        Array.Copy(data, waves, Math.Min(data.Length, waves.Length));

        if (data.Length > 2 * WAVE_BUFFER_SIZE)
        {
            Array.Copy(data, 2 * WAVE_BUFFER_SIZE, volumes, 0, Math.Min(data.Length - 2 * WAVE_BUFFER_SIZE, volumes.Length));
        }

        ushort[] instruments = Tables.InstrumentWaves.Select(word => unchecked((ushort)word)).ToArray();

        Music.Player.LoadWaveforms(instruments, waves, volumes);
    }

    // Loads the story font; the panel uses the built-in one.
    private static bool OpenFonts(Screen screen)
    {
        string? path = Paths.Resolve(STORY_FONT_FILE, false);

        if (path is null)
        {
            return false;
        }

        screen.SetFont(FontKind.Panel, Font.Builtin);
        screen.SetFont(FontKind.Story, Font.Load(path) ?? Font.Builtin);

        return true;
    }

    // Makes the picture, sector, shape, shadow-mask and terrain areas.
    private static void AllocateGameMemory(MapData map, Graphics graphics)
    {
        graphics.Image = new byte[IMAGE_SIZE + AREA_SLACK];
        map.Allocate();
        graphics.Shape = new byte[SHAPE_SIZE + AREA_SLACK];
        graphics.Shadow = new byte[SHADOW_SIZE + AREA_SLACK];
    }
}
