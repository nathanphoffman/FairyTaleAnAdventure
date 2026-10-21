/* amiga.h - stand-ins for the Amiga system headers listed in fincludes.c
 *
 * Only what the game uses. Struct layouts are our own (not binary compatible with AmigaOS);
 * the Rust platform layer implements the functions declared here.
 */
#ifndef AMIGA_H
#define AMIGA_H

#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

/* Aztec C on the Amiga had a 32 bit long. On 64 bit Linux long is 64 bit, which changes struct
 * sizes and arithmetic. Make `long` mean 32 bit. Keep this after all system headers. */
#define long int

/* ---- exec/types.h ---- */
typedef void *APTR;
typedef int32_t LONG;
typedef uint32_t ULONG;
typedef uint16_t UWORD;
typedef int16_t WORD;
typedef uint8_t UBYTE;
typedef int8_t BYTE;
typedef int16_t SHORT;
typedef uint16_t USHORT;
typedef uint8_t *PLANEPTR;
typedef int16_t BOOL;
typedef intptr_t BPTR;
typedef char *STRPTR;

#ifndef TRUE
#define TRUE 1
#define FALSE 0
#endif
#ifndef NULL
#define NULL ((void *)0)
#endif

/* ---- exec/memory.h ---- */
#define MEMF_PUBLIC 1L
#define MEMF_CHIP 2L
#define MEMF_FAST 4L
#define MEMF_CLEAR 0x10000L

/* ---- dos ---- */
#define MODE_OLDFILE 1005L
#define MODE_NEWFILE 1006L
#define ACCESS_READ (-2L)
#define ACCESS_WRITE (-1L)
#define OFFSET_BEGINNING (-1L)

/* ---- graphics ---- */
#define JAM1 0
#define JAM2 1
#define COMPLEMENT 2
#define INVERSVID 4
#define DEST 0x0100  /* from hardware/blit.h, only used in a dead comparison in iffsubs.c */
#define HIRES 0x8000
#define LAYERSIMPLE 1
#define RASSIZE(w,h) ((ULONG)(h) * (((ULONG)(w) + 15) >> 3 & 0xFFFE))
#define SPRITES 0x4000
#define VP_HIDE 0x2000

struct BitMap {
	UWORD BytesPerRow;
	UWORD Rows;
	UBYTE Flags;
	UBYTE Depth;
	UWORD pad;
	PLANEPTR Planes[8];
};

struct Layer {
	struct BitMap *rp_bitmap;
};
struct Layer_Info {
	int dummy;
};
struct TextFont {
	UWORD tf_YSize;
	UWORD tf_XSize;
	UWORD tf_Baseline;
	UBYTE tf_Flags;
};
#define FPF_ROMFONT 1

struct TextAttr {
	STRPTR ta_Name;
	UWORD ta_YSize;
	UBYTE ta_Style;
	UBYTE ta_Flags;
};

struct ColorMap {
	UWORD Count;
	UWORD *ColorTable;
};

struct TmpRas {
	BYTE *RasPtr;
	LONG Size;
};

struct AreaInfo {
	WORD *VctrTbl;
	WORD *VctrPtr;
	BYTE *FlagTbl;
	BYTE *FlagPtr;
	WORD Count;
	WORD MaxCount;
};

struct RastPort {
	struct Layer *Layer;
	struct BitMap *BitMap;
	struct AreaInfo *AreaInfo;
	struct TmpRas *TmpRas;
	UBYTE FgPen, BgPen, DrawMode;
	WORD cp_x, cp_y;
	struct TextFont *Font;
};

struct RasInfo {
	struct RasInfo *Next;
	struct BitMap *BitMap;
	WORD RxOffset, RyOffset;
};

struct cprlist;
struct ViewPort {
	struct ViewPort *Next;
	struct ColorMap *ColorMap;
	struct cprlist *DspIns, *SprIns, *ClrIns;
	struct RasInfo *RasInfo;
	WORD DWidth, DHeight, DxOffset, DyOffset;
	UWORD Modes;
};

struct View {
	struct ViewPort *ViewPort;
	struct cprlist *LOFCprList;
	struct cprlist *SHFCprList;
	WORD DyOffset, DxOffset;
	UWORD Modes;
};

struct SimpleSprite {
	UWORD *posctldata;
	UWORD height;
	UWORD x, y;
	UWORD num;
};

struct GfxBase {
	struct View *ActiView;
	UWORD DisplayFlags;
};

/* ---- exec ---- */
struct Library {
	UWORD lib_Version;
};
struct Task {
	BYTE tc_Node_pri;
	APTR tc_TrapCode;
};
struct Process {
	struct Task pr_Task;
	APTR pr_WindowPtr;
};
struct MsgPort {
	UBYTE flags;
};
struct Node {
	BYTE ln_Pri;
};
struct Interrupt {
	struct Node is_Node;
	APTR is_Data;
	void (*is_Code)();
};
struct Message {
	struct MsgPort *mn_ReplyPort;
};

struct IORequest {
	struct Message io_Message;
	struct Library *io_Device;
	APTR io_Unit;
	UWORD io_Command;
	UBYTE io_Flags;
	BYTE io_Error;
};
/* IOStdReq starts with the same fields as IORequest, so casts between them work as on the Amiga. */
struct IOStdReq {
	struct Message io_Message;
	struct Library *io_Device;
	APTR io_Unit;
	UWORD io_Command;
	UBYTE io_Flags;
	BYTE io_Error;
	ULONG io_Actual, io_Length;
	APTR io_Data;
	ULONG io_Offset;
};
#define CMD_RESET 1
#define CMD_READ 2
#define CMD_WRITE 3
#define CMD_INVALID 0
#define TD_MOTOR 9
#define TD_NAME "trackdisk.device"
#define IOF_QUICK 1

struct IOExtTD {
	struct IOStdReq iotd_Req;
	ULONG iotd_Count;
	ULONG iotd_SecLabel;
};

struct IOAudio {
	struct IORequest ioa_Request;
	WORD ioa_AllocKey;
	UBYTE *ioa_Data;
	ULONG ioa_Length;
	UWORD ioa_Period, ioa_Volume, ioa_Cycles;
};

struct FileLock;
struct DeviceList;
struct WBArg {
	BPTR wa_Lock;
};
struct WBStartup {
	struct WBArg *sm_ArgList;
};
struct DiskFontHeader {
	struct TextFont dfh_TF;
};

#define IND_ADDHANDLER 9
#define IND_REMHANDLER 10

/* ---- AmigaOS functions the platform layer provides ---- */
APTR AllocMem(ULONG size, ULONG flags);
void FreeMem(APTR mem, ULONG size);
PLANEPTR AllocRaster(ULONG w, ULONG h);
void FreeRaster(PLANEPTR p, ULONG w, ULONG h);

BPTR Open(const char *name, LONG mode);
void Close(BPTR f);
LONG Read(BPTR f, void *buf, LONG len);
LONG Write(BPTR f, const void *buf, LONG len);
LONG Seek(BPTR f, LONG pos, LONG mode);
BPTR Lock(const char *name, LONG mode);
void UnLock(BPTR lock);
void Delay(LONG ticks);

void InitBitMap(struct BitMap *bm, LONG depth, LONG w, LONG h);
void InitRastPort(struct RastPort *rp);
void SetRast(struct RastPort *rp, ULONG pen);
void SetAPen(struct RastPort *rp, ULONG pen);
void SetBPen(struct RastPort *rp, ULONG pen);
void SetDrMd(struct RastPort *rp, ULONG mode);
void Move(struct RastPort *rp, LONG x, LONG y);
void Draw(struct RastPort *rp, LONG x, LONG y);
void RectFill(struct RastPort *rp, LONG x0, LONG y0, LONG x1, LONG y1);
LONG Text(struct RastPort *rp, const char *s, LONG len);
void SetFont(struct RastPort *rp, struct TextFont *f);
void ScrollRaster(struct RastPort *rp, LONG dx, LONG dy, LONG x0, LONG y0, LONG x1, LONG y1);
LONG BltBitMap(struct BitMap *src, LONG sx, LONG sy, struct BitMap *dst, LONG dx, LONG dy,
	LONG w, LONG h, ULONG minterm, ULONG mask, PLANEPTR temp);
void WaitBlit(void);
void OwnBlitter(void);
void DisownBlitter(void);
void SetRGB4(struct ViewPort *vp, LONG n, ULONG r, ULONG g, ULONG b);
void LoadRGB4(struct ViewPort *vp, UWORD *colors, LONG count);
ULONG GetRGB4(struct ColorMap *cm, LONG n);
void InitArea(struct AreaInfo *a, void *buf, LONG maxvectors);
struct TmpRas *InitTmpRas(struct TmpRas *t, PLANEPTR buf, LONG size);
LONG AreaMove(struct RastPort *rp, LONG x, LONG y);
LONG AreaDraw(struct RastPort *rp, LONG x, LONG y);
LONG AreaEnd(struct RastPort *rp);

struct Layer_Info *NewLayerInfo(void);
void DisposeLayerInfo(struct Layer_Info *li);
struct Layer *CreateUpfrontLayer(struct Layer_Info *li, struct BitMap *bm, LONG x0, LONG y0, LONG x1, LONG y1, LONG flags, struct BitMap *super);
LONG DeleteLayer(APTR dummy, struct Layer *l);
void InitView(struct View *v);
void InitVPort(struct ViewPort *vp);
void MakeVPort(struct View *v, struct ViewPort *vp);
void MrgCop(struct View *v);
void LoadView(struct View *v);
void WaitBOVP(struct ViewPort *vp);
void WaitTOF(void);
LONG GetSprite(struct SimpleSprite *s, LONG n);
void FreeSprite(LONG n);
void ChangeSprite(struct ViewPort *vp, struct SimpleSprite *s, APTR data);
void MoveSprite(struct ViewPort *vp, struct SimpleSprite *s, LONG x, LONG y);

struct TextFont *OpenFont(struct TextAttr *a);
struct TextFont *OpenDiskFont(struct TextAttr *a);
void CloseFont(struct TextFont *f);
BPTR LoadSeg(const char *name);
void UnLoadSeg(BPTR seg);

struct Library *OpenLibrary(const char *name, ULONG version);
void CloseLibrary(struct Library *lib);
struct Task *FindTask(const char *name);
BPTR CurrentDir(BPTR lock);

/* trackdisk / audio device I/O. The port reads blocks from the `image` file instead. */
struct MsgPort *CreatePort(const char *name, LONG pri);
void DeletePort(struct MsgPort *p);
struct IORequest *CreateExtIO(struct MsgPort *p, LONG size);
void DeleteExtIO(struct IORequest *io);
LONG OpenDevice(const char *name, ULONG unit, struct IORequest *io, ULONG flags);
void CloseDevice(struct IORequest *io);
LONG DoIO(struct IORequest *io);
void SendIO(struct IORequest *io);
LONG WaitIO(struct IORequest *io);
LONG CheckIO(struct IORequest *io);
void BeginIO(struct IORequest *io);

#endif
