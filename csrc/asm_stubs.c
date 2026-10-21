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

