/* Struct sizes, checked against the Rust mirrors in src/amiga.rs at startup. */
#include "amiga.h"

const unsigned int fta_sizes[] = {
	sizeof(struct BitMap), sizeof(struct RastPort), sizeof(struct RasInfo),
	sizeof(struct ViewPort), sizeof(struct View), sizeof(struct ColorMap),
	sizeof(struct TextFont), sizeof(struct TextAttr), sizeof(struct SimpleSprite),
	sizeof(struct GfxBase), sizeof(struct Process), sizeof(struct Layer),
	sizeof(struct AreaInfo), sizeof(struct TmpRas), sizeof(struct DiskFontHeader),
};
const unsigned int fta_sizes_count = sizeof(fta_sizes) / sizeof(fta_sizes[0]);
