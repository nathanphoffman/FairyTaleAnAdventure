using System.Reflection;
using System.Text.Json;

namespace FaeryTale.Game.Remastered;

/// The lines Remastered and Reimagined show when the hero looks at something, kept in look_text.json
/// (three variations for each kind of thing) so they can be changed without touching the code.
public static class LookText
{
    private const string RESOURCE_NAME = "look_text.json";

    private static readonly Lazy<Dictionary<string, string[]>> lines = new(Read);

    /// One of the lines for a kind of thing, picked by a random draw (any number, it is wrapped).
    public static string Pick(string subject, int draw)
    {
        string[] variations = lines.Value[subject];

        return variations[draw % variations.Length];
    }

    // Reads the file that is built into the program.
    private static Dictionary<string, string[]> Read()
    {
        using Stream file = Assembly.GetExecutingAssembly().GetManifestResourceStream(RESOURCE_NAME)
            ?? throw new InvalidOperationException($"{RESOURCE_NAME} is missing from the program.");

        return JsonSerializer.Deserialize<Dictionary<string, string[]>>(file)
            ?? throw new InvalidOperationException($"{RESOURCE_NAME} is empty.");
    }
}
