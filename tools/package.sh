#!/bin/sh
# Publishes a release build and packs it (without any game data) into dist/.
# Players put the original game files next to it in game/ or set FTA_DATA. Needs the .NET 10 runtime.
set -e
cd "$(dirname "$0")/.."
rm -rf dist && mkdir -p dist/faery_tale
dotnet publish csharp/FaeryTale -c Release -o dist/faery_tale
cp README.md dist/faery_tale/
[ -f controls.cfg.example ] && cp controls.cfg.example dist/faery_tale/
tar -C dist -czf dist/faery_tale.tar.gz faery_tale
echo "dist/faery_tale.tar.gz"
