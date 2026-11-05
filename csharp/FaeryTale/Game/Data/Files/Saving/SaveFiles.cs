namespace FaeryTale.Game.Files;

/// Reading and writing the save files (A.faery, B.faery ... one letter per menu choice). The
/// platform layer puts them in the saves folder. Showing messages and redrawing after a load is
/// the menu's job.
public static class SaveFiles
{
    private const char FIRST_SLOT_LETTER = 'A';
    private const string SAVE_EXTENSION = ".faery";

    /// The save file name for menu slot slot (0 = A.faery ...).
    public static string NameForSlot(short slot) => $"{(char)(FIRST_SLOT_LETTER + (byte)slot)}{SAVE_EXTENSION}";

    /// Writes the game to the save file name. Throws IOException if it cannot.
    public static void Write(Data.Game game, string name)
    {
        Transfer transfer = Transfer.ForSaving();

        GameTransfer.TransferAll(game, transfer);

        string path = Paths.Resolve(name, true) ?? throw new FileNotFoundException(name);

        File.WriteAllBytes(path, transfer.Finish());
    }

    /// Reads the game from the save file name. Throws IOException if it cannot.
    public static void Read(Data.Game game, string name)
    {
        string path = Paths.Resolve(name, false) ?? throw new FileNotFoundException(name);
        Transfer transfer = Transfer.ForLoading(File.ReadAllBytes(path));

        GameTransfer.TransferAll(game, transfer);
    }
}
