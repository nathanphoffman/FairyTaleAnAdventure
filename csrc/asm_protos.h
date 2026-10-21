/* asm_protos.h - routines that were 68000 assembly (fsubs.asm, narr.asm, gdriver.asm,
 * MakeBitMap.asm, inline #asm in fmain2.c).
 * Declared without argument lists for now, only so their return types are known (without it C
 * assumes int, which is wrong for 64 bit pointers). Phase 4 gives them real prototypes.
 */
#ifndef ASM_PROTOS_H
#define ASM_PROTOS_H

int px_to_im(int x, int y), prox(int x, int y);
int newx(int x, int dir, int speed), newy(int y, int dir, int speed), wrap(int x);
void map_adjust(int x, int y), genmini(int img_x, int img_y);
int page_det(int x);
unsigned char *mapxy(int imx, int imy);
long rand(void), bitrand(), rand2(void), rand4(void), rand8(void), rand64(void), rand256(void), rnd();
int getkey(void);
void decode_mouse(void);
void set_course(long object, long target_x, long target_y, long mode);
long stuff_flag(long n);
void prdec(long value, long len), move(long x, long y), text(char *s, long len), ssp(unsigned char *s);
void placard(void), placard_text(long n), question(long n), cursor(long len, long pen);
void event(long n), speak(long n), msg(char *start, long num), prq(long x);
long ion(unsigned long v);
void map_draw(void), strip_draw(int x), row_draw(int r), scrollmap(int dir);
void bigdraw(int mx, int my);
void make_mask(unsigned char *src, unsigned char *dest, int words, int scans, int chars);
void maskit(int x, int y, int mod, int ch);
void mask_blit(void), shape_blit(void);
void save_blit(unsigned char *savemem), rest_blit(unsigned char *savemem), clear_blit(unsigned char *mem, int size);
void do_error(long n);
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
