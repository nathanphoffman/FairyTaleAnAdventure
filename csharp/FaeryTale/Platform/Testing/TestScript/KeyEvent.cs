using FaeryTale.Game;

namespace FaeryTale.Testing;

/// A key pressed (Down true) or released at a tick of 1/50 second.
public readonly record struct KeyEvent(uint Tick, string Name, bool Down);
