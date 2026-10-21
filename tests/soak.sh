#!/bin/bash
# Soak test: feeds random keys and mouse clicks for a few thousand ticks, with the
# AddressSanitizer build, and fails if the game crashes or reports a memory error.
#   tests/soak.sh [runs] [ticks]     (default 5 runs of 6000 ticks, seeds 1..runs)
cd "$(dirname "$0")/.."
runs=${1:-5}; ticks=${2:-6000}
export SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy FTA_FAST=1
T=/tmp/fta-asan-target
CFLAGS="-fsanitize=address -fno-omit-frame-pointer" \
RUSTFLAGS="-C link-arg=-Wl,--no-as-needed -C link-arg=-lasan" \
    cargo build -q --target x86_64-unknown-linux-gnu --target-dir $T 2>&1 | grep -E "^error" -A5 && exit 2
BIN=$T/x86_64-unknown-linux-gnu/debug/faery_tale
fail=0
for seed in $(seq 1 $runs); do
    read -r inj clk < <(python3 - $seed $ticks <<'P'
import random, sys
r = random.Random(int(sys.argv[1])); n = int(sys.argv[2])
keys = list("IT?UGYSAMFQLVXOR8CWBEK1234567") + ["Space", "F1", "F2", "F3", "F4", "F5", "F6", "F7", "Left", "Right", "Up", "Down", "LCtrl"]
inj = []; clk = []
t = 400
while t < n:
    t += r.randint(5, 40)
    if r.random() < 0.7:
        k = r.choice(keys)
        inj.append(f"{t}:{k}:{r.randint(1, 30)}")
    else:
        clk.append(f"{t}:{r.randint(0, 319)}:{r.randint(0, 199)}")
print(",".join(inj) or "-", ",".join(clk) or "-")
P
)
    LD_PRELOAD=$(gcc -print-file-name=libasan.so) ASAN_OPTIONS=detect_leaks=0 \
    FTA_SAVES=$(mktemp -d) FTA_INJECT="$inj" FTA_CLICK="$clk" FTA_SHOT=/dev/null FTA_SHOT_AFTER=$ticks \
        timeout -s KILL 300 $BIN >/dev/null 2>tests/out/soak$seed.err
    code=$?
    if [ $code -ne 0 ] || grep -q "AddressSanitizer\|panicked" tests/out/soak$seed.err; then
        echo "FAIL  soak seed $seed (exit $code, tests/out/soak$seed.err)"; fail=1
    else echo "ok    soak seed $seed"; fi
done
exit $fail
