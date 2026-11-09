# gba-emu

An original Game Boy Advance emulator written from scratch in Rust, built as a `wasm32-unknown-unknown` cdylib that implements the gba-eval wasm ABI (`src/lib.rs`). Modules: ARM7TDMI CPU, PPU (modes 0-5, sprites, affine, windows, blending, mosaic), APU (PSG + direct sound), bus (memory map, waitstates, timers, DMA, IRQ, save types). Contract between modules: `ARCH.md`.

Best graded overall: **0.9343** (procedural 0.800, replay 0.958, audio 0.997), from a stub baseline of 0.0085 and the repo's partial baseline of about 0.53. Full history: `.claude-flow/data/scores.md`; source snapshots of each new best: `.cache/snapshots/`.

## Build / run / grade
```
export PATH=/opt/homebrew/opt/rustup/bin:$PATH
scripts/build.sh                      # -> target/wasm32-unknown-unknown/release/gba_emu.wasm
scripts/precompute.sh                 # once: grader build + reference cache (see GRADING.md)
scripts/grade.sh                      # ~1 min; results/<ts>/, scores.md line, worst-testcase report, regression guard
cargo run --release --example run -- rom.gba 600 out.ppm   # native run, writes a frame (also: trace, dump, steps, ppu_test)
scripts/lockstep.sh rom.gba           # per-instruction (pc, cycles) comparison vs the reference wasm
```

## Layout
- `src/cpu.rs` CPU; `src/ppu.rs` video; `src/apu.rs` audio; `src/bus.rs` + `src/bus/save.rs` memory map, timers, DMA, IRQ, SRAM/Flash/EEPROM; `src/emu.rs` machine loop; `src/lib.rs` wasm ABI.
- `assets/bios.bin` BIOS image included into the wasm.
- `examples/` native tools; `tests/` integration tests; `scripts/` build and grading.
- `results/` graded runs; `GRADING.md` grading how-to and scoring weights.
- Other docs: `PLAN.md`, `DECISIONS.md`, `KNOWN_LIMITATIONS.md`, `NEXT_STEPS.md`.

## Disclosure
- The grader's reference `gba-eval/reference/mesen.wasm` was used only as a black-box oracle (running it and comparing traces/frames/cycle counts, via the grader and `scripts/lockstep.sh`), mainly by the cpu and bus agents. No Mesen source code was read or copied.
- `gba-eval/candidates/gba-core` (a prior baseline emulator) was never read.
- No git was used in this project; snapshots are tarballs in `.cache/snapshots/`.
- `gba-eval/corpus/reference-cache/*.refcache` were git-LFS pointers and git-lfs is unavailable, so they were regenerated in place with `grader --precompute` using `reference/mesen.wasm`. No other file in gba-eval was modified.
- Tools: rustc 1.99.1, cargo 1.99.1, node v22.23.2, ruflo 3.72.1 (agent memory/coordination only; not a build dependency), python3 for scoring scripts, macOS Darwin 25.5 arm64.
- Human time spent and model/token usage are not available to the team and are therefore not reported.
- No credentials are present in this repository.
- Provenance: `assets/bios.bin` is a copy of the benchmark-provided `gba-eval/spec/gba_bios_stub.bin` (see DECISIONS.md).

### Additional disclosure (round 2)
- Findings measured by driving `reference/mesen.wasm` as a black box (inputs/outputs and per-instruction timing only; no Mesen source was read): keys applied with a one-frame lag (`emu_set_keys` latched at the END of `run_frame`), IRQ/halt-wake delay of 6 cycles, event-DMA starting at the CPU's next non-sequential/data access, audio sample rate `32768 >> ...` per SOUNDBIAS resolution bits and a DC-blocker coefficient 0.9962, VCOUNT/DISPSTAT read latency of 8 cycles, ROM-to-non-ROM branch refetch costs.
- Process: parallel module agents (cpu, bus, ppu, apu, grader) coordinated via `ARCH.md` and ruflo shared memory. Complete AI conversation exports are not available to the author of this repo handoff; per-agent reports are summarized in `PLAN.md` / `DECISIONS.md`.
- PPU window wrap-around (`x1 > x2`, `y1 > y2`) was verified on synthetic ROMs by comparing frames with `mesen.wasm` as a black box.
- Review: an independent read-only review (Opus) of logic and code found a reset-determinism bug, an audio buffer cap, and several test-fitted constants; fixes and the remaining fitted constants are listed in `DECISIONS.md` and `KNOWN_LIMITATIONS.md`.
- Disk note: `results/` accumulates ~237 MB of PNGs per graded run; delete old runs (keep the JSON summaries).
