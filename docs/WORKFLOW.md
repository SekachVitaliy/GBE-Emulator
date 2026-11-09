# Workflow: how the work was done

**Human time: about 2-3 hours (self-reported)**, spread over 5-9 November 2026. The rest was agent time: about 6 h 12 min of active elapsed time with up to three agents in parallel (see `USAGE.md`).

## Roles
- **Human:** set the goal, made environment and scope decisions, approved destructive or outward-facing steps, decided when to stop.
- **Lead agent** (Claude Code, Sonnet): analysed the task, wrote the module contract, dispatched and coordinated the other agents, ran independent verification of every reported score.
- **Module agents** (ruflo-managed, same model): `cpu`, `bus`, `ppu`, `apu`, `grader`, each owning separate files. A second generation (`*-r3`) continued after the session was resumed.
- **Reviewer** (Claude Opus, read-only): independent review of logic and code quality near the end.

## Steps
1. **Kickoff (human):** initialise ruflo in the folder, analyse the project, start a swarm and the autopilot, enable shared agent memory and RAG. The lead read the task, created a 10-item checklist (`.claude-flow/data/checklist.json`) and seeded memory with the task, the ABI and the grading rules.
2. **Environment (human):** installed the Rust toolchain with the `wasm32-unknown-unknown` target when the lead reported it missing.
3. **Skeleton and contract (lead):** a compiling crate with all 10 exports, then `gba-emu/ARCH.md` with the interface of each module and file ownership.
4. **Parallel build (5 agents):** CPU, bus/timers/DMA, PPU, APU and the grading scripts at the same time. First graded result: 0.69.
5. **Fit against the reference (3 rounds):** targeted tasks per agent from the grader's worst testcases. Biggest wins: the one-frame key lag (found by per-frame hash comparison, confirmed by an A/B run: 0.747 to 0.910), IRQ/halt timing, event-triggered DMA timing, the explicit prefetch buffer model, APU resampling. Rule for everyone: run the grader before and after each change and keep only non-regressing changes; `grade.sh` also snapshots each new best.
6. **Independent review (human request, Opus):** found a reset-determinism bug, an audio buffer cap, test-fitted constants and cleanup items; fixed by the owners, with ABI/determinism tests added (`tests/abi.rs`).
7. **Stop and hand over (human decision):** stopped at 0.9343; docs rewritten to match the code; this repository published.

## Intermediate mistakes and failed attempts (kept on purpose)
- A score jump was first attributed to one change; an A/B run on the tree showed it came mostly from another. A later regression turned out to be a real interaction between a CPU-side workaround and a new DMA gate, found by diffing against a source snapshot.
- Concurrent edits repeatedly broke the shared tree for a few minutes (score 0.0000 runs); mitigated with ownership rules and re-verification before accepting a report.
- Grader outputs filled the disk (about 9.8 GB of WAV files in 41 runs); old runs were pruned with the human's approval and the pruning script fixed.
- Rejected approaches: fixed one-instruction IRQ delays (fixed one ROM, broke another), a flat +1 VRAM contention stall (worse), hard-coding multiply-carry tables (replaced by a labelled partial rule).

## What is reproducible
Everything needed to rebuild and re-grade is in this repository (see the root `README.md`). The full agent conversations are not included yet.
