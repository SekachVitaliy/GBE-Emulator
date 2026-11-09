# Known limitations

State at overall **0.9343** (procedural 0.800, replay 0.958, audio 0.997; `.claude-flow/data/scores.md` has the full history).

## Score ceiling
- The `comprehensive` procedural subsystem (20% of procedural, 4% of overall) has no testcase in the corpus, so the practical ceiling is about 0.96.
- All procedural subsystems with testcases score 1.0.

## Replays (60% of the score)
- Weakest: `spout` (video about 0.60, first divergence at frame 338), then `heartwrench`, `piugba`. No PPU rendering bug was found; remaining differences are cycle-count drift.
- IRQ acceptance in VCOUNT-polling loops is not a plain delay: the reference accepts it after the same instruction of a poll loop while the event-to-instruction-end distance differs, so no single threshold fits (see DECISIONS.md). Open cases: heartwrench @342930, waimanu, celeste, goodboy.
- `trogdor` event DMA is 71 cycles vs 69 in the reference. `another-world` and `key-demo` leave a VCOUNT loop one iteration early (PPU line phase suspected).
- PAL/VRAM contention stalls (+1 cycle in some draw slots) are not modelled. The prefetch buffer is a fitted model (8 halfwords, one fetch in flight); `bulletgba` @276252 is not fitted.

## Reference-fitted, not hardware-derived
These were fitted against `reference/mesen.wasm` used as a black box; see the table in DECISIONS.md. They may not generalise to ROMs outside the corpus.
- `mull_carry()` in `src/cpu.rs`: partial empirical rule for the C flag after UMULLS/SMULLS (matches the oracle on about 91% of random UMULL and 87% of SMULL pairs; covers all 72 pairs used by `mgba-multiply-long`). Accumulate forms (UMLAL/SMLAL) are unverified.
- Timing constants: IRQ_DELAY, EV_X, PPU_IRQ_EARLY, VBLANK_DELAY, START_DELAY, READ_ADJ (`src/bus.rs`), the VCOUNT read latency (`src/ppu.rs`), branch penalties (`src/cpu.rs`), DC-blocker coefficient 0.9962 (`src/apu.rs`).
- Behaviours measured on the oracle: keys take effect one frame late, 6 warmup frames inside `emu_load_rom`, audio sample rate `32768 << SOUNDBIAS resolution`.

## Other
- Save type is detected only from ROM-embedded SDK strings (SRAM, Flash 64K/128K, EEPROM); ROMs without them get no backup memory model beyond the default.
- The grader's fuel limits: `corpus/grader.yaml` sets `fuel_load_rom` to 300G (code default 60G); the worst `emu_load_rom` including warmup uses under 1G. Per-frame headroom against 500M is at least 3.4x (tightest: ppu-vram-mirror, varooom-3d).
- No UI, rewind or link cable. Native examples under `examples/` are developer tools.
- No Git history exists for this project (by the owner's decision); source snapshots of each new best are in `../.cache/snapshots/`.
