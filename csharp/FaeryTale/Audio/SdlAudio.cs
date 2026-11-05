using System.Runtime.InteropServices;
using FaeryTale.Game.Ui;
using static SDL2.SDL;

namespace FaeryTale.Audio;

/// Opens the SDL audio device and connects it to an AudioEngine.
public static class SdlAudio
{
    // Kept in a field so the garbage collector does not free the delegate SDL calls.
    private static SDL_AudioCallback? callback;

    /// Starts the sound and makes it the game's player. Without an audio device the game stays
    /// silent and says so.
    public static void Start()
    {
        if (SDL_InitSubSystem(SDL_INIT_AUDIO) != 0)
        {
            Console.Error.WriteLine($"No audio: {SDL_GetError()}");

            return;
        }

        AudioEngine engine = new(AudioTables.SAMPLE_RATE);
        SDL_AudioSpec wanted = new()
        {
            freq = AudioTables.SAMPLE_RATE,
            format = AUDIO_F32SYS,
            channels = AudioTables.OUTPUT_CHANNELS,
            samples = AudioTables.BUFFER_SAMPLES,
            callback = callback = (_, stream, length) => Fill(engine, stream, length),
        };

        // This SDL2-CS version cannot take a null device name, so the first device is named.
        string? deviceName = SDL_GetNumAudioDevices(0) > 0 ? SDL_GetAudioDeviceName(0, 0) : null;

        if (deviceName is null)
        {
            Console.Error.WriteLine("No audio: no playback device");

            return;
        }

        if (SDL_OpenAudioDevice(deviceName, 0, ref wanted, out SDL_AudioSpec _, 0) is var device && device == 0)
        {
            Console.Error.WriteLine($"No audio: {SDL_GetError()}");

            return;
        }

        Music.Player = engine;
        SDL_PauseAudioDevice(device, 0);
    }

    // Mixes length bytes of 32 bit float samples into SDL's buffer.
    private static void Fill(AudioEngine engine, IntPtr stream, int length)
    {
        float[] samples = new float[length / sizeof(float)];

        engine.Mix(samples);
        Marshal.Copy(samples, 0, stream, samples.Length);
    }
}
