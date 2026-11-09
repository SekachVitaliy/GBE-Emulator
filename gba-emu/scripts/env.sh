# sourced by the other scripts
export PATH=/opt/homebrew/opt/rustup/bin:$PATH
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")/../.." && pwd)"
EMU=$ROOT/gba-emu
EVAL=$ROOT/gba-eval
GRADER=$ROOT/.cache/grader-target/release/grader
WASM_DEFAULT=$EMU/target/wasm32-unknown-unknown/release/gba_emu.wasm
SCORES=$ROOT/.claude-flow/data/scores.md
