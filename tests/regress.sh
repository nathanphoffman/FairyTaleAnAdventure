#!/bin/bash
# Regression tests: replays scripted sessions without a window and compares a hash of the final
# screen with tests/golden.txt. Only hashes are stored, so no game graphics are in the repo.
#   tests/regress.sh            run all, report differences
#   tests/regress.sh --update   accept the current output as the new golden hashes
# Needs the original game data (see README). Screenshots of failures land in tests/out/.
cd "$(dirname "$0")/.."
export SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy FTA_FAST=1
cargo build -q 2>&1 | grep -E "^error" -A5 && exit 2
BIN=target/debug/faery_tale
mkdir -p tests/out
update=0; [ "$1" = "--update" ] && update=1
new=tests/golden.new; : > $new
fail=0
while IFS='|' read -r name tick inject clicks; do
    name=$(echo $name); [[ -z "$name" || "$name" == \#* ]] && continue
    inject=$(echo $inject); clicks=$(echo $clicks); tick=$(echo $tick)
    FTA_SAVES=$(mktemp -d) FTA_INJECT="$inject" FTA_CLICK="$clicks" FTA_SHOT=tests/out/$name.ppm \
        FTA_SHOT_AFTER=$tick timeout -s KILL 120 $BIN >/dev/null 2>tests/out/$name.err
    if [ ! -s tests/out/$name.ppm ]; then echo "FAIL  $name (no screenshot, see tests/out/$name.err)"; fail=1; continue; fi
    echo "$name $(sha256sum < tests/out/$name.ppm | cut -d' ' -f1)" >> $new
done < tests/scenarios.txt
# Save and load round trip: walk, save to slot A, then a fresh run loads it. The play field must match.
sv=$(mktemp -d)
FTA_SAVES=$sv FTA_FAST=1 FTA_INJECT="700:Right:60" FTA_CLICK="800:225:166,815:225:184,830:250:166,845:225:148" \
    FTA_SHOT=tests/out/save_before.ppm FTA_SHOT_AFTER=870 timeout -s KILL 120 $BIN >/dev/null 2>&1
FTA_SAVES=$sv FTA_FAST=1 FTA_CLICK="700:225:166,720:250:184,740:225:148" \
    FTA_SHOT=tests/out/save_after.ppm FTA_SHOT_AFTER=900 timeout -s KILL 120 $BIN >/dev/null 2>&1
if [ -s $sv/A.faery ] && python3 - <<'P'
a = open('tests/out/save_before.ppm', 'rb').read()[15:]
b = open('tests/out/save_after.ppm', 'rb').read()[15:]
rows = 140 * 640 * 3  # the play field
raise SystemExit(0 if a[:rows] == b[:rows] else 1)
P
then echo "saveload ok"; else echo "FAIL  saveload (tests/out/save_*.ppm)"; fail=1; fi
rm -rf $sv
if [ $update = 1 ]; then mv $new tests/golden.txt; echo "golden hashes updated"; exit 0; fi
while read -r name hash; do
    want=$(grep "^$name " tests/golden.txt 2>/dev/null | cut -d' ' -f2)
    if [ "$hash" = "$want" ]; then echo "ok    $name"; else echo "DIFF  $name (tests/out/$name.ppm)"; fail=1; fi
done < $new
rm -f $new; exit $fail
