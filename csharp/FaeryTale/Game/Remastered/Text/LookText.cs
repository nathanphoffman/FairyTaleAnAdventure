using FaeryTale.Game.Files;

namespace FaeryTale.Game.Remastered;

/// The lines Remastered and Reimagined show when the hero looks at something, kept in look_text.json
/// (three variations for each kind of thing) so they can be changed without touching the code.
public static class LookText
{
    private const string FILE_NAME = "look_text.json";

    private static readonly Lazy<Dictionary<string, string[]>> lines = new(() => EmbeddedJson.Read<Dictionary<string, string[]>>(FILE_NAME));

    /// One of the lines for a kind of thing, picked by a random draw (any number, it is wrapped).
    public static string Pick(string subject, int draw)
    {
        string[] variations = lines.Value[subject];

        return variations[draw % variations.Length];
    }
}
