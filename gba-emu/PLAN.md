# Plan and how it changed

1. **Scaffold (lead).** Rust cdylib implementing the gba-eval ABI, `ARCH.md` as the contract: file ownership per agent, exact public signatures (`Ppu::tick/read_io/write_io`, `Apu::tick/drain/fifo_write/timer_overflow`, `Cpu::step`, `Bus::read*/write*/idle`). Stub wasm scored **0.0085**.
2. **Parallel module agents.** cpu-agent (`cpu.rs`), ppu-agent (`ppu.rs`), apu-agent (`apu.rs`), bus-agent/lead (`bus.rs`, `emu.rs`, `lib.rs`) edited only their own files and kept the crate compiling. Result of the first integrated grade: **0.5728** (replay 0.786, audio 0.428, procedural 0.078).
3. **Grader loop (grader agent).** Native grader built once, reference cache regenerated, `scripts/grade.sh` logging every run to `scores.md` with a worst-testcase report. Procedural jumped to 0.506 then 0.606 as CPU/timer/halt fixes landed (overall 0.6603, 0.6803); 0.6962 was reached with replay 0.850.
4. **Round 2 (current).** Instruction-level lockstep against the reference (`scripts/lockstep.sh`) for CPU cycle timing, DMA/memory-order fixes, handoff docs, and a regression guard in `grade.sh` (delta vs best, loud warning on a drop above 0.01, source snapshot on a new best). Scores logged in this phase were lower than the best (0.5567 to 0.6428) while timing changes were being re-fit; see `scores.md` for the live number.

Scope changes: the original plan assumed HLE BIOS; the implementation executes a BIOS image from `assets/bios.bin` (see DECISIONS.md). The corpus has no `comprehensive` testcases, so 4% of overall is unreachable.
