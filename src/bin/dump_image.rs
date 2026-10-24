//! Prints a map of the 880K floppy `image` file, so we can see where the data lives.
//! Usage: cargo run --bin dump_image -- path/to/image [first_block] [count]
//!
//! With just the path it prints one line per 16 blocks. With a block range it prints a hex
//! dump of the first 32 bytes of each block.

use std::fs;

const BLOCK: usize = 512;

/// Known block ranges from the original source (see docs/data-formats.md).
const KNOWN: &[(usize, usize, &str)] = &[
    (149, 12, "terrain tables (TERRA_BLOCK + 0..10)"),
    (200, 120, "landscape images (region image[] groups)"),
    (240, 40, "landscape images"),
    (280, 40, "landscape images"),
    (320, 40, "landscape images"),
    (360, 40, "landscape images"),
    (400, 40, "landscape images"),
    (440, 40, "landscape images"),
    (480, 40, "landscape images"),
    (520, 40, "landscape images"),
    (560, 40, "landscape images"),
    (600, 40, "landscape images"),
    (640, 40, "landscape images"),
    (680, 40, "indoor/dungeon images"),
    (720, 40, "indoor/dungeon images"),
    (760, 40, "indoor/dungeon images"),
    (800, 40, "indoor/dungeon images"),
    (840, 40, "indoor/dungeon images"),
    (880, 1, "disk id / copy protection check"),
    (896, 24, "shadow masks"),
    (920, 11, "sound effect samples"),
    (931, 25, "set figure sprites"),
    (960, 40, "ogre sprites"),
    (1000, 40, "dknight / spider sprites"),
    (1040, 40, "necromancer sprites"),
    (1080, 40, "ghost sprites"),
    (1120, 40, "bird sprites"),
    (1160, 12, "dragon sprites"),
    (1312, 36, "object sprites"),
    (1348, 3, "raft sprites"),
    (1351, 20, "turtle sprites"),
    (1376, 42, "julian sprites"),
    (1418, 42, "phillip sprites"),
    (1460, 42, "kevin sprites"),
];

fn label(block: usize) -> &'static str {
    KNOWN
        .iter()
        .find(|(start, len, _)| block >= *start && block < start + len)
        .map(|(_, _, name)| *name)
        .unwrap_or("")
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let Some(path) = args.get(1) else {
        eprintln!("usage: dump_image <image> [first_block] [count]");
        std::process::exit(2);
    };
    let data = fs::read(path).unwrap_or_else(|e| {
        eprintln!("cannot read {path}: {e}");
        std::process::exit(1);
    });
    let blocks = data.len() / BLOCK;
    println!("{path}: {} bytes, {blocks} blocks", data.len());

    if let Some(first) = args.get(2).and_then(|s| s.parse::<usize>().ok()) {
        let count = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(1);
        for b in first..(first + count).min(blocks) {
            let bytes = &data[b * BLOCK..b * BLOCK + 32];
            let hex: Vec<String> = bytes.iter().map(|x| format!("{x:02x}")).collect();
            println!("{b:5} {}  {}", hex.join(" "), label(b));
        }
        return;
    }

    for start in (0..blocks).step_by(16) {
        let end = (start + 16).min(blocks);
        let chunk = &data[start * BLOCK..end * BLOCK];
        let used = chunk.iter().filter(|&&b| b != 0).count();
        let pct = used * 100 / chunk.len();
        println!(
            "{start:5}-{:<5} {pct:3}% non-zero  {}",
            end - 1,
            label(start)
        );
    }
}
