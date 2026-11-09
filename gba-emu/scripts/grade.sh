#!/usr/bin/env bash
# Usage: scripts/grade.sh [wasm]   (default: target/.../gba_emu.wasm)
# Writes gba-emu/results/<timestamp>/ and appends a line to scores.md.
set -euo pipefail
. "$(dirname "$0")/env.sh"
WASM=$(cd "$(dirname "${1:-$WASM_DEFAULT}")" && pwd)/$(basename "${1:-$WASM_DEFAULT}")
[ -f "$WASM" ] || { echo "no wasm: $WASM" >&2; exit 1; }
[ -x "$GRADER" ] || { echo "grader not built: run scripts/precompute.sh" >&2; exit 1; }
if head -c 40 "$EVAL/corpus/reference-cache/aw-arm-alu.refcache" | grep -q '^version https://git-lfs'; then
  echo "refcache is LFS pointers: run scripts/precompute.sh" >&2; exit 1; fi
TS=$(date +%Y%m%dT%H%M%S)
OUT=$EMU/results/$TS
mkdir -p "$OUT"
cd "$EVAL"
"$GRADER" --reference "$EVAL/reference/mesen.wasm" "$WASM" corpus/ "$OUT" 2>&1 | tee "$OUT/grader.log" | tail -5 || true
[ -f "$OUT/summary.json" ] || { echo "no summary.json, see $OUT/grader.log" >&2; exit 1; }
mkdir -p "$(dirname "$SCORES")"
# regression guard: compare with best previous overall, snapshot source on a new best
NEW=$(python3 -c 'import json,sys;print("%.4f"%json.load(open(sys.argv[1]))["overall"])' "$OUT/summary.json")
BEST=$(grep -o 'overall=[0-9.]*' "$SCORES" 2>/dev/null | cut -d= -f2 | sort -g | tail -1)
BEST=${BEST:-0}
python3 "$EMU/scripts/worst.py" "$OUT" --line "$(basename "$WASM")" >> "$SCORES"
python3 - "$NEW" "$BEST" <<'PY'
import sys
n, b = float(sys.argv[1]), float(sys.argv[2])
print(f"\noverall {n:.4f}  best-previous {b:.4f}  delta {n-b:+.4f}")
if b - n > 0.01:
    print("!" * 70 + f"\n!!! REGRESSION: overall dropped {b-n:.4f} below best previous ({b:.4f}) !!!\n" + "!" * 70)
PY
if python3 -c 'import sys;sys.exit(not float(sys.argv[1])>float(sys.argv[2]))' "$NEW" "$BEST"; then
  mkdir -p "$ROOT/.cache/snapshots"
  tar czf "$ROOT/.cache/snapshots/score-$NEW-src.tgz" -C "$EMU" src examples scripts
  echo "new best: snapshot .cache/snapshots/score-$NEW-src.tgz"
fi
python3 "$EMU/scripts/worst.py" "$OUT"
echo "results: $OUT"
# disk is tight: each run's PNGs are ~230MB; keep PNGs only for the 2 newest runs
for d in $(ls -1dr "$EMU"/results/*/ | tail -n +3); do rm -f "${d:?}"*.png "${d:?}"*.wav; done
