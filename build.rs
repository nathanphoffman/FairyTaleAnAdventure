// Compiles the original C game code (csrc/) into the Rust binary.
// The Amiga library calls and the assembly routines are provided by Rust (src/platform/*),
// except the ones still listed in csrc/asm_stubs.c.
fn main() {
    let files = [
        "fmain.c",
        "fmain2.c",
        "iffsubs.c",
        "hdrive.c",
        "layout.c",
        "asm_port.c",
        "input.c",
        "asm_map.c",
        "asm_blit.c",
        "asm_text.c",
        "narr_data.c",
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
        // The old code is built with warnings on. These categories are switched off because the
        // 1980s K&R style trips them hundreds of times without pointing at real bugs.
        .flag("-Wall")
        .flag("-Wextra")
        .flag("-Wno-implicit-int") // K&R functions without a return type
        .flag("-Wno-return-type") // K&R functions that return int but fall off the end
        .flag("-Wno-missing-field-initializers")
        .flag("-Wno-missing-braces")
        .flag("-Wno-pointer-sign")
        .flag("-Wno-char-subscripts")
        .flag("-Wno-sign-compare")
        .flag("-Wno-unused-parameter")
        .flag("-Wno-unused-variable")
        .flag("-Wno-missing-parameter-type")
        .flag("-Wno-unused-but-set-variable")
        .flag("-Wno-implicit-fallthrough")
        // the C file with the game's main() is called from Rust as fta_main()
        .define("main", "fta_main")
        .compile("fta_c");
    for f in ["amiga.h", "ftale.h", "protos.h", "asm_protos.h"] {
        println!("cargo:rerun-if-changed=csrc/{f}");
    }
}
