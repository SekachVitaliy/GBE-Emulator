# Grading gba-emu locally

Native grader from `gba-eval` (read-only; no docker, no git-lfs). Source of truth: `gba-eval/README.md` "How grading works".

## Commands (all in `gba-emu/scripts/`)
| script | what |
|---|---|
| `build.sh` | `cargo build --release --lib --target wasm32-unknown-unknown` -> `target/wasm32-unknown-unknown/release/gba_emu.wasm` |
| `precompute.sh [--force]` | builds grader once (`CARGO_TARGET_DIR=<repo>/.cache/grader-target`, ~50 s) and regenerates reference caches |
| `grade.sh [wasm]` | grades (default wasm above) against `gba-eval/corpus` with `reference/mesen.wasm`; writes `results/<timestamp>/` (per-testcase JSON, cand/ref PNGs, `summary.json`, `grader.log`); appends a line to `<repo root>/.claude-flow/data/scores.md`; prints the worst-testcases report; deletes PNGs of all but the 2 newest runs |
| `worst.py <results_dir>` | per-subsystem worst testcases with `first_diverge` frame (fix earliest-diverging first); `--line NAME` prints the scores.md line |
| `env.sh` | shared paths, rustup PATH |

Typical loop: `scripts/build.sh && scripts/grade.sh` (about 1 min for the 27-case corpus on a multi-core Mac; slower once the emulator does real work).

## Reference cache (one-time, done)
`gba-eval/corpus/reference-cache/*.refcache` were git-LFS pointers. `precompute.sh` regenerated them **in place inside gba-eval/corpus** (51 files, 227 MB, 2m40s, ~5 cores busy). Redo only if the reference wasm or testcases.json changes. grade.sh refuses to run if they are pointers.

## Gotchas
- The grader resolves `--reference` relative to cwd; the scripts pass an absolute path and `cd` into gba-eval.
- Only the refcache is written inside gba-eval; results live in `gba-emu/results/`.
- Disk: grader target 410 MB (`.cache/grader-target`); each results run ~237 MB (PNGs; pruned automatically).
- Corpus has 27 testcases in testcases.json, not the 44 replay files (extra replays/ROMs are unused by the grader).

## Tool versions (2026-11-06)
rustc 1.99.1, cargo 1.99.1 (gba-eval README says 1.87; 1.99 builds fine, wasmtime 27), targets aarch64-apple-darwin + wasm32-unknown-unknown, python3 (worst.py), macOS Darwin 25.5 arm64. Node is not used by the grader (only by the ruflo memory CLI).

## Outside reviewer rerun
```
export PATH=/opt/homebrew/opt/rustup/bin:$PATH   # any rustup with wasm32-unknown-unknown
cd gba-emu && scripts/build.sh
scripts/precompute.sh      # only if corpus/reference-cache holds LFS pointers (git lfs pull is the alternative)
scripts/grade.sh           # or: scripts/grade.sh path/to.wasm
```
Without scripts, from `gba-eval/`: `cargo run -p grader --release -- --reference reference/mesen.wasm cand.wasm corpus/ out/`.

## Baseline
Stub wasm (21 KB): **overall 0.0085** (procedural 0.0000, replay 0.0141, audio 0.0000). Grading took 64 s.

## Scoring (corpus/grader.yaml)
overall = 0.60 replay + 0.20 procedural + 0.20 audio. Subsystem weights are fixed within a section, so ROM count does not change weight (per-ROM weight = subsystem weight / #ROMs). Procedural cases are `endstate` (only the last frame vs a tight threshold, so close to all-or-nothing); replays are `frame_mean` of a per-frame sigmoid.

| section (weight) | subsystem (weight in section) | testcases | overall weight per testcase |
|---|---|---|---|
| procedural (0.20) | cpu (0.35) | aw-arm-alu, aw-arm-ldr-str, aw-arm-ldm-stm, aw-thumb-alu, aw-thumb-ldr-str, aw-thumb-ldm-stm, mgba-shifter, mgba-carry, mgba-multiply-long (9) | 0.0078 |
| | memory (0.15) | memory, mgba-io-read (2) | 0.0150 |
| | comprehensive (0.20) | none in corpus (weight unreachable) | - |
| | timer (0.12) | timer-reload, timer-start-stop (2) | 0.0120 |
| | dma (0.08) | mgba-dma, dma-priority (2) | 0.0080 |
| | halt_irq (0.10) | ppu-vram-mirror (1) | 0.0200 |
| replay (0.60) | gameplay (1.0) | celeste, spout, heartwrench, piugba, waimanu, goodboy, chip-advance, bulletgba, varooom-3d, collie-defense, tonc-snd1 (11) | 0.0545 (video) |
| audio (0.20) | psg (0.5) | tonc-snd1, chip-advance (2) | 0.0500 |
| | game_audio (0.5) | celeste, heartwrench, piugba, goodboy, bulletgba, varooom-3d, collie-defense (7) | 0.0143 |

Priorities: (1) the 11 gameplay replays are 60%: whole-system correctness (CPU, PPU, DMA, IRQ, timing) with matching video; PPU matters most. (2) CPU tests give ~7% and boot-ability. (3) PSG audio is only 2 ROMs but 10% of total; game_audio 10% across 7 ROMs. `comprehensive` (4% overall) has no testcase, so the practical max is about 0.96 unless the grader scores it some other way.
