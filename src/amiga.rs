//! Rust mirrors of the stand-in Amiga structs in csrc/amiga.h.
//! The sizes are checked against the C compiler's at startup (see `check_layout`).
#![allow(non_snake_case, non_camel_case_types, dead_code)]
// These keep the Amiga's own type names (APTR, UWORD, ...), so the structs match the C headers.
#![allow(clippy::upper_case_acronyms)]

use std::os::raw::c_void;

pub type APTR = *mut c_void;
pub type PLANEPTR = *mut u8;
pub type LONG = i32;
pub type ULONG = u32;
pub type UWORD = u16;
pub type WORD = i16;
pub type UBYTE = u8;
pub type BYTE = i8;
pub type BPTR = isize;

/// An Amiga bitmap holds at most this many bitplanes.
pub const MAX_PLANES: usize = 8;
pub const MEMF_CHIP: u32 = 2;

pub const JAM1: u8 = 0;
pub const JAM2: u8 = 1;
pub const COMPLEMENT: u8 = 2;
pub const HIRES: u16 = 0x8000;

#[repr(C)]
pub struct BitMap {
    pub BytesPerRow: UWORD,
    pub Rows: UWORD,
    pub Flags: UBYTE,
    pub Depth: UBYTE,
    pub pad: UWORD,
    pub Planes: [PLANEPTR; MAX_PLANES],
}

#[repr(C)]
pub struct Layer {
    pub rp_bitmap: *mut BitMap,
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
}

#[repr(C)]
pub struct TextFont {
    pub tf_YSize: UWORD,
    pub tf_XSize: UWORD,
    pub tf_Baseline: UWORD,
    pub tf_Flags: UBYTE,
    pub priv_: APTR,
}

#[repr(C)]
pub struct DiskFontHeader {
    pub dfh_TF: TextFont,
}

#[repr(C)]
pub struct TextAttr {
    pub ta_Name: *const u8,
    pub ta_YSize: UWORD,
    pub ta_Style: UBYTE,
    pub ta_Flags: UBYTE,
}

#[repr(C)]
pub struct ColorMap {
    pub Count: UWORD,
    pub ColorTable: *mut UWORD,
}

#[repr(C)]
pub struct TmpRas {
    pub RasPtr: *mut BYTE,
    pub Size: LONG,
}

#[repr(C)]
pub struct AreaInfo {
    pub VctrTbl: *mut WORD,
    pub VctrPtr: *mut WORD,
    pub FlagTbl: *mut BYTE,
    pub FlagPtr: *mut BYTE,
    pub Count: WORD,
    pub MaxCount: WORD,
}

#[repr(C)]
pub struct RastPort {
    pub Layer: *mut Layer,
    pub BitMap: *mut BitMap,
    pub AreaInfo: *mut AreaInfo,
    pub TmpRas: *mut TmpRas,
    pub FgPen: UBYTE,
    pub BgPen: UBYTE,
    pub DrawMode: UBYTE,
    pub cp_x: WORD,
    pub cp_y: WORD,
    pub Font: *mut TextFont,
}

#[repr(C)]
pub struct RasInfo {
    pub Next: *mut RasInfo,
    pub BitMap: *mut BitMap,
    pub RxOffset: WORD,
    pub RyOffset: WORD,
}

#[repr(C)]
pub struct ViewPort {
    pub Next: *mut ViewPort,
    pub ColorMap: *mut ColorMap,
    pub DspIns: APTR,
    pub SprIns: APTR,
    pub ClrIns: APTR,
    pub RasInfo: *mut RasInfo,
    pub DWidth: WORD,
    pub DHeight: WORD,
    pub DxOffset: WORD,
    pub DyOffset: WORD,
    pub Modes: UWORD,
}

#[repr(C)]
pub struct View {
    pub ViewPort: *mut ViewPort,
    pub LOFCprList: APTR,
    pub SHFCprList: APTR,
    pub DyOffset: WORD,
    pub DxOffset: WORD,
    pub Modes: UWORD,
}

#[repr(C)]
pub struct SimpleSprite {
    pub posctldata: *mut UWORD,
    pub height: UWORD,
    pub x: UWORD,
    pub y: UWORD,
    pub num: UWORD,
}

#[repr(C)]
pub struct GfxBase {
    pub ActiView: *mut View,
    pub DisplayFlags: UWORD,
}

#[repr(C)]
pub struct Task {
    pub tc_Node_pri: BYTE,
    pub tc_TrapCode: APTR,
}

#[repr(C)]
pub struct Process {
    pub pr_Task: Task,
    pub pr_WindowPtr: APTR,
}

unsafe extern "C" {
    static fta_sizes: [u32; 15];
    static fta_sizes_count: u32;
}

/// Panics if any Rust struct differs in size from its C twin.
pub fn check_layout() {
    use std::mem::size_of;
    let rust = [
        ("BitMap", size_of::<BitMap>()),
        ("RastPort", size_of::<RastPort>()),
        ("RasInfo", size_of::<RasInfo>()),
        ("ViewPort", size_of::<ViewPort>()),
        ("View", size_of::<View>()),
        ("ColorMap", size_of::<ColorMap>()),
        ("TextFont", size_of::<TextFont>()),
        ("TextAttr", size_of::<TextAttr>()),
        ("SimpleSprite", size_of::<SimpleSprite>()),
        ("GfxBase", size_of::<GfxBase>()),
        ("Process", size_of::<Process>()),
        ("Layer", size_of::<Layer>()),
        ("AreaInfo", size_of::<AreaInfo>()),
        ("TmpRas", size_of::<TmpRas>()),
        ("DiskFontHeader", size_of::<DiskFontHeader>()),
    ];
    unsafe {
        assert_eq!(fta_sizes_count as usize, rust.len());
        for (i, (name, size)) in rust.iter().enumerate() {
            assert_eq!(
                fta_sizes[i] as usize, *size,
                "struct {name} differs between C and Rust"
            );
        }
    }
}
