/* asm_blit.c - sprite drawing routines that drove the Amiga blitter in fsubs.asm.
 *
 * The blitter combines up to three source channels (A, B, C) into a destination D, 16 bits at
 * a time, with an optional barrel shift of A and B, first/last word masks on A, and a
 * "minterm" saying how the bits combine. blit() below does the same in software for the modes
 * the game uses, so each routine here sets up the same values as its assembly original and the
 * drawing comes out the same. Words are big-endian in memory (leftmost pixels in the first byte).
 */
#include <string.h>
#include "ftale.h"

extern struct sshape *shp;
extern unsigned char *mask_buffer, *shapedata, *bmask_mem, *shadow_mem;
extern short shift, aoff, boff, bmod, planesize, wmask;
extern PLANEPTR *planes;

#define USEA 1
#define USEB 2
#define USEC 4
#define USED 8

static unsigned int rd16(const unsigned char *p) { return (p[0] << 8) | p[1]; }
static void wr16(unsigned char *p, unsigned int v) { p[0] = v >> 8; p[1] = v; }

struct blt
{	unsigned char *a, *b, *c, *d;
	int amod, bmod, cmod, dmod;		/* bytes to skip at the end of each line */
	unsigned int afwm, alwm;		/* masks for the first and last word of a line of A */
	int ashift, bshift;
	int use;
	int minterm;				/* result for each combination of A, B, C bits */
	int width, height;			/* in words, lines */
};

/* One 16 bit word of the minterm: for every bit, look up the result from the A B C bits. */
static unsigned int minterm16(int mt, unsigned int a, unsigned int b, unsigned int c)
{
	unsigned int r = 0;
	int bit;

	for (bit = 0; bit < 16; bit++)
	{	int idx = (((a >> bit) & 1) << 2) | (((b >> bit) & 1) << 1) | ((c >> bit) & 1);
		r |= ((mt >> idx) & 1) << bit;
	}
	return r;
}

/* Ascending blit (all the game's blits go left to right, top to bottom). */
static void blit(struct blt *t)
{
	int x, y;

	for (y = 0; y < t->height; y++)
	{	unsigned int aprev = 0, bprev = 0;

		for (x = 0; x < t->width; x++)
		{	unsigned int aw = 0xffff, bw = 0, cw = 0, dw, as, bs;

			if (t->use & USEA)
			{	aw = rd16(t->a); t->a += 2;
				if (x == 0) aw &= t->afwm;
				if (x == t->width - 1) aw &= t->alwm;
			}
			if (t->use & USEB) { bw = rd16(t->b); t->b += 2; }
			if (t->use & USEC) cw = rd16(t->c);

			as = ((aprev << 16 | aw) >> t->ashift) & 0xffff;
			bs = ((bprev << 16 | bw) >> t->bshift) & 0xffff;
			aprev = aw;
			bprev = bw;

			dw = minterm16(t->minterm, as, bs, cw);
			if (t->use & USED) { wr16(t->d, dw); t->d += 2; }
			if (t->use & USEC) t->c += 2;
		}
		if (t->use & USEA) t->a += t->amod;
		if (t->use & USEB) t->b += t->bmod;
		if (t->use & USEC) t->c += t->cmod;
		if (t->use & USED) t->d += t->dmod;
	}
}

/* BLTSIZE: height in the top 10 bits, width in words in the low 6 */
#define BLT_H(size) (((size) >> 6) & 0x3ff)
#define BLT_W(size) (((size) & 0x3f) ? ((size) & 0x3f) : 64)

/* clear_blit(mem, size): zero a rectangle of memory */
void clear_blit(unsigned char *mem, int size)
{
	memset(mem, 0, BLT_H(size) * BLT_W(size) * 2);
}

/* save_blit(savemem): copy the background under the sprite (5 planes) to savemem */
void save_blit(unsigned char *savemem)
{
	int p;

	for (p = 4; p >= 0; p--)
	{	struct blt t;
		memset(&t, 0, sizeof t);
		t.use = USEB | USED;
		t.minterm = 0xcc;			/* D = B */
		t.afwm = t.alwm = 0xffff;
		t.b = planes[p] + shp->Coff;
		t.bmod = shp->Cmod;
		t.d = savemem + shp->savesize * p;
		t.dmod = 0;
		t.width = BLT_W(shp->blitsize);
		t.height = BLT_H(shp->blitsize);
		blit(&t);
	}
}

/* rest_blit(savemem): put the saved background back */
void rest_blit(unsigned char *savemem)
{
	int p;

	for (p = 4; p >= 0; p--)
	{	struct blt t;
		memset(&t, 0, sizeof t);
		t.use = USEB | USED;
		t.minterm = 0xcc;			/* D = B */
		t.afwm = t.alwm = 0xffff;
		t.b = savemem + shp->savesize * p;
		t.bmod = 0;
		t.d = planes[p] + shp->Coff;
		t.dmod = shp->Cmod;
		t.width = BLT_W(shp->blitsize);
		t.height = BLT_H(shp->blitsize);
		blit(&t);
	}
}

/* mask_blit(): combine the sprite's own mask with the scenery mask built by maskit(). Result in
 * bmask_mem: sprite is opaque here and nothing in front of it (D = A and not C). */
void mask_blit(void)
{
	struct blt t;

	memset(&t, 0, sizeof t);
	t.use = USEA | USEC | USED;
	t.minterm = 0x50;				/* D = A & ~C */
	t.a = mask_buffer + boff;
	t.amod = bmod;
	t.ashift = shift & 15;
	t.afwm = t.alwm = 0xffff;
	t.c = t.d = bmask_mem + 2 + aoff;
	t.cmod = t.dmod = 0;
	t.width = BLT_W(shp->blitsize);
	t.height = BLT_H(shp->blitsize);
	blit(&t);
}

/* shape_blit(): draw the sprite into the 5 planes through the mask: D = mask ? shape : screen */
void shape_blit(void)
{
	int p;
	unsigned int first = 0xffff >> (shift & 15), last = ~first & 0xffff;

	if (last == 0) last = 0xffff;
	for (p = 4; p >= 0; p--)
	{	struct blt t;
		memset(&t, 0, sizeof t);
		t.use = USEA | USEB | USEC | USED;
		t.minterm = 0xca;			/* D = A ? B : C */
		t.bshift = shift & 15;
		t.afwm = first & (unsigned short)wmask;
		t.alwm = last;
		t.a = bmask_mem + 2 + aoff;
		t.amod = 0;
		t.b = shapedata + boff + planesize * p;
		t.bmod = bmod;
		t.c = t.d = planes[p] + shp->Coff;
		t.cmod = t.dmod = shp->Cmod;
		t.width = BLT_W(shp->blitsize);
		t.height = BLT_H(shp->blitsize);
		blit(&t);
	}
}

/* maskit(x, y, mod, char): copy the 32 line mask of scenery tile `char` (from shadow_mem) into
 * the scenery mask at word column x, tile row y; `mod` is the mask width in words. */
void maskit(int x, int y, int mod, int ch)
{
	unsigned char *dst = bmask_mem + 2;
	unsigned char *src = shadow_mem + ch * 64;
	int bytes = mod * 2, line;

	dst += y * bytes * 32 + x * 2;
	for (line = 0; line < 32; line++)
	{	dst[0] = src[0];
		dst[1] = src[1];
		src += 2;
		dst += bytes;
	}
}

/* make_mask(src, dest, words, scans, chars): for every sprite frame build a one plane mask:
 * all 5 planes ANDed then inverted, so colour 31 (all bits set) is transparent. */
void make_mask(unsigned char *src, unsigned char *dest, int words, int scans, int chars)
{
	int plane_words = (unsigned short)(words * scans);
	int plane_bytes = plane_words * 2, c, i;

	for (c = 0; c < chars; c++)
	{	unsigned char *p0 = src, *p1 = p0 + plane_bytes, *p2 = p1 + plane_bytes;
		unsigned char *p3 = p2 + plane_bytes, *p4 = p3 + plane_bytes;

		for (i = 0; i < plane_bytes; i++)
			*dest++ = ~(p0[i] & p1[i] & p2[i] & p3[i] & p4[i]);
		src = p4 + plane_bytes;
	}
}
