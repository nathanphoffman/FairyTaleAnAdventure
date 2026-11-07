using static SDL2.SDL;

namespace FaeryTale.Platform;

/// Gamepad buttons and the left stick. Pad buttons act as the original's menu keys.
internal sealed class PadInput
{
    /// How far a stick must move (of 32767) before it counts as a direction.
    private const int STICK_DEAD_ZONE = 12000;

    private readonly List<IntPtr> pads = [];

    /// Left stick state: left, right, up, down.
    private readonly bool[] stick = new bool[4];

    public PadInput()
    {
        for (int index = 0; index < SDL_NumJoysticks(); index++)
        {
            Open(index);
        }
    }

    public void Handle(SDL_Event windowEvent)
    {
        switch (windowEvent.type)
        {
            case SDL_EventType.SDL_CONTROLLERDEVICEADDED:
                Open(windowEvent.cdevice.which);

                break;

            case SDL_EventType.SDL_CONTROLLERBUTTONDOWN:
            case SDL_EventType.SDL_CONTROLLERBUTTONUP:
                Button((SDL_GameControllerButton)windowEvent.cbutton.button, windowEvent.type == SDL_EventType.SDL_CONTROLLERBUTTONDOWN);

                break;

            case SDL_EventType.SDL_CONTROLLERAXISMOTION:
                Stick((SDL_GameControllerAxis)windowEvent.caxis.axis, windowEvent.caxis.axisValue);

                break;
        }
    }

    private void Open(int index)
    {
        if (SDL_IsGameController(index) == SDL_bool.SDL_TRUE && SDL_GameControllerOpen(index) is var pad && pad != IntPtr.Zero)
        {
            pads.Add(pad);
        }
    }

    // The key name each button sends (the names are the ones the key table knows).
    private static void Button(SDL_GameControllerButton button, bool down)
    {
        string? key = button switch
        {
            SDL_GameControllerButton.SDL_CONTROLLER_BUTTON_A => "Left Ctrl",
            SDL_GameControllerButton.SDL_CONTROLLER_BUTTON_X => "T",
            SDL_GameControllerButton.SDL_CONTROLLER_BUTTON_Y => "I",
            SDL_GameControllerButton.SDL_CONTROLLER_BUTTON_B => "?",
            SDL_GameControllerButton.SDL_CONTROLLER_BUTTON_LEFTSHOULDER => "U",
            SDL_GameControllerButton.SDL_CONTROLLER_BUTTON_RIGHTSHOULDER => "G",
            SDL_GameControllerButton.SDL_CONTROLLER_BUTTON_START => "Space",
            SDL_GameControllerButton.SDL_CONTROLLER_BUTTON_DPAD_UP => "Up",
            SDL_GameControllerButton.SDL_CONTROLLER_BUTTON_DPAD_DOWN => "Down",
            SDL_GameControllerButton.SDL_CONTROLLER_BUTTON_DPAD_LEFT => "Left",
            SDL_GameControllerButton.SDL_CONTROLLER_BUTTON_DPAD_RIGHT => "Right",
            _ => null,
        };

        if (key is not null)
        {
            InputBridge.KeyEvent(key, down);
        }
    }

    // Turns the left stick into the four arrow keys, reporting only changes.
    private void Stick(SDL_GameControllerAxis axis, short value)
    {
        (int first, string firstKey, string secondKey) = axis switch
        {
            SDL_GameControllerAxis.SDL_CONTROLLER_AXIS_LEFTX => (0, "Left", "Right"),
            SDL_GameControllerAxis.SDL_CONTROLLER_AXIS_LEFTY => (2, "Up", "Down"),
            _ => (-1, "", ""),
        };

        if (first < 0)
        {
            return;
        }

        Change(first, firstKey, value < -STICK_DEAD_ZONE);
        Change(first + 1, secondKey, value > STICK_DEAD_ZONE);
    }

    private void Change(int index, string key, bool on)
    {
        if (stick[index] != on)
        {
            stick[index] = on;
            InputBridge.KeyEvent(key, on);
        }
    }
}
