#!/bin/bash
# State regression: replays the scenarios and compares the game state (hero, stats, actors,
# inventory, RNG seed) with tests/state_golden.txt. Unlike regress.sh this does not look at
# pixels, so it also works when the drawing changes.
#   tests/state_regress.sh            run all, report differences
#   tests/state_regress.sh --update   accept the current output as the new golden state
# Scenarios come from tests/scenarios.txt and tests/state_scenarios.txt (the second has an extra
# FTA_CMD column for teleport and kill commands).
# A failing run can be recorded with FTA_RECORD=file and repeated with FTA_REPLAY=file.
cd "$(dirname "$0")/.."
export SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy FTA_FAST=1
cargo build -q 2>&1 | grep -E "^error" -A5 && exit 2
BIN=target/debug/faery_tale
mkdir -p tests/out
update=0; [ "$1" = "--update" ] && update=1
new=tests/state_golden.new; : > $new
fail=0
cat tests/scenarios.txt tests/state_scenarios.txt | while IFS='|' read -r name tick inject clicks cmds; do
    name=$(echo $name); [[ -z "$name" || "$name" == \#* ]] && continue
    inject=$(echo $inject); clicks=$(echo $clicks); cmds=$(echo $cmds); tick=$(echo $tick)
    FTA_SAVES=$(mktemp -d) FTA_INJECT="$inject" FTA_CLICK="$clicks" FTA_CMD="$cmds" FTA_STATE=tests/out/$name.state \
        FTA_STATE_AFTER=$tick timeout -s KILL 120 $BIN </dev/null >/dev/null 2>tests/out/$name.err
    if [ ! -s tests/out/$name.state ]; then echo "FAIL  $name (no state, see tests/out/$name.err)"; echo "$name FAIL" >> $new; continue; fi
    echo "$name $(sha256sum < tests/out/$name.state | cut -d' ' -f1)" >> $new
done
if [ $update = 1 ]; then mv $new tests/state_golden.txt; echo "golden state updated"; exit 0; fi
while read -r name hash; do
    want=$(grep "^$name " tests/state_golden.txt 2>/dev/null | cut -d' ' -f2)
    if [ "$hash" = "$want" ]; then echo "ok    $name"; else echo "DIFF  $name (tests/out/$name.state)"; fail=1; fi
done < $new
rm -f $new; exit $fail
