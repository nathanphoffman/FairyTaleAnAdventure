/* asm_stubs.c - routines that were 68000 assembly (Phase 4 rewrites these)
 *
 * Every function here just reports that it was called and returns 0. They exist so the game
 * links. Replace them one by one with real code (see plan.md).
 */
#include <stdio.h>


static void stub_called(const char *name)
{
	static int count;
	if (count++ < 20) fprintf(stderr, "STUB called: %s\n", name);
}
#define STUB(name) long name() { stub_called(#name); return 0; }

STUB(trapper)
STUB(HandlerInterface)

/* Data that the assembly files define. Real contents come from narr.asm / fsubs.asm later. */
void *WBenchMsg;

/* The game's own random number generator (fsubs.asm). Must match exactly, and must replace the
 * libc rand() (renamed with a macro in amiga.h). */
extern long seed1;   /* defined in fmain.c */

int fta_rand(void)
{
	/* mulu.w multiplies only the low 16 bits of the seed */
	seed1 = (long)((unsigned long)((unsigned short)seed1) * 45821UL) + 1;
	unsigned int r = ((unsigned int)seed1 >> 6) | ((unsigned int)seed1 << 26); /* ror.l #6 */
	return (int)(r & 0x7fffffff);
}
long bitrand(long mask) { return fta_rand() & mask; }
long rand2(void) { return fta_rand() & 1; }
long rand4(void) { return fta_rand() & 3; }
long rand8(void) { return fta_rand() & 7; }
long rand64(void) { return fta_rand() & 63; }
long rand256(void) { return fta_rand() & 255; }
long rnd(long n) { return (long)((unsigned long)((unsigned short)fta_rand()) % (unsigned long)(n & 0xffff)); }
