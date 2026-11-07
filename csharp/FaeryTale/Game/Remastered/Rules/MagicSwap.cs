namespace FaeryTale.Game.Remastered;

/// The bird totem is gone in Remastered (in Reimagined its place in the pack holds the inferno skull): all it did was show the map of the land, and
/// the minimap and the big map (M) do that now. Wherever the game would give the hero one (lying about,
/// in a chest, on a body, in the shop) the hero gets one of the other magic stones at random instead.
public static class MagicSwap
{
    /// The bird totem's place in the pack.
    public const int BIRD_TOTEM = 13;

    // The other magic stones: the blue stone, the green jewel, the glass vial, the crystal orb, the
    // gold ring and the jade skull.
    private static readonly int[] OTHER_STONES = [9, 10, 11, 12, 14, 15];

    /// The item to give for an item of the pack: a random other stone in place of the bird totem, and
    /// the same item for anything else (and always in the original).
    public static int Resolve(Data.Game game, int slot) =>
        game.Experience != Data.Experience.Remastered || slot != BIRD_TOTEM ? slot : OTHER_STONES[game.Rng.Rnd(OTHER_STONES.Length)];
}
