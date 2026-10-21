/* asm_text.c - text, story and enemy-steering routines that were 68000 assembly
 * (fsubs.asm, narr.asm and inline #asm in fmain2.c), rewritten in C.
 * The story strings themselves are in narr_data.c (generated from narr.asm).
 */
#include "ftale.h"

extern struct RastPort *rp, rp_map;
extern struct shape anim_list[];
extern UBYTE *stuff;
extern char answr[];
extern short xx, yy;
extern char print_que[];
extern short prec, pplay;

extern char event_msg[], speeches[];
extern const char *const placard_msgs[], *const question_msgs[];

#define XY 128		/* in a string: followed by x/2 and y, moves the text position */

/* ---- numbers and plain text ---- */

/* 10 character buffer for numbers, plus one trailing space that is printed after them */
extern char numbuf[11];		/* defined in fmain.c */

/* ion(value): right-aligned decimal in numbuf[0..9], leading positions blank */
long ion(unsigned long v)
{
	int i = 9;

	do { numbuf[i--] = '0' + v % 10; v /= 10; } while (v && i >= 0);
	while (i >= 0) numbuf[i--] = ' ';
	return 0;
}

/* prdec(value, len): print the number in a field of len characters, plus the trailing space */
void prdec(long value, long len)
{
	ion(value);
	Text(rp, numbuf + 10 - len, len + 1);
}

void move(long x, long y) { Move(rp, x, y); }
void text(char *s, long len) { Text(rp, s, len); }

/* ssp(str): print a string that may hold XY escapes (128, x/2, y) to move the text position.
 * A string ends at a zero byte; bytes of 128 and above (XY) end each plain run of text. */
void ssp(unsigned char *s)
{
	while (*s)
	{	if (*s == XY)
		{	Move(rp, s[1] * 2, s[2]);
			s += 3;
		}
		else
		{	long n = 0;
			while (s[n] && s[n] < 128) n++;
			Text(rp, (char *)s, n);
			s += n;
			if (n == 0) s++;	/* stray byte >= 128 that is not XY */
		}
	}
}

void placard_text(long n) { ssp((unsigned char *)placard_msgs[n]); }
void question(long n) { print_cont((unsigned char *)question_msgs[n]); }

/* ---- messages: event(n) and speak(n) pick the n-th string from a list ---- */

static char *skip_strings(char *p, long n)
{
	while (n > 0) { while (*p++) ; n--; }
	return p;
}

void msg(char *start, long num) { extract(skip_strings(start, num)); }
void event(long n) { msg(event_msg, n); }
void speak(long n) { msg(speeches, n); }

/* prq(x): add a request to the print queue (ignored if the queue is full) */
void prq(long x)
{
	short next = (prec + 1) & 31;

	if (next == pplay) return;
	print_que[prec] = (char)x;
	prec = next;
}

/* cursor(len, pen): redraw the typed answer and a block cursor after it */
void cursor(long len, long pen)
{
	static char spaces[2] = "  ";

	SetBPen(rp, 0);
	Move(rp, xx, yy);
	Text(rp, answr, len);
	SetBPen(rp, pen);
	Text(rp, spaces, 1);
	SetBPen(rp, 0);
	Text(rp, spaces, 1);
}

/* ---- the decorative border drawn on story pages ---- */

static const signed char xmod[16] = { -4, -4, -4, 0, 0, 0, 4, 4, 0, -4, 0, 4, 4, 0, 0, 0 };
static const signed char ymod[16] = { 0, 0, 0, 4, 4, 4, 0, 0, -4, 0, -4, 0, 0, 4, 4, 4 };

void placard(void)
{
	struct RastPort *r = &rp_map;
	long xorg = 12, yorg = 0, dx, dy;
	int i, j, k;

	for (i = 16; i >= 0; i--)
	{	for (j = 0; j < 16; j++)
		{	dx = xorg + xmod[j];
			dy = yorg + ymod[j];
			for (k = 4; k >= 0; k--)
			{	SetAPen(r, k == 0 ? 24 : 1);
				if (i > 9)
				{	Move(r, xorg, yorg);
					Draw(r, dx, dy);
					Move(r, 284 - xorg, 124 - yorg);
					Draw(r, 284 - dx, 124 - dy);
				}
				Move(r, 16 + yorg, 12 - xorg);
				Draw(r, 16 + dy, 12 - dx);
				Move(r, 268 - yorg, 112 + xorg);
				Draw(r, 268 - dy, 112 + dx);
			}
			xorg = dx;
			yorg = dy;
		}
	}
}

/* ---- misc ---- */

/* stuff_flag(n): 10 if the player has item n, otherwise 8 (a menu item state) */
long stuff_flag(long n)
{
	return stuff[n] ? 10 : 8;
}

/* set_course(object, target_x, target_y, mode): point an actor toward (or away from) a target
 * and set it walking. Modes: 0 go toward, 1 and 2 toward with a sideways wobble when close,
 * 3 move away, 4 straight toward (no smart steering), 5 turn only (do not start walking),
 * 6 the target is a direction offset, not a position. */
void set_course(long object, long target_x, long target_y, long mode)
{
	static const unsigned char com2[9] = { 0, 1, 2, 7, 9, 3, 6, 5, 4 };
	struct shape *an = &anim_list[object];
	short xdif, ydif;
	unsigned short xabs, yabs;
	int xdir = 0, ydir = 0, deviation = 0, j;

	mode &= 0xff;
	if (mode == 6) { xdif = (short)target_x; ydif = (short)target_y; }
	else
	{	xdif = (short)(an->abs_x - target_x);
		ydif = (short)(an->abs_y - target_y);
	}
	xabs = (unsigned short)(xdif < 0 ? -xdif : xdif);
	yabs = (unsigned short)(ydif < 0 ? -ydif : ydif);
	if (xdif > 0) xdir = 1; else if (xdif < 0) xdir = -1;
	if (ydif > 0) ydir = 1; else if (ydif < 0) ydir = -1;

	if (mode != 4)
	{	if ((short)yabs < (short)(xabs >> 1)) ydir = 0;
		if ((short)xabs < (short)(yabs >> 1)) xdir = 0;
	}

	if (mode == 1 && (short)(xabs + yabs) < 40) deviation = 1;
	if (mode == 2 && (short)(xabs + yabs) < 30) deviation = 1;
	if (mode == 3) { xdir = -xdir; ydir = -ydir; }

	j = com2[(unsigned char)(4 - ydir - ydir - ydir - xdir)];
	if (j == 9) an->state = STILL;
	else
	{	if (rand() & 2) j += deviation; else j -= deviation;
		an->facing = j & 7;
		if (mode != 5) an->state = WALKING;
	}
}

/* do_error(n): show "ERROR n" on the text panel (startup failures) */
void do_error(long n)
{
	if (n == 2) return;	/* graphics not open, cannot print */
	move(200, 90);
	text("ERROR ", 6);
	prdec(n, 2);
}
