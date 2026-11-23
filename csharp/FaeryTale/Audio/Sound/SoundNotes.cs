namespace FaeryTale.Audio;

/// Sound: starting notes.
public sealed partial class Sound
{
    // Runs a non-note event. Returns false when the track ended.
    private bool RunCommand(int voice, byte command, byte value)
    {
        switch (command)
        {
            case AudioTables.CMD_INSTRUMENT:
                int index = value & AudioTables.INSTRUMENT_MASK;
                ushort word = index < Instruments.Length ? Instruments[index] : (ushort)0;

                // An instrument word holds the waveform number in its high byte and the volume envelope in the low one.
                Voices[voice].WaveNum = (byte)(word >> 8);
                Voices[voice].VolNum = (byte)word;

                break;

            case AudioTables.CMD_TEMPO:
                Tempo = value;

                break;

            case AudioTables.CMD_END:
                return EndOfTrack(voice, value != 0);
        }

        return true;
    }

    private void StartNote(int voice, byte command, byte value)
    {
        int length = AudioTables.NOTE_LENGTHS[value & AudioTables.NOTE_LENGTH_MASK];
        int stop = length - AudioTables.GAP;

        if (stop < 0)
        {
            stop = length;
        }

        Voice v = Voices[voice];

        v.EventStop = unchecked(v.EventStart + stop);
        v.EventStart = unchecked(v.EventStart + length);

        if (v.Stat != 0)
        {
            return; // sound effect has the channel
        }

        int envelope = v.VolNum * AudioTables.ENVELOPE_BYTES;
        byte startVolume = envelope < Volumes.Length ? Volumes[envelope] : (byte)0;

        v.VolList = envelope + 1;
        v.VolDelay = 0;

        if (command == AudioTables.CMD_REST)
        {
            Chans[voice].Volume = 0;
            v.EventStop = TimeClock;

            return;
        }

        StartWave(voice, command, startVolume);
    }

    // Loads the waveform for note command into the voice's channel and starts it.
    private void StartWave(int voice, byte command, byte startVolume)
    {
        (ushort period, ushort offset) = command < AudioTables.NOTES.Length ? AudioTables.NOTES[command] : AudioTables.FALLBACK_NOTE;
        int waveStart = Voices[voice].WaveNum * AudioTables.WAVE_BYTES + offset * AudioTables.OFFSET_UNIT;
        int words = AudioTables.WAVE_WORDS - offset;
        Channel channel = Chans[voice];

        channel.Data = new sbyte[words * 2];

        for (int index = 0; index < channel.Data.Length; index++)
        {
            int at = waveStart + index;

            channel.Data[index] = unchecked((sbyte)(at < Waves.Length ? Waves[at] : (byte)0));
        }

        channel.Looping = true;
        channel.Phase = 0;
        channel.Volume = Math.Min(startVolume, AudioTables.MAX_VOLUME);
        channel.Active = true;
        SetPeriod(voice, period);
    }
}
