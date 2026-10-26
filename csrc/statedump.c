/* Test helper: writes the main game state as text, so tests can compare state instead of pixels.
   Called from Rust (display.rs) when FTA_STATE=file is set. Read-only: never changes the game. */
#include <stdio.h>
#include "amiga.h"
#include "ftale.h"
#include "protos.h"

#define ARROWBASE 35
extern struct shape anim_list[20];
extern short anix, anix2, brother, riding, flying, brave, luck, kind, wealth, hunger, fatigue;
extern unsigned short hero_x, hero_y, map_x, map_y, daynight, lightlevel, region_num;
extern UBYTE *stuff;
extern long seed1, seed2;

void fta_dump_state(const char *path)
{	FILE *f = fopen(path, "w");
	int i;
	if (!f) return;
	fprintf(f, "brother %d region %u hero %u,%u map %u,%u\n", brother, region_num, hero_x, hero_y, map_x, map_y);
	fprintf(f, "brave %d luck %d kind %d wealth %d hunger %d fatigue %d\n", brave, luck, kind, wealth, hunger, fatigue);
	fprintf(f, "daynight %u light %u riding %d flying %d anix %d,%d\n", daynight, lightlevel, riding, flying, anix, anix2);
	fprintf(f, "seed %d %d\n", (int)seed1, (int)seed2);
	for (i = 0; i < 20; i++)
	{	struct shape *a = &anim_list[i];
		fprintf(f, "actor %d pos %u,%u type %d race %d idx %d wpn %d env %d goal %d tac %d state %d face %d vit %d vel %d,%d\n",
			i, a->abs_x, a->abs_y, a->type, a->race, a->index, a->weapon, a->environ, a->goal, a->tactic,
			a->state, a->facing, a->vitality, a->vel_x, a->vel_y);
	}
	fprintf(f, "stuff");
	for (i = 0; i < ARROWBASE; i++) fprintf(f, " %d", stuff[i]);
	fprintf(f, "\n");
	fclose(f);
}

/* Test commands (FTA_CMD in display.rs): queued from the display thread, applied at the top of the
   game's main loop by fta_test_hook(), so they never run in the middle of a frame. */
static int cmd_kind, cmd_a, cmd_b;

void fta_queue_cmd(int kind, int a, int b)
{	cmd_kind = kind; cmd_a = a; cmd_b = b;
}

extern short luck;
void fta_test_hook(void)
{	int kind = cmd_kind, i;
	if (!kind) return;
	cmd_kind = 0;
	if (kind == 1) xfer((unsigned short)cmd_a, (unsigned short)cmd_b, 1);	/* teleport, like a door */
	else if (kind == 3) stuff[cmd_a] = cmd_b;							/* give: stuff[slot] = count */
	else if (kind == 4)													/* set a stat: 0 hunger, 1 fatigue, 2 wealth */
	{	if (cmd_a == 0) hunger = cmd_b; else if (cmd_a == 1) fatigue = cmd_b; else wealth = cmd_b;
	}
	else if (kind == 5)													/* kill every enemy that is out */
	{	for (i = 3; i < anix; i++)
			if (anim_list[i].type == ENEMY) { anim_list[i].vitality = 0; checkdead(i, 0); }
	}
	else if (kind == 2)													/* kill the hero with the given luck */
	{	luck = cmd_a; anim_list[0].vitality = 0; checkdead(0, 5);
	}
}
