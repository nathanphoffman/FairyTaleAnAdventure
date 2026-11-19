# Start the game (intro, launch menu, then play). Esc quits.
run:
    dotnet run -c Release --project csharp/FaeryTale

# Start the game and skip the intro and launch menu.
play:
    dotnet run -c Release --project csharp/FaeryTale -- --skip-launch

# Run the unit tests.
test:
    dotnet test csharp/FaeryTale.slnx

# Build a self-contained release for one platform: linux-x64, win-x64 or osx-x64.
# The result is dist/<platform>/ with the game data next to the program.
build platform:
    dotnet publish csharp/FaeryTale -c Release -r {{platform}} --self-contained -p:PublishSingleFile=true -o dist/{{platform}}
    mkdir -p dist/{{platform}}/faery-tale-amiga
    cp -r faery-tale-amiga/game dist/{{platform}}/faery-tale-amiga/game

# Build releases for Linux, Windows and Mac.
build-all:
    just build linux-x64
    just build win-x64
    just build osx-x64
