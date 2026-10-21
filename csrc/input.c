/* input.c - keyboard and mouse input, replacing the Amiga input handler in fsubs.asm
 * (HandlerInterface, getkey, decode_mouse).
 *
 * On the Amiga an input.device handler received raw events and filled handler_data. Now the
 * Rust platform layer (src/platform/input.rs) translates SDL events and calls fta_* below.
 */
#include "ftale.h"

extern struct in_work handler_data;
extern short keydir, oldir;

/* Stand-in for the CIA port A register the game peeks at (0xbfe001). Bit 7 low = joystick fire. */
unsigned char fta_pia = 0xff;

static void push_key(int code)
{
	int next = handler_data.laydown;

	handler_data.keybuf[next] = (unsigned char)code;
	next = (next + 1) & 0x7f;
	if (next == handler_data.pickup) return;	/* buffer full, drop the key */
	handler_data.laydown = next;
}

/* Key event from the platform layer. `code` is the game's key code, bit 7 set = key released. */
void fta_key(int code)
{
	push_key(code);
}

/* Pops the next key code, 0 if none. */
int getkey(void)
{
	int d0 = 0;

	if (handler_data.pickup != handler_data.laydown)
	{	d0 = handler_data.keybuf[handler_data.pickup];
		handler_data.pickup = (handler_data.pickup + 1) & 0x7f;
	}
	return d0;
}

/* Pointer position in lores pixels (x 0-319) and screen lines (y 0-199). The pointer lives in
 * the text panel, so it is kept there. */
void fta_mouse_move(int x, int y)
{
	if (x > 315) x = 315;
	if (x < 5) x = 5;
	if (y > 195) y = 195;
	if (y < 147) y = 147;
	handler_data.xsprite = x;
	handler_data.ysprite = y;
}

/* Mouse button change. A left click in the menu column becomes a menu key ('a' + row, +1 for
 * the right-hand column) on press and the same key with bit 7 set on release. */
void fta_mouse_button(int left, int right)
{
	int newq = (left ? 0x4000 : 0) | (right ? 0x2000 : 0);
	int changed = (newq ^ handler_data.qualifier) & 0x4000;

	if (changed)
	{	int key = -1;
		if (newq & 0x4000)
		{	int x = handler_data.xsprite, y = handler_data.ysprite;
			if (x >= 215 && x <= 265 && y >= 144)
			{	key = ((y - 144) / 9) * 2 + 0x61;
				if (x >= 240) key++;
			}
		}
		else if (handler_data.lastmenu)
		{	key = (unsigned char)handler_data.lastmenu | 0x80;
			handler_data.lastmenu = 0;
		}
		if (key >= 0)
		{	push_key(key);
			/* as in the original: the last key sent is remembered, release included */
			handler_data.lastmenu = (char)key;
		}
	}
	handler_data.qualifier = newq;
}

/* Decides the walking direction (0-7, 9 = none) from the mouse over the on-screen compass or
 * from the keyboard direction keys, and redraws the compass when it changes. */
void decode_mouse(void)
{
	int dir;

	if (handler_data.qualifier & 0x6000)	/* a button is down */
	{	int x = handler_data.xsprite, y = handler_data.ysprite;
		int up, center, down;

		dir = 9;
		if (x > 265)
		{	if (x < 292) { up = 0; center = 7; down = 6; }
			else if (x <= 300) { up = 1; center = 9; down = 5; }
			else { up = 2; center = 3; down = 4; }
			dir = up;
			if (y >= 166)
			{	dir = down;
				if (y <= 174) dir = center;
			}
		}
	}
	else
	{	dir = keydir - 20;
		if (dir < 0 || dir >= 10) { dir = 9; keydir = 0; }
	}
	if (dir != oldir)
	{	oldir = dir;
		drawcompass(dir);
	}
}
