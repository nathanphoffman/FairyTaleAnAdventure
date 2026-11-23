namespace FaeryTale.Game.Render;

/// Where the clipped sprite starts and how big it is, in words and lines.
internal readonly record struct BlitExtent(sbyte Xbw, sbyte BlitWide, sbyte BlitHigh);
