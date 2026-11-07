using System.Reflection;
using System.Text.Json;

namespace FaeryTale.Game.Files;

/// Reading the game's data from the JSON files built into the program (the csproj embeds every
/// .json file under its own file name), so numbers and text can change without touching the code.
public static class EmbeddedJson
{
    // Encounter and the like keep their numbers in plain fields.
    private static readonly JsonSerializerOptions OPTIONS = new() { IncludeFields = true };

    /// Reads a built-in JSON file into a value; a missing or empty file is a loud error.
    public static T Read<T>(string fileName)
    {
        using Stream file = Assembly.GetExecutingAssembly().GetManifestResourceStream(fileName)
            ?? throw new InvalidOperationException($"{fileName} is missing from the program.");

        return JsonSerializer.Deserialize<T>(file, OPTIONS)
            ?? throw new InvalidOperationException($"{fileName} is empty.");
    }
}
