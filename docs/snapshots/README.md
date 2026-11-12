# Source snapshots

`score-<overall>-src.tgz`: the tracked source (`src/`, `examples/`, `scripts/`, `ARCH.md`, `Cargo.toml`) as it was when each new best score was first reached. `scripts/grade.sh` creates them automatically; the first one (0.643) was taken by hand before the second round. They are the only intermediate history of the code (no version control was used), so they are a sparse sample, not every edit.

| Snapshot | Taken (EET) |
|---|---|
| score-0.643 | 2026-11-06 15:47 |
| score-0.7159 | 2026-11-06 19:35 |
| score-0.7245 | 2026-11-06 19:39 |
| score-0.7309 | 2026-11-06 19:47 |
| score-0.7389 | 2026-11-06 19:50 |
| score-0.7469 | 2026-11-06 19:51 |
| score-0.9103 | 2026-11-06 19:54 |
| score-0.9111 | 2026-11-06 20:21 |
| score-0.9121 | 2026-11-09 13:27 |
| score-0.9199 | 2026-11-09 13:35 |
| score-0.9272 | 2026-11-09 14:16 |
| score-0.9343 | 2026-11-09 15:02 |

Notes: the 0.9343 snapshot is the state when 0.9343 was first reached; the published `gba-emu/src/` is the final tree after the review fixes (file splits, cleanup, reset fix), which scores the same 0.9343. The `0.696` result mentioned in early notes predates the snapshot feature and has no tarball. Home-directory paths inside the archives were replaced by `/Users/USER`; no other change.
