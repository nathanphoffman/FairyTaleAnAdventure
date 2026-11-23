namespace FaeryTale.Game.Render;

/// A sprite cut to the screen, ready to blit.
internal sealed record Placed(Sprite Sprite, Clip Clip, sbyte BytesWide);
