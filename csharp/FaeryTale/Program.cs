using FaeryTale.Testing;

if (args.Contains("--window-test"))
{
    return WindowTest.Run();
}

Console.Error.WriteLine("The game is not ported yet. Run with --window-test to check SDL.");

return 2;
