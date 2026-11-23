using System.Text;
using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Data;
using FaeryTale.Game.Input;
using FaeryTale.Game.Output;
using FaeryTale.Game.Ui.Options;

namespace FaeryTale.Game.Remastered;

/// One board of the bubble: its words, what it does, which Talk entry or which thing for sale it is,
/// and whether it can be used now (an item that costs too much is greyed out).
public readonly record struct Row(string Label, RowKind Kind, int Number, bool Enabled);
