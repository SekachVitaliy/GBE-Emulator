#!/usr/bin/env bash
# Instruction-level lockstep vs reference Mesen: first instruction whose (r15-after, cycles) differs.
# usage: scripts/lockstep.sh <rom path rel. to gba-eval/corpus/roms, no .gba> [N=300000]
#   e.g. scripts/lockstep.sh test/jsmolka/arm 500000
# Line format "r15_after cycles_of_that_instr" (ARM r15=pc+8, Thumb r15=pc+4). Needs: cargo build --release --example steps
# Debug: DUMPAT=<hexaddr> prints 8 words at addr + registers after N steps (stderr).
set -e
D=$(cd "$(dirname "$0")/.." && pwd); T=$(mktemp -d); trap "rm -rf $T" EXIT
export PATH=/opt/homebrew/opt/rustup/bin:$PATH
(cd $D && cargo build --release --example steps 2>&1 | grep -E '^error' -A8 || true)
N=${2:-300000}; ROM=$D/../gba-eval/corpus/roms/$1.gba
$D/target/release/examples/steps $ROM $N > $T/ours.txt
node $D/scripts/mesteps.mjs $ROM $N > $T/ref.txt
python3 - "$T" <<'PY'
import sys
t=sys.argv[1]
a=open(t+'/ours.txt').read().split('\n'); b=open(t+'/ref.txt').read().split('\n')
n=next((i for i,(x,y) in enumerate(zip(a,b)) if x!=y), None)
if n is None: print('identical', len(a))
else:
    print('first diff at instr', n, 'ours', a[n], 'ref', b[n])
    print(' ours ctx', ' | '.join(a[max(0,n-4):n+3])); print(' ref  ctx', ' | '.join(b[max(0,n-4):n+3]))
PY
