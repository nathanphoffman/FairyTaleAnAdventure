/* platform_stubs.c - Amiga system library calls (Phase 3 replaces these with SDL2/Rust)
 *
 * Every function here just reports that it was called and returns 0. They exist so the game
 * links. Replace them one by one with real code (see plan.md).
 */
#include <stdio.h>
#include <stdlib.h>

static void stub_called(const char *name)
{
	static int count;
	if (count++ < 20) fprintf(stderr, "STUB called: %s\n", name);
}
#define STUB(name) long name() { stub_called(#name); return 0; }

