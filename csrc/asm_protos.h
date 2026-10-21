/* asm_protos.h - routines that were 68000 assembly (fsubs.asm, narr.asm, gdriver.asm,
 * MakeBitMap.asm, inline #asm in fmain2.c).
 * Declared without argument lists for now, only so their return types are known (without it C
 * assumes int, which is wrong for 64 bit pointers). Phase 4 gives them real prototypes.
 */
#ifndef ASM_PROTOS_H
#define ASM_PROTOS_H

long px_to_im(), prox(), newx(), newy(), wrap(), map_adjust(), genmini(), page_det();
long rand(void), bitrand(), rand2(void), rand4(void), rand8(void), rand64(void), rand256(void), rnd();
int getkey(void);
void decode_mouse(void);
long set_course(), stuff_flag();
long prdec(), text(), move(), ssp(), placard(void), placard_text(), question(), cursor();
long event(), speak(), msg(), prq();
long map_draw(void), strip_draw(), row_draw(), scrollmap(), bigdraw();
long make_mask(), maskit(), mask_blit(void), shape_blit(void), save_blit(), rest_blit(), clear_blit();
long do_error();
void ReadHeader(void), ReadLength(void), unpack_line(unsigned char *dest);
long init_music(), wrap_music(void), setscore(), playscore(), stopscore(void), playsample();
long MakeBitMap(), UnMakeBitMap();

/* more Amiga library calls (platform layer) */
ULONG AvailMem(ULONG flags);
ULONG TypeOfMem(APTR p);
LONG IoErr(void);
struct IOStdReq *CreateStdIO(struct MsgPort *p);
void DeleteStdIO(struct IOStdReq *io);
struct ColorMap *GetColorMap(LONG n);
void FreeColorMap(struct ColorMap *cm);
void FreeVPortCopLists(struct ViewPort *vp);
void FreeCprList(struct cprlist *c);

#endif
