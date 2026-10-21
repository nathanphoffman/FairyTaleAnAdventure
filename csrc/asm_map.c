/* asm_map.c - the map routines that were 68000 assembly in fsubs.asm, rewritten in C.
 * Behaviour (including 16 bit wrap-around) follows the assembly. Each function names its
 * original. See docs/data-formats.md "Map data layout".
 *
 * World coordinates are 15 bit. A tile ("image") is 16 pixels wide and 32 tall; a sector is
 * 16 x 8 tiles; the region map is 128 x 32 sectors, one byte each.
 */
#include <string.h>
#include "ftale.h"

extern unsigned short xreg, yreg;
extern unsigned short map_x, map_y, hero_x, hero_y, region_num;
extern short hero_sector;
extern unsigned char *map_mem, *sector_mem, *terra_mem, *image_mem;
extern short minimap[];
extern PLANEPTR *planes;

#define IPLAN_SZ 16384		/* one bitplane of 256 tiles, 64 bytes per tile */
#define ROW_BYTES 40		/* bytes per scan line of the play field bitmaps */

/* sector column from an image column: values outside 0-63 wrap, as in the assembly */
static unsigned short wrap_secx(unsigned short v)
{
	if (v & 64) v = (v & 32) ? 0 : 63;
	return v;
}

/* sector row from an image row, rows outside 0-31 are clamped */
static unsigned short clamp_secy(short v)
{
	if (v < 0) v = 0;
	if (v >= 32) v = 31;
	return (unsigned short)v;
}

/* px_to_im(x, y): terrain type (0-15) at a pixel position, 0 if that sub-area is clear. */
int px_to_im(int x, int y)
{
	unsigned short ux = (unsigned short)x, uy = (unsigned short)y;
	unsigned char bit = 0x80, sec, tile;
	unsigned short imx, imy, secx, secy;

	if (ux & 8) bit >>= 4;
	if (uy & 8) bit >>= 1;
	if (uy & 16) bit >>= 2;

	imx = ux >> 4;
	imy = uy >> 5;
	secx = wrap_secx((unsigned short)((imx >> 4) - xreg));
	secy = clamp_secy((short)((imy >> 3) - yreg));

	sec = map_mem[(unsigned short)(secy * 128 + secx + xreg)];
	tile = sector_mem[(unsigned short)(sec * 128 + (imy & 7) * 16 + (imx & 15))];

	if ((terra_mem[tile * 4 + 2] & bit) == 0) return 0;
	return terra_mem[tile * 4 + 1] >> 4;
}

/* prox(x, y): collision test with two probes either side of the character.
 * Returns the blocking terrain type, 0 if the way is clear. */
int prox(int x, int y)
{
	int t = px_to_im(x + 4, y + 2);
	if (t == 1 || t >= 10) return t;
	t = px_to_im(x - 4, y + 2);
	if (t == 1 || t >= 8) return t;
	return 0;
}

/* mapxy(imx, imy): address of the tile byte for a tile position (wraps vertically too) */
unsigned char *mapxy(int imx, int imy)
{
	unsigned short ux = (unsigned short)imx, uy = (unsigned short)imy;
	unsigned short secx = wrap_secx((unsigned short)((ux >> 4) - xreg));
	unsigned short secy = (unsigned short)((uy >> 3) - yreg);
	unsigned char sec;

	if (secy & 32) secy = (secy & 16) ? 0 : 31;
	sec = map_mem[(unsigned short)(secy * 128 + secx + xreg)];
	return sector_mem + (unsigned short)(sec * 128 + (uy & 7) * 16 + (ux & 15));
}

/* gen_mini support: fill minimap[] (19 columns of 6 tile numbers, column by column) for the
 * area starting at tile (img_x, img_y), and work out which sector the hero is in. */
void genmini(int img_x, int img_y)
{
	unsigned short x = (unsigned short)img_x, y;
	short *mp = minimap;
	int i, j;

	for (i = 0; i < 19; i++)
	{	unsigned short xs;

		x &= 0x07ff;
		xs = wrap_secx((unsigned short)((x >> 4) - xreg));
		xs += xreg;
		for (j = 0; j < 6; j++)
		{	unsigned short ys;
			unsigned char sec;

			y = (unsigned short)(img_y + j) & 0x7fff;
			ys = clamp_secy((short)((y >> 3) - yreg));
			sec = map_mem[(unsigned short)((ys << 7) + xs)];
			*mp++ = sector_mem[(unsigned short)(sec * 128 + (y & 7) * 16 + (x & 15))];
		}
		x++;
	}
	{	/* the assembly read the high byte of hero_x and hero_y (68000 is big-endian) */
		int hx = hero_x >> 8, hy = (short)((hero_y >> 8) - yreg);
		int sec = map_mem[(unsigned short)((hy << 7) + hx)];

		if ((region_num & 0xff) > 7) sec += 256;
		hero_sector = (short)sec;
	}
}

/* newx(x, dir, speed) / newy(y, dir, speed): move one step in one of 8 directions (dir > 7 =
 * no move). The result stays within 15 bits; newy keeps bit 15 of y. */
static const short xdir[10] = { -2, 0, 2, 3, 2, 0, -2, -3, 0, 0 };
static const short ydir[10] = { -2, -3, -2, 0, 2, 3, 2, 0, 0, 0 };

static int step(int v, int dir, int speed, const short *table)
{
	int prod = (short)table[dir & 0xff] * (short)speed;
	unsigned short half = (unsigned short)prod >> 1;		/* lsr.w #1 */
	return (unsigned short)(v + half) & 0x7fff;
}

int newx(int x, int dir, int speed)
{
	if ((dir & 0xff) > 7) return x;
	return step(x, dir, speed, xdir);
}

int newy(int y, int dir, int speed)
{
	if ((dir & 0xff) > 7) return y;
	return step(y, dir, speed, ydir) | (y & 0x8000);
}

/* wrap(x): copy bit 14 into bit 15 and clear bit 14 effect, giving a signed 15 bit value */
int wrap(int x)
{
	unsigned int low = (unsigned int)x & 0xffff;

	if (low & 0x4000) low |= 0x8000; else low &= 0x7fff;
	return (int)(((unsigned int)x & 0xffff0000u) | low);
}

/* map_adjust(x, y): move the camera (map_x, map_y) after the hero, which is allowed to wander
 * 20 pixels (10 vertically) before the camera creeps along, and 70 (24 / 44) before it jumps. */
void map_adjust(int x, int y)
{
	short cx = (short)(x - 144), cy = (short)(y - 70);
	short dx = (short)(cx - map_x), dy = (short)(cy - map_y);

	dx = (short)(dx << 1) >> 1;		/* sign-extend from bit 14 */
	dy = (short)(dy << 1) >> 1;

	if (dx < -70) map_x = cx + 70;
	else if (dx > 70) map_x = cx - 70;
	else if (dx < -20) map_x--;
	else if (dx > 20) map_x++;

	if (dy < -24) map_y = cy + 24;
	else if (dy > 44) map_y = cy - 44;
	else if (dy < -10) map_y--;
	else if (dy > 10) map_y++;
}

/* page_det(x): small lookup used by the page-turn animation */
int page_det(int x)
{
	static const unsigned char pd10[11] = { 9, 9, 8, 7, 6, 5, 5, 5, 4, 4, 4 };

	if (x < 11) return pd10[x];
	if (x > 136) return 10;
	if (x > 135) return 7;
	if (x > 123) return 6;
	if (x > 98) return 5;
	if (x > 71) return 4;
	return 3;
}

/* ---- drawing the map into the play field bitmaps (planes[0..4]) ---- */

/* draws one tile at byte offset `dst` in each of the five planes */
static void draw_tile(int tile, long dst)
{
	int line, p;

	for (p = 0; p < 5; p++)
	{	unsigned char *src = image_mem + p * IPLAN_SZ + tile * 64;
		unsigned char *d = planes[p] + dst;
		for (line = 0; line < 32; line++)
		{	d[0] = src[0];
			d[1] = src[1];
			src += 2;
			d += ROW_BYTES;
		}
	}
}

/* map_draw(): the whole screen, 19 x 6 tiles */
void map_draw(void)
{
	int col, row;

	for (col = 0; col < 19; col++)
		for (row = 0; row < 6; row++)
			draw_tile(minimap[col * 6 + row], row * 32L * ROW_BYTES + col * 2);
}

/* strip_draw(x): one column of tiles, x = byte offset on screen (0 or 36) */
void strip_draw(int x)
{
	int row, col = x / 2;

	for (row = 0; row < 6; row++)
		draw_tile(minimap[col * 6 + row], row * 32L * ROW_BYTES + x);
}

/* row_draw(r): one row of tiles, r = twice the row number (0 or 10) */
void row_draw(int r)
{
	int col, row = r / 2;

	for (col = 0; col < 19; col++)
		draw_tile(minimap[col * 6 + row], row * 32L * ROW_BYTES + col * 2);
}

/* scrollmap(dir): shift the map one tile in one of 8 directions (the blitter did this on the
 * Amiga). Directions are numbered by how the picture moves; the new edge is filled in by
 * strip_draw / row_draw afterwards.
 *   0 right  1 down+right  2 down  3 down+left  4 left  5 up+left  6 up  7 up+right
 * Each entry: source x (bytes), source line, destination x, destination line, width bytes, lines */
static const short scroll_modes[8][6] = {
	{ 0,  0, 2,  0, 36, 192 },
	{ 0,  0, 2, 32, 36, 160 },
	{ 0,  0, 0, 32, 38, 160 },
	{ 2,  0, 0, 32, 36, 160 },
	{ 2,  0, 0,  0, 36, 192 },
	{ 2, 32, 0,  0, 36, 160 },
	{ 0, 32, 0,  0, 38, 160 },
	{ 0, 32, 2,  0, 36, 160 },
};

void scrollmap(int dir)
{
	const short *m = scroll_modes[dir & 7];
	int p, i;

	for (p = 0; p < 5; p++)
	{	for (i = 0; i < m[5]; i++)
		{	/* copy bottom-up when moving down so rows are not overwritten before being read */
			int line = (m[3] > m[1]) ? m[5] - 1 - i : i;
			memmove(planes[p] + (m[3] + line) * ROW_BYTES + m[2],
				planes[p] + (m[1] + line) * ROW_BYTES + m[0], m[4]);
		}
	}
}
