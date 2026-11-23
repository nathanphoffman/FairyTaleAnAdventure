namespace FaeryTale.Audio;

/// Sound: the voices and their tracks.
public sealed partial class Sound
{
    // Loops back to the start when asked to, otherwise stops the voice. False when stopped.
    internal bool EndOfTrack(int voice, bool looping)
    {
        Voice v = Voices[voice];

        if (looping && v.TrackStart is not null)
        {
            v.Loops++;
            v.Track = v.TrackStart;
            v.Pos = 0;

            return true;
        }

        v.Track = null;
        v.VolDelay = -1;
        Chans[voice].Volume = 0;

        return false;
    }

    private void ReplaceVoice(int index, Voice voice) => Voices[index] = voice;

    private void DoVoice(int voice)
    {
        Voice v = Voices[voice];

        if (v.Track is null)
        {
            return;
        }

        int now = TimeClock;

        if (unchecked(now - v.EventStart) >= 0)
        {
            NewNote(voice);
        }
        else if (v.Stat != 0)
        {
            // A sound effect is playing on this channel.
        }
        else if (unchecked(now - v.EventStop) >= 0)
        {
            Chans[voice].Volume = 0; // gap before the next note
        }
        else
        {
            FollowEnvelope(voice);
        }
    }

    // What an envelope reads as when it runs past the end of the volume list.
    private const byte END_OF_ENVELOPE = 0xff;

    // Steps the volume envelope of a sounding note.
    private void FollowEnvelope(int voice)
    {
        Voice v = Voices[voice];

        if (v.VolList is not { } at || v.VolDelay != 0)
        {
            return;
        }

        sbyte level = unchecked((sbyte)(at < Volumes.Length ? Volumes[at] : END_OF_ENVELOPE));

        if (level >= 0)
        {
            Chans[voice].Volume = (byte)level;
            v.VolList = at + 1;
        }
    }

    // Reads events until one makes a sound (or the track ends).
    private void NewNote(int voice)
    {
        while (true)
        {
            (byte command, byte value) = NextEvent(voice);

            if ((command & AudioTables.CMD_NOTE_FLAG) == 0 || command == AudioTables.CMD_REST)
            {
                StartNote(voice, command, value);

                return;
            }

            if (!RunCommand(voice, command, value))
            {
                return;
            }
        }
    }

    // The next (command, value) pair of the voice's track. Past the end of the data it is the
    // end-of-track command.
    private (byte Command, byte Value) NextEvent(int voice)
    {
        Voice v = Voices[voice];
        int offset = v.Pos;

        v.Pos += 2;

        return v.Track is { } track && offset + 2 <= track.Length ? (track[offset], track[offset + 1]) : (AudioTables.CMD_END, (byte)0);
    }
}
