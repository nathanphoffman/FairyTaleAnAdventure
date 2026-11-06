namespace FaeryTale.Game.Remastered;

/// The classic buttons of the text panel that a shortcut of the new controls stands for. While the
/// shortcut is in use the button gets a red square round it, so the two are seen to belong together.
[Flags]
public enum PanelButtons
{
    None = 0,

    /// Talking to someone (clicked on or walking up to).
    Talk = 1,

    /// Taking something (the hand pointer is over it, or the hero is walking up to it).
    Take = 2,

    /// Looking (shift is held, or the hero is walking to a spot to look).
    Look = 4,
}
