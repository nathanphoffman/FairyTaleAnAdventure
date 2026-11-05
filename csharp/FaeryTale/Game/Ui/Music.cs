using FaeryTale.Game.Data;

namespace FaeryTale.Game.Ui;

/// What the music code needs from the sound player. The real player comes with the platform
/// phase; until then the game is silent.
public interface IAudioPlayer
{
    /// Starts the tune now.
    void PlayScore(int tune);

    /// Changes to the tune at the end of the current one.
    void SetScore(int tune);

    void StopScore();

    /// Gives the music driver its waveforms (8 of 128 bytes), volume envelopes (10 of 256 bytes)
    /// and instrument table.
    void LoadWaveforms(ushort[] instruments, byte[] waves, byte[] volumes);

    /// Plays sound effect num at the pitch speed.
    void PlayEffect(int num, int speed);

    /// Changes the waveform of an instrument slot.
    void SetInstrument(int slot, ushort wave);
}

/// Choosing the background music. The music depends on what the hero is doing: dying, in the
/// astral plane, fighting, indoors, or just walking by day or night.
public static class Music
{
    // Tune numbers.
    public const int TUNE_DAY = 0;
    public const int TUNE_BATTLE = 1;
    public const int TUNE_NIGHT = 2;
    public const int TUNE_ASTRAL = 4;
    public const int TUNE_INDOORS = 5;
    public const int TUNE_DEATH = 6;

    // The astral plane (a rectangle of the world).
    private const ushort ASTRAL_X_MIN = 0x2400;
    private const ushort ASTRAL_X_MAX = 0x3100;
    private const ushort ASTRAL_Y_MIN = 0x8200;
    private const ushort ASTRAL_Y_MAX = 0x8a00;

    // Indoors, the instrument changes: caves (region 9) use another wave from the dungeons.
    private const ushort INDOOR_REGION = 7;
    private const ushort CAVE_REGION = 9;
    private const int WAVE_SLOT = 10;
    private const ushort WAVE_CAVE = 0x0307;
    private const ushort WAVE_INDOORS = 0x0100;

    /// It is night when the light level is at or below this.
    private const ushort NIGHT_LIGHT = 120;

    // The game menu entry that switches music on and off.
    private const int GAME_MENU = 4;
    private const int MUSIC_ENTRY = 6;
    private const int SOUND_ENTRY = 7;

    /// Plays sound effect num at the pitch speed, unless sound effects are switched off.
    public static void PlayEffect(Data.Game game, short num, int speed)
    {
        if ((game.Menus[GAME_MENU].Enabled[SOUND_ENTRY] & 1) != 0)
        {
            Player.PlayEffect(num, speed);
        }
    }

    /// Where the music goes. Silent until a real player is set.
    public static IAudioPlayer Player { get; set; } = new SilentPlayer();

    /// Picks the tune for the situation and plays it right away if now is set, otherwise changes
    /// to it at the end of the current one. If music is switched off, stops the music.
    public static void SetMood(Data.Game game, sbyte now)
    {
        int tune = PickTune(game.Actors, game.World, game.Hero);

        if ((game.Menus[GAME_MENU].Enabled[MUSIC_ENTRY] & 1) == 0)
        {
            Player.StopScore();
        }
        else if (now != 0)
        {
            Player.PlayScore(tune);
        }
        else
        {
            Player.SetScore(tune);
        }
    }

    // The tune for what the hero is doing: dying, in the astral plane, fighting, indoors (which
    // also picks the instrument for houses or caves), or walking by day or by night.
    internal static int PickTune(Actors actors, World world, Hero hero)
    {
        if (actors.List[0].Vitality == 0)
        {
            return TUNE_DEATH;
        }

        if (hero.X > ASTRAL_X_MIN && hero.X < ASTRAL_X_MAX && hero.Y > ASTRAL_Y_MIN && hero.Y < ASTRAL_Y_MAX)
        {
            return TUNE_ASTRAL;
        }

        if (actors.BattleFlag != 0)
        {
            return TUNE_BATTLE;
        }

        if (world.Region > INDOOR_REGION)
        {
            Player.SetInstrument(WAVE_SLOT, world.Region == CAVE_REGION ? WAVE_CAVE : WAVE_INDOORS);

            return TUNE_INDOORS;
        }

        return world.LightLevel > NIGHT_LIGHT ? TUNE_DAY : TUNE_NIGHT;
    }

    private sealed class SilentPlayer : IAudioPlayer
    {
        public void PlayScore(int tune)
        {
        }

        public void SetScore(int tune)
        {
        }

        public void StopScore()
        {
        }

        public void LoadWaveforms(ushort[] instruments, byte[] waves, byte[] volumes)
        {
        }

        public void PlayEffect(int num, int speed)
        {
        }

        public void SetInstrument(int slot, ushort wave)
        {
        }
    }
}
