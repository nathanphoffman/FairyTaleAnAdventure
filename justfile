# Start the game (intro, launch menu, then play). Esc quits.
run:
    dotnet run -c Release --project csharp/FaeryTale

# Start the game and skip the intro and launch menu.
play:
    dotnet run -c Release --project csharp/FaeryTale -- --skip-launch

# Run the unit tests.
test:
    dotnet test csharp/FaeryTale.slnx
