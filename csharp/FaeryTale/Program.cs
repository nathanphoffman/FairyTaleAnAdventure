using FaeryTale.Audio;
using FaeryTale.Game.Render;
using FaeryTale.Game.Startup;
using FaeryTale.Platform;
using FaeryTale.Testing;

if (args.Contains("--window-test"))
{
    return WindowTest.Run();
}

// The intro and the launch menu play unless you skip them: dotnet run -- --skip-launch
Launch.LaunchSkipped = args.Contains("--skip-launch");
Launch.RunGame(CreateDisplay(args));

return 0;

// A window and sound, except for the test runs (they set FTA_SHOT, FTA_STATE or FTA_FAST, or
// pass --headless), which use the screenless display.
static IDisplayDevice CreateDisplay(string[] args)
{
    bool headless = args.Contains("--headless")
        || Environment.GetEnvironmentVariable("FTA_SHOT") is not null
        || Environment.GetEnvironmentVariable("FTA_STATE") is not null
        || Environment.GetEnvironmentVariable("FTA_FAST") is not null;

    if (headless)
    {
        return new HeadlessDisplay(Launch.LaunchSkipped);
    }

    SdlDisplay display = new();

    SdlAudio.Start();

    return display;
}
