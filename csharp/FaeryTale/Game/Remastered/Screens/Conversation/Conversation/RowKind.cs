using System.Text;
using FaeryTale.Game.Creatures.Interaction;
using FaeryTale.Game.Data;
using FaeryTale.Game.Input;
using FaeryTale.Game.Output;
using FaeryTale.Game.Ui.Options;

namespace FaeryTale.Game.Remastered;

/// What a row of the bubble does: say something, open or close the list of things for sale, or buy one.
public enum RowKind
{
    Talk,
    Buy,
    Item,
}
