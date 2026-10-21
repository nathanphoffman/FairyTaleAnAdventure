#!/bin/sh
# Builds a release binary and packs it (without any game data) into dist/.
# Players put the original game files next to it in game/ or set FTA_DATA.
set -e
cd "$(dirname "$0")/.."
cargo build --release
rm -rf dist && mkdir -p dist/faery_tale
cp target/release/faery_tale dist/faery_tale/
cp README.md dist/faery_tale/
[ -f controls.cfg.example ] && cp controls.cfg.example dist/faery_tale/
tar -C dist -czf dist/faery_tale-linux-x86_64.tar.gz faery_tale
echo "dist/faery_tale-linux-x86_64.tar.gz"
