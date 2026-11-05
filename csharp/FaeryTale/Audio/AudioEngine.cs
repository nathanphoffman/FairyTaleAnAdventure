using FaeryTale.Game.Ui;

namespace FaeryTale.Audio;

/// The sound player the game talks to: plays tunes and effects through a Sound. The SDL device
/// (or a test) pulls samples out with Mix.
public sealed class AudioEngine : IAudioPlayer
{
    private readonly object padlock = new();
    private readonly Sound sound;

    public AudioEngine(double sampleRate) => sound = new Sound(sampleRate);

    /// Gives the music driver its waveforms, volume envelopes and instrument table.
    public void LoadWaveforms(ushort[] instruments, byte[] waves, byte[] volumes)
    {
        lock (padlock)
        {
            sound.InitMusic(instruments, waves, volumes);
        }
    }

    public void PlayScore(int tune)
    {
        lock (padlock)
        {
            sound.Play(SoundBank.TuneTracks(tune));
        }
    }

    public void SetScore(int tune)
    {
        lock (padlock)
        {
            sound.SetScore(SoundBank.TuneTracks(tune));
        }
    }

    public void StopScore()
    {
        lock (padlock)
        {
            sound.StopScore();
        }
    }

    public void SetInstrument(int slot, ushort wave)
    {
        lock (padlock)
        {
            sound.SetInstrument(slot, wave);
        }
    }

    public void PlayEffect(int num, int speed)
    {
        if (SoundBank.Sample(num) is not { } sample)
        {
            return;
        }

        lock (padlock)
        {
            sound.PlaySample(sample, speed);
        }
    }

    /// Mixes the next samples (interleaved left, right).
    public void Mix(Span<float> output)
    {
        lock (padlock)
        {
            sound.Mix(output);
        }
    }
}
