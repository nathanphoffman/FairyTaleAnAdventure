using FaeryTale.Game;

namespace FaeryTale.Testing;

/// A game command to queue at a tick.
public readonly record struct TimedCommand(uint Tick, Command Command);
