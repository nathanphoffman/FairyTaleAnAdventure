// Compiles the original C game code (csrc/) into the Rust binary.
// The Amiga library calls and the assembly routines are provided by Rust (src/platform/*),
// except the ones still listed in csrc/asm_stubs.c.
use cc::Build;

/// The C files that are still built; everything else of csrc/ is reference only.
const C_FILES: [&str; 2] = ["layout.c", "narr_data.c"];

/// The headers the C files include, so a change in one rebuilds them.
const C_HEADERS: [&str; 4] = ["amiga.h", "ftale.h", "protos.h", "asm_protos.h"];

/// Parts of fmain.c that were ported to Rust. fmain.c is no longer compiled, so these guards only
/// matter if it is built again.
const PORTED_PARTS: [&str; 18] = [
    "RSRC_REGIONS",
    "RSRC_SOUND",
    "RSRC_DISPLAY",
    "RSRC_CARRIER",
    "RSRC_MENUS",
    "RSRC_SAVE",
    "RSRC_DOORS",
    "RSRC_PLACES",
    "RSRC_ENCOUNTERS",
    "RSRC_REVIVE",
    "RSRC_SCREEN_SIZE",
    "RSRC_MOOD",
    "RSRC_KEYS",
    "RSRC_OPTIONS",
    "RSRC_WORLD",
    "RSRC_MOVE_ACTOR",
    "RSRC_DRAW",
    "RSRC_STARTUP",
];

/// Warnings of the old C that are switched off because the 1980s K&R style trips them hundreds of
/// times without pointing at real bugs.
const QUIET_WARNINGS: [&str; 12] = [
    "-Wno-implicit-int", // K&R functions without a return type
    "-Wno-return-type",  // K&R functions that return int but fall off the end
    "-Wno-missing-field-initializers",
    "-Wno-missing-braces",
    "-Wno-pointer-sign",
    "-Wno-char-subscripts",
    "-Wno-sign-compare",
    "-Wno-unused-parameter",
    "-Wno-unused-variable",
    "-Wno-missing-parameter-type",
    "-Wno-unused-but-set-variable",
    "-Wno-implicit-fallthrough",
];

fn main() {
    let mut build = Build::new();

    add_c_files(&mut build);
    set_c_dialect_and_warnings(&mut build);

    for part in PORTED_PARTS {
        build.define(part, None);
    }

    build.compile("fta_c");

    for header in C_HEADERS {
        println!("cargo:rerun-if-changed=csrc/{header}");
    }
}

/// Adds the C files to the build, and rebuilds when one changes.
fn add_c_files(build: &mut Build) {
    for file in C_FILES {
        build.file(format!("csrc/{file}"));
        println!("cargo:rerun-if-changed=csrc/{file}");
    }
}

/// The old C needs the old dialect; it is built with warnings on, minus the noisy categories.
fn set_c_dialect_and_warnings(build: &mut Build) {
    build
        .include("csrc")
        .include("csrc/stubs")
        .flag("-std=gnu89")
        .flag("-fcommon")
        .flag("-fno-strict-aliasing")
        .flag("-Wall")
        .flag("-Wextra");

    for flag in QUIET_WARNINGS {
        build.flag(flag);
    }
}
