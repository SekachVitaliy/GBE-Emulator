#!/usr/bin/env bash
# One-time: build grader (if needed) and regenerate reference caches in place
# (gba-eval/corpus/reference-cache; the committed files are git-LFS pointers).
set -euo pipefail
. "$(dirname "$0")/env.sh"
if [ ! -x "$GRADER" ]; then
  (cd "$EVAL" && CARGO_TARGET_DIR=$ROOT/.cache/grader-target cargo build -p grader --release)
fi
cd "$EVAL"
if [ "${1:-}" != "--force" ] && [ -f corpus/reference-cache/aw-arm-alu.refcache ] \
   && ! head -c 40 corpus/reference-cache/aw-arm-alu.refcache | grep -q '^version https://git-lfs'; then
  echo "reference cache already real; pass --force to redo"; exit 0
fi
time "$GRADER" --reference "$EVAL/reference/mesen.wasm" --precompute corpus/
