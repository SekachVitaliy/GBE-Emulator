#!/usr/bin/env bash
# Build gba-emu release wasm. Usage: scripts/build.sh
set -euo pipefail
. "$(dirname "$0")/env.sh"
cd "$EMU"
cargo build --release --lib --target wasm32-unknown-unknown
ls -l "$WASM_DEFAULT"
