// Compiles the original C game code (csrc/) into the Rust binary.
// The Amiga library calls and the assembly routines are provided by Rust (src/platform/*),
// except the ones still listed in csrc/platform_stubs.c and csrc/asm_stubs.c.
fn main() {
    let files = [
        "fmain.c",
        "fmain2.c",
        "iffsubs.c",
        "hdrive.c",
        "layout.c",
        "asm_port.c",
        "input.c",
        "platform_stubs.c",
        "asm_stubs.c",
    ];
    let mut b = cc::Build::new();
    for f in files {
        b.file(format!("csrc/{f}"));
        println!("cargo:rerun-if-changed=csrc/{f}");
    }
    b.include("csrc")
        .include("csrc/stubs")
        .flag("-std=gnu89")
        .flag("-fcommon")
        .flag("-fno-strict-aliasing")
        .flag("-w")
        // the C file with the game's main() is called from Rust as fta_main()
        .define("main", "fta_main")
        .compile("fta_c");
    for f in ["amiga.h", "ftale.h", "protos.h", "asm_protos.h"] {
        println!("cargo:rerun-if-changed=csrc/{f}");
    }
}
