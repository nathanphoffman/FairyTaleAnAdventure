using FaeryTale.Game;

namespace FaeryTale.Testing;

/// A left click at a tick, at x in 320 wide lores pixels and y in screen lines.
public readonly record struct Click(uint Tick, int X, int Y);
