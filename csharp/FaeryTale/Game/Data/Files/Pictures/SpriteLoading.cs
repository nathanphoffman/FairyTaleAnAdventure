using FaeryTale.Game.Land;
using FaeryTale.Game.Output;

namespace FaeryTale.Game.Files;

/// Loading the sprite sheets of the hero, enemies, carriers and objects, and the pending region.
public static class SpriteLoading
{
    // Indexes into SpriteFiles.All that are always loaded together at the start.
    private const int OBJECTS_FILE = 3;
    private const int RAFT_FILE = 4;

    /// Loads the sprite sets the game starts with: objects, the current brother, the raft, the
    /// current monsters and set figures, then the region.
    public static void LoadStartSprites(Data.Game game, IOutput output)
    {
        output.ForgetSprites();
        output.LoadSpriteFile(OBJECTS_FILE);
        output.FinishSpriteLoad(SpriteFiles.OBJECTS);
        output.LoadSpriteFile(game.Hero.Brother - 1);
        output.FinishSpriteLoad(SpriteFiles.PHIL);
        output.LoadSpriteFile(RAFT_FILE);
        output.FinishSpriteLoad(SpriteFiles.RAFT);
        output.StartSpriteSheet(SpriteFiles.ENEMY);
        output.LoadSpriteFile(game.Actors.LoadedActorFile);
        output.FinishSpriteLoad(SpriteFiles.All[game.Actors.LoadedActorFile].SeqNum);
        output.LoadSpriteFile(game.Actors.LoadedSetfigFile);
        output.FinishSpriteLoad(SpriteFiles.SETFIG);
        game.World.NewRegion = game.World.Region;
        Regions.LoadRegionNow(game, output);
    }

    /// Loads the next region piece. In the original the disk was slow, so the game kept playing
    /// while a region loaded in the background and called this to check; now a read is always done.
    public static void LoadPendingRegion(Data.Game game, IOutput output) => Regions.LoadRegionPiece(game, output);
}
