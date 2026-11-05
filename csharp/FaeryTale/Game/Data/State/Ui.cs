namespace FaeryTale.Game.Data;

/// The text printing queue: requests wait here until the screen code takes them.
public sealed class PrintQueue
{
    public sbyte[] Items = new sbyte[32];

    /// Where the next request is added, and where the next one is taken from.
    public short Write;
    public short Read;
}

/// The menus, the keyboard and mouse state, and test and debug switches.
public sealed class Ui
{
    public const int MENU_SLOTS = 12;

    /// The direction lit on the compass when none is.
    public const short NO_COMPASS_DIRECTION = 9;

    public short CurrentMenu;
    public sbyte ChosenEntry;
    public sbyte MenuHitEnabled;
    public short HeldDirection;
    public short FightKeyHeld;
    public short CheatEnabled;
    public sbyte QuitRequested;

    /// The player chose to leave the program (not just this game).
    public bool ExitProgram;

    /// What the screen is showing or must do next: 0 is the normal view, the other values (game
    /// over, fade in, redraw, items, map message, ...) are named where they are used.
    public sbyte ViewStatus;

    /// The file menu is for saving (else for loading).
    public bool Saving;
    public sbyte[] SlotEntries = new sbyte[MENU_SLOTS];
    public PrintQueue PrintQueue = new();
    public sbyte IntroSkipRequested;
    public short CompassLitDirection = NO_COMPASS_DIRECTION;

    /// The direction the compass picture shows now (the picture is only redrawn when this changes).
    public short CompassDrawnDirection = NO_COMPASS_DIRECTION;

    /// A click to move route is steering the hero this frame (Remastered and Reimagined only).
    public bool PathWalking;

    /// The direction (0-7, 9 = none) click to move is walking in; the compass shows it.
    public short PathDirection = NO_COMPASS_DIRECTION;
}

/// The witch and her spinning beam.
public sealed class Witch
{
    public sbyte Direction;
    public sbyte OnScreen;
    public byte Angle;

    /// Which side of the beam's two edges the hero is on.
    public short Side1;
    public short Side2;
}

/// Which version of the game the player chose in the launch menu.
public enum Experience
{
    Original,
    Remastered,
    Reimagined,
}
