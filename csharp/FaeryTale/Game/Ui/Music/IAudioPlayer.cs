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
