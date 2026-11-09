# Next steps (ordered by score value, at 0.9343)

1. IRQ acceptance rule in VCOUNT-polling loops (heartwrench @342930, waimanu, celeste, goodboy). Needs a rule that depends on more than the event time; collect more per-instruction reference samples (`scripts/lockstep.sh`, `scripts/mesteps.mjs`).
2. `spout` video (0.60, diverges at frame 338): trogdor-style DMA length (71 vs 69), remaining FIFO-refill timing.
3. Prefetch model gaps (bulletgba @276252) and PAL/VRAM contention stalls.
4. VCOUNT-loop phase for `another-world` and `key-demo`.
5. Replace the empirical `mull_carry()` with a derivation of the ARM7TDMI multiplier carry (and verify UMLAL/SMLAL).
6. Hidden-test robustness: more ABI/determinism tests (load == reset, audio pairs per frame, odd ROM sizes), see `tests/abi.rs`.
7. After any timing change run `scripts/grade.sh`: it warns if overall falls more than 0.01 below the best and snapshots the source when a new best is reached.
