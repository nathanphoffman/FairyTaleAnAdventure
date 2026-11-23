namespace FaeryTale.Game.Remastered;

/// What kind of thing an item is, and so what a click on it does.
public enum ItemUse
{
    /// Nothing: it is carried (treasure and quest items, supplies).
    None,

    /// A weapon: clicking it takes it in hand.
    Equip,

    /// A magic stone: it is cast from the hotbar, not from the inventory.
    Cast,

    /// A key: clicking it tries it on the doors and chests round the hero.
    Unlock,

    /// A special item the Use menu knows (the sea shell, the sun stone).
    UseSpecial,
}
