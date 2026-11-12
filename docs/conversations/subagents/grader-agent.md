# Subagent: grader-agent

### Agent report · 2026-11-06 13:56
<teammate-message teammate_id="team-lead" summary="Grader tooling and score loop">
You are the GRADER/QA agent in a team building an ORIGINAL GBA emulator (Rust wasm32 cdylib at ~/work/Gbe-test/gba-emu; read gba-emu/ARCH.md). Your job (checklist t08, t09, t10 groundwork): make the grading loop runnable locally and repeatable. The grader lives in gba-eval (Rust workspace, harness/grader, README.md 'How grading works', quickstart/README.md `--native` path that skips docker). Use `export PATH=/opt/homebrew/opt/rustup/bin:$PATH`. Shared agent memory: `cd ~/work/Gbe-test && ./node_modules/.bin/ruflo memory search -q "..." -n shared`; store notes with `memory store -n shared -k grader-<topic> --value ...`.

RULES: no git in this project, never run git or git-lfs. Do NOT read or copy gba-eval/candidates/gba-core sources (it's a prior baseline emulator; originality required; you may, only for confirming the grader toolchain works, not for anything else, avoid even that if possible). Do not modify gba-eval sources (it is the read-only benchmark); put your scripts in gba-emu/scripts/ and docs in gba-emu/GRADING.md. Other agents are editing gba-emu/src concurrently; don't touch src. Disk is nearly full (~15GB free): build the grader natively ONCE with CARGO_TARGET_DIR=~/work/Gbe-test/.cache/grader-target, no docker images, and report disk use.

Facts: corpus/reference-cache/*.refcache files are git-LFS POINTERS (git-lfs unavailable) so they must be regenerated: `cargo run -p grader --release -- --precompute corpus/` using reference/mesen.wasm (a real 1.1MB wasm); check how long/heavy it is (44 replays + test ROMs) and whether outputs are written inside gba-eval/corpus (acceptable: regenerate in place, note it in GRADING.md). Deliver: (a) scripts/build.sh (build gba-emu wasm release), (b) scripts/precompute.sh (one-time reference cache), (c) scripts/grade.sh [wasm] (runs native grader on corpus, writes results under gba-emu/results/<timestamp>/ and appends a one-line summary with per-section scores (procedural/replay/audio/overall) to ~/work/Gbe-test/.claude-flow/data/scores.md), (d) a script/summary that lists the worst-failing testcases per subsystem so devs know what to fix first, (e) GRADING.md documenting all of this, tool versions (rustc, cargo, node) and how an outside reviewer reruns it. Verify the loop by grading the current stub wasm (gba-emu/target/wasm32-unknown-unknown/release/gba_emu.wasm; expect near-zero score, that's fine) and record the number. Also read corpus/testcases.json and write a short table in GRADING.md of testcase categories -> subsystem weights so the team can prioritise. Store the key findings in shared memory (namespace shared, key grader-howto). Finish with a concise report including any blockers.
</teammate-message>

**→ Bash**
```
{"command": "cd ~/work/Gbe-test; ls; ls gba-eval gba-eval/harness gba-eval/corpus | head -60; sed -n 1,200p gba-eval/quickstart/README.md; df -h . | tail -1"}
```

**← Bash result**
```
AGENTS.md
CLAUDE.md
gba-emu
gba-eval
graft
node_modules
package-lock.json
package.json
poetry.lock
pyproject.toml
task.md
gba-eval:
CONTACT.md
Cargo.toml
LEGAL.md
LICENSE
LICENSE-GPL3
LICENSE-MIT
NOTICE
README.md
candidates
corpus
harness
quickstart
reference
spec

gba-eval/corpus:
LICENSE
README.md
grader.yaml
reference-cache
replays
roms
testcases.json

gba-eval/harness:
LICENSE
dump
grader
lockstep
oracle
# GBA Eval Quickstart

This directory contains scripts and files allowing you to either:

1. **Run your own agent in the same container** the benchmark's agents
   saw — `docker compose up -d && ./shell.sh`
2. **Grade a wasm you or an agent produce** — `./grade.sh path/to/my.wasm`

## Zero-to-score (most users start here)

```bash
# from repo root, after cloning
git submodule update --init --recursive
git lfs install && git lfs pull          # pulls corpus/reference-cache/

./quickstart/smoke-test.sh                # 30s preflight — should print all ✓
./quickstart/grade.sh candidates/gba-core/gba_core_shim.wasm baseline
```

First grade: 5-10 min cold (docker builds `gba-eval-grader` the first
time it runs). Subsequent grades: seconds to a minute, depending on
whether the reference cache is warm.

Output lands in `./results/baseline/` — per-testcase JSON, PNG
screenshots, and `summary.json` with the overall score.

## Requirements

Pick one path:

- **Docker (default)** — docker and docker compose.
- **Native (`--native`)** — Rust 1.87, clang, cmake, and a one-time
  `reference/build-mesen.sh native`. Use this if you're hacking on the
  grader.

Either way you also need `git-lfs` for `corpus/reference-cache/`
(~230 MB pull, 51 precomputed reference frames). `smoke-test.sh` will
tell you if LFS didn't fire.

## Running your own agent

The `task` service is the container the benchmark's agents ran inside —
Rust and the wasm target, wasmtime, `/task/spec/`, `/task/dev-roms/`,
`/task/TASK.md`, and an `oracle` CLI that forwards to the reference
emulator sidecar.

```bash
cd quickstart
cp .env.example .env    # add whichever model key your agent needs
docker compose up -d    # builds images on first run

./shell.sh              # interactive shell as uid 1000
./shell.sh cargo build --release --lib --target wasm32-unknown-unknown
./shell.sh oracle info
```

The container starts with no agent CLI installed — you bring your own.
The compose file passes `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`,
`GEMINI_API_KEY`, `GOOGLE_API_KEY`, and `OPENROUTER_API_KEY` through
fr
… [truncated 2701 chars]
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-eval; sed -n 1,400p README.md | grep -n -i -A40 \"How grading works\" | head -120; cat corpus/README.md | head -60; cat quickstart/grade.sh | head -120; ls corpus/replays | head; ls corpus/replays | wc -l; ls corpus/roms | head; ls reference; ls ../gba-emu ../gba-emu/scripts 2>&1; ls ../.claude-flow/data"}
```

**← Bash result**
```
34:## How grading works
35-
36-```bash
37-# One-time: precompute reference frames + per-replay thresholds.
38-# Writes corpus/reference-cache/*.refcache (git-LFS tracked).
39-cargo run -p grader --release -- --precompute corpus/
40-
41-# Per candidate: grade wasm → per-testcase JSON + summary.json
42-cargo run -p grader --release -- candidate.wasm corpus/ results/<candidate>/
43-
44-# Custom reference: any wasm implementing the ABI works. Default is the
45-# bundled Mesen2 build at reference/mesen.wasm.
46-cargo run -p grader --release -- --reference my-ref.wasm candidate.wasm corpus/ results/<candidate>/
47-```
48-
49-## Scoring: three sections
50-
51-The overall score is the weighted sum of three independent sections.
52-Default weights, configurable in [`corpus/grader.yaml`](corpus/grader.yaml):
53-
54-| Section | Weight | What it measures |
55-|---|---|---|
56-| **Gameplay Replays** | 60% | Real gameplay with button presses — end-to-end behavior under input |
57-| **Procedural Tests** | 20% | CPU, memory, timers, DMA — deterministic self-checking ROMs |
58-| **Audio**            | 20% | Per-frame log-mel spectral distance vs Mesen2 |
59-
60-```
61-overall = 0.60 × replay + 0.20 × procedural + 0.20 × audio
62-```
63-
64-## Building from source
65-
66-The bundled `reference/mesen.wasm` is enough to grade. To rebuild it
67-from source (Mesen2 fork at `third_party/mesen`):
68-
69-```bash
70-git submodule update --init --recursive
71-reference/build-mesen.sh wasm      # → reference/mesen.wasm
72-cargo build --workspace
73-```
74-
# Corpus

The test suite: which ROMs, which replays, what each one tests.

---

## `testcases.json`

The grading manifest. Each entry is one (ROM, replay) pair with metadata
for scoring.

```json
{
  "id": "aw-arm-alu",
  "section": "procedural",
  "subsystem": "cpu",
  "audio_subsystem": null,
  "rom_sha256": "9f08d807c03ef296d38ef73e9b827a3d8c77cead9ced44b07d64c10f5f7d0746",
  "rom_name": "test/armwrestler.gba",
  "replay": "aw-arm-alu.txt",
  "frames": 600,
  "scoring_mode": "endstate",
  "description": "Armwrestler ARM ALU — ADC/ADD/AND/BIC/CMN/EOR/MOV/MVN/ORR/RSC/SBC/MLA/MUL/UMULL/SMULL (part 1) + UMLAL/SMLAL/SWP/SWPB/MRS/MSR (part 2)."
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `id` | string | yes | Unique, URL-safe. Used in result paths (`results/<candidate>/<id>.json`). |
| `section` | string | yes | Which scored section this belongs to: `procedural` or `replay`. (Audio is scored from any testcase 
… [truncated 7838 chars]
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-eval; sed -n 120,220p quickstart/grade.sh; cat corpus/grader.yaml; python3 - <<'E'\nimport json,collections\nt=json.load(open('corpus/testcases.json'))\nt=t if isinstance(t,list) else t.get('testcases',t)\nc=collections.Counter((x['section'],x['subsystem'],x.get('audio_subsystem')) for x in t)\nfor k,v in sorted(c.items(),key=str):print(k,v)\nprint(len(t))\nE\nexport PATH=/opt/homebrew/opt/rustup/bin:$PATH; rustc -V; cargo -V; node -V; rustup target list --installed; ls ../gba-emu/target; ls -la corpus/reference-cache | head -5; du -sh corpus/reference-cache"}
```

**← Bash result**
```
"${REF_MOUNTS[@]}" \
        -v "$OUT":/out \
        gba-eval-grader "${REF_ARGS[@]}" /wasm/"$WASM_NAME" corpus/ /out

else
    command -v cargo >/dev/null 2>&1 || { echo "error: cargo not found (needed for --native)." >&2; exit 1; }
    if [ ! -d corpus/reference-cache ] || [ -z "$(ls -A corpus/reference-cache 2>/dev/null)" ]; then
        echo "note: corpus/reference-cache/ is empty — precomputing (one-time, minutes)." >&2
        cargo run -p grader --release -- --precompute corpus/
    fi
    WASM_ABS="$(cd "$(dirname "$WASM")" && pwd)/$(basename "$WASM")"
    REF_FLAG=()
    if [ -n "$REFERENCE" ]; then
        REF_ABS="$(cd "$(dirname "$REFERENCE")" && pwd)/$(basename "$REFERENCE")"
        REF_FLAG=(--reference "$REF_ABS")
        echo "── Grading $WASM (native) ref=$REFERENCE → $OUT ──"
    else
        echo "── Grading $WASM (native) → $OUT ──"
    fi
    cargo run -p grader --release -- "${REF_FLAG[@]}" "$WASM_ABS" corpus/ "$OUT"
fi

# ── Summary ─────────────────────────────────────────────────────────
SUMMARY="$OUT/summary.json"
if [ -f "$SUMMARY" ]; then
    echo
    echo "── summary ──"
    if command -v jq >/dev/null 2>&1; then
        jq . "$SUMMARY"
        OVERALL=$(jq -r '.overall // empty' "$SUMMARY" 2>/dev/null || true)
        [ -n "$OVERALL" ] && echo && echo "overall: $OVERALL"
    else
        cat "$SUMMARY"
    fi
fi
# Grader configuration. Defaults to <corpus_dir>/grader.yaml.
# All fields are optional — see GraderConfig in main.rs for defaults.

# Escape hatch for candidates that need extra warmup on top of their
# own boot_frames() value. Conformant candidates leave this at 0.
candidate_boot_frames: 0

fuel_per_frame: 500000000
fuel_load_rom: 300000000000

# ── Section weights ──────────────────────────────────────────────
# Three sections, each [0,1], combined into overall score.
# overall = Σ(section_weight * section_score)
section_weights:
  procedural: 0.20
  replay: 0.60
  audio: 0.20

# ── Subsystem weights (within each section) ──────────────────────
# ROMs are grouped by subsystem. Each subsystem gets a fixed weight.
# Adding more ROMs to a subsystem improves measurement precision
# without shifting other subsystems' weights.
subsystem_weights:
  procedural:
    cpu: 0.35
    memory: 0.15
    comprehensive: 0.20
    timer: 0.12
    dma: 0.08
    halt_irq: 0.10
  replay:
    gameplay: 1.00
  audio:
    psg: 0.50
    game_audio: 0.50
('procedural', 'cpu', None) 9
('procedural', 'dma', None) 2
('procedural', 'halt_irq', N
… [truncated 640 chars]
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-eval; export PATH=/opt/homebrew/opt/rustup/bin:$PATH; python3 -c \"\nimport json;t=json.load(open('corpus/testcases.json'));t=t if isinstance(t,list) else t.get('testcases',t)\nfor x in t:print(x['id'],x['section'],x['subsystem'],x.get('audio_subsystem'),x['frames'],x['scoring_mode'] if 'scoring_mode' in x else '')\n\"; ls corpus/reference-cache | wc -l; ls corpus/roms/*|head -50; (time CARGO_TARGET_DIR=~/work/Gbe-test/.cache/grader-target cargo build -p grader --release 2>&1 | tail -15)"}
```

**← Bash result**
```
aw-arm-alu procedural cpu None 600 endstate
aw-arm-ldr-str procedural cpu None 600 endstate
aw-arm-ldm-stm procedural cpu None 400 endstate
aw-thumb-alu procedural cpu None 400 endstate
aw-thumb-ldr-str procedural cpu None 500 endstate
aw-thumb-ldm-stm procedural cpu None 500 endstate
memory procedural memory None 60 endstate
mgba-io-read procedural memory None 600 endstate
mgba-shifter procedural cpu None 600 endstate
mgba-carry procedural cpu None 600 endstate
mgba-multiply-long procedural cpu None 600 endstate
mgba-dma procedural dma None 600 endstate
timer-reload procedural timer None 60 endstate
timer-start-stop procedural timer None 60 endstate
dma-priority procedural dma None 60 endstate
ppu-vram-mirror procedural halt_irq None 1200 endstate
celeste-gameplay replay gameplay game_audio 7439 
spout-gameplay replay gameplay None 4475 
heartwrench-gameplay replay gameplay game_audio 7326 
tonc-snd1 replay gameplay psg 60 
piugba-gameplay replay gameplay game_audio 1937 
waimanu-gameplay replay gameplay None 4086 
goodboy-gameplay replay gameplay game_audio 5650 
chip-advance-gameplay replay gameplay psg 3523 
bulletgba-gameplay replay gameplay game_audio 4467 
varooom-3d-gameplay replay gameplay game_audio 7976 
collie-defense-gameplay replay gameplay game_audio 6427 
      51
corpus/roms/homebrew:
anguna.gba
another-world.gba
blindjump.gba
bulletgba.gba
celeste-classic.gba
chip-advance.gba
collie-defense.gba
goodboy-advance.gba
heartwrench-advance.gba
meteorain.gba
piugba.gba
spout.gba
trogdor.gba
varooom-3d.gba
waimanu.gba
xniq.gba

corpus/roms/test:
armwrestler.gba
destoer
fuzzarm.gba
jsmolka
mgba-suite.gba
misc
nba-hw
tonc
   Compiling smallvec v1.16.4
   Compiling postcard v1.1.3
   Compiling bincode v1.3.3
   Compiling regalloc2 v0.10.2
   Compiling cranelift-bforest v0.114.0
   Compiling wasmparser v0.219.2
   Compiling serde_yaml v0.9.34+deprecated
   Compiling zstd v0.13.4
   Compiling wasmprinter v0.219.2
   Compiling wasmtime-environ v27.0.0
   Compiling cranelift-native v0.114.0
   Compiling cranelift-frontend v0.114.0
   Compiling wasmtime-cranelift v27.0.0
   Compiling grader v0.1.0 (~/work/Gbe-test/gba-eval/harness/grader)
    Finished `release` profile [optimized] target(s) in 49.97s
CARGO_TARGET_DIR=~/work/Gbe-test/.cache/grader-target cargo buil  104.56s user 8.64s system 226% cpu 50.013 total
tail -15  0.00s user 0.00s system 0% cpu 50.013 total
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-eval; export PATH=/opt/homebrew/opt/rustup/bin:$PATH; G=~/work/Gbe-test/.cache/grader-target/release/grader; $G --help 2>&1 | head -30; du -sh ~/work/Gbe-test/.cache; (time $G --precompute corpus/ 2>&1 | tail -15); ls -la corpus/reference-cache | head -5; du -sh corpus/reference-cache"}
```

**← Bash result**
```
Error: unknown flag: --help
410M	~/work/Gbe-test/.cache
  run  tonc-snd1 (60 frames)
  run  dma-priority (60 frames)
  run  spout-gameplay (4475 frames)
  run  ppu-vram-mirror (1200 frames)
  run  varooom-3d-gameplay (7976 frames)
  run  waimanu-gameplay (4086 frames)
  run  aw-thumb-ldm-stm (500 frames)
  run  collie-defense-gameplay (6427 frames)
  run  bulletgba-gameplay (4467 frames)
  run  mgba-multiply-long (600 frames)
  run  mgba-shifter (600 frames)
  run  goodboy-gameplay (5650 frames)

precompute: 27 written, 0 skipped, 0 errors
51 cache files, 219.1 MB on disk
$G --precompute corpus/ 2>&1  781.65s user 3.77s system 494% cpu 2:38.70 total
tail -15  0.00s user 0.00s system 0% cpu 2:38.70 total
total 465496
drwxr-xr-x@ 53 USER  staff      1696 Nov  5 18:47 .
drwxr-xr-x@  9 USER  staff       288 Nov  5 18:47 ..
-rw-r--r--@  1 USER  staff   1031171 Nov  6 13:57 aw-arm-alu.refcache
-rw-r--r--@  1 USER  staff    287150 Nov  6 13:57 aw-arm-ldm-stm.refcache
227M	corpus/reference-cache
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-emu; ls -la target/wasm32-unknown-unknown/release/*.wasm; mkdir -p scripts; export PATH=/opt/homebrew/opt/rustup/bin:$PATH; G=~/work/Gbe-test/.cache/grader-target/release/grader; mkdir -p /tmp/x 2>/dev/null; O=$(mktemp -d); (time $G target/wasm32-unknown-unknown/release/gba_emu.wasm ../gba-eval/corpus $O 2>&1 | tail -8); ls $O | head; cat $O/summary.json | head -c 1500; f=$(ls $O/*.json | grep -v summary | head -1); head -c 800 $f; echo $O"}
```

**← Bash result**
```
-rwxr-xr-x@ 1 USER  staff  21391 Nov  6 13:58 target/wasm32-unknown-unknown/release/gba_emu.wasm
config: reference=reference/mesen.wasm
Error: reading reference wasm "reference/mesen.wasm"

Caused by:
    No such file or directory (os error 2)
$G target/wasm32-unknown-unknown/release/gba_emu.wasm ../gba-eval/corpus $O 2  0.00s user 0.00s system 68% cpu 0.002 total
tail -8  0.00s user 0.00s system 41% cpu 0.002 total
cat: /var/folders/cy/fjp0hj050ysc_1rs7g1tyvx80000gn/T/tmp.6WU0LZr0Gf/summary.json: No such file or directory
(eval):1: no matches found: /var/folders/cy/fjp0hj050ysc_1rs7g1tyvx80000gn/T/tmp.6WU0LZr0Gf/*.json
/var/folders/cy/fjp0hj050ysc_1rs7g1tyvx80000gn/T/tmp.6WU0LZr0Gf
```

### Assistant · 2026-11-06 14:00
Reference path is relative to cwd; scripts will pass `--reference` absolutely. Writing scripts.

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-emu/scripts\ncat > env.sh <<'E'\n# sourced by the other scripts\nexport PATH=/opt/homebrew/opt/rustup/bin:$PATH\nROOT=~/work/Gbe-test\nEMU=$ROOT/gba-emu\nEVAL=$ROOT/gba-eval\nGRADER=$ROOT/.cache/grader-target/release/grader\nWASM_DEFAULT=$EMU/target/wasm32-unknown-unknown/release/gba_emu.wasm\nSCORES=$ROOT/.claude-flow/data/scores.md\nE\ncat > build.sh <<'E'\n#!/usr/bin/env bash\n# Build gba-emu release wasm. Usage: scripts/build.sh\nset -euo pipefail\n. \"$(dirname \"$0\")/env.sh\"\ncd \"$EMU\"\ncargo build --release --lib --target wasm32-unknown-unknown\nls -l \"$WASM_DEFAULT\"\nE\ncat > precompute.sh <<'E'\n#!/usr/bin/env bash\n# One-time: build grader (if needed) and regenerate reference caches in place\n# (gba-eval/corpus/reference-cache; the committed files are git-LFS pointers).\nset -euo pipefail\n. \"$(dirname \"$0\")/env.sh\"\nif [ ! -x \"$GRADER\" ]; then\n  (cd \"$EVAL\" && CARGO_TARGET_DIR=$ROOT/.cache/grader-target cargo build -p grader --release)\nfi\ncd \"$EVAL\"\nif [ \"${1:-}\" != \"--force\" ] && [ -f corpus/reference-cache/aw-arm-alu.refcache ] \\\n   && ! head -c 40 corpus/reference-cache/aw-arm-alu.refcache | grep -q '^version https://git-lfs'; then\n  echo \"reference cache already real; pass --force to redo\"; exit 0\nfi\ntime \"$GRADER\" --reference \"$EVAL/reference/mesen.wasm\" --precompute corpus/\nE\ncat > grade.sh <<'E'\n#!/usr/bin/env bash\n# Usage: scripts/grade.sh [wasm]   (default: target/.../gba_emu.wasm)\
… [truncated 1454 chars]
```

**← Bash result**
```
└──────────────────────────────────────────────────

╔══════════════════════════════════════════════════════════╗
║  OVERALL: 0.0085                                        ║
╠══════════════════════════════════════════════════════════╣
║  procedural   0.0000 ░░░░░░░░░░░░░░░░░░░░
║  replay       0.0141 ░░░░░░░░░░░░░░░░░░░░
║  audio        0.0000 ░░░░░░░░░░░░░░░░░░░░
╚══════════════════════════════════════════════════════════╝
written to "/var/folders/cy/fjp0hj050ysc_1rs7g1tyvx80000gn/T/tmp.Zv3glCZYiu"
~/work/Gbe-test/.cache/grader-target/release/grader --reference   217.62s user 4.01s system 724% cpu 30.582 total
tail  0.00s user 0.00s system 0% cpu 30.582 total
aw-arm-alu.cand.png
aw-arm-alu.json
aw-arm-alu.ref.png
aw-arm-ldm-stm.cand.png
aw-arm-ldm-stm.json
{
  "candidate": "gba_emu",
  "candidate_sha256": "dbae01a6a16cd1521c95425ffc79660c98bf1d319770c3b4e364a9aea767a614",
  "graded_at": "2026-11-06T12:00:41Z",
  "per_testcase": {
    "aw-arm-alu": {
      "video_score": 9.999998989515043e-17,
      "section": "procedural",
      "subsystem": "cpu"
    },
    "aw-arm-ldm-stm": {
      "video_score": 9.999998989515043e-17,
      "section": "procedural",
      "subsystem": "cpu"
    },
    "memory": {
      "video_score": 9.999998989515043e-17,
      "section": "procedural",
      "subsystem": "memory"
    },
    "aw-thumb-ldr-str": {
      "video_score": 9.999998989515043e-17,
      "section": "procedural",
      "subsystem": "cpu"
    },
    "mgba-shifter": {
      "video_score": 9.999998989515043e-17,
      "section": "procedural",
      "subsystem": "cpu"
    },
    "mgba-carry": {
      "video_score": 9.999998989515043e-17,
      "section": "procedural",
      "subsystem": "cpu"
    },
    "mgba-dma": {
      "video_score": 9.999998989515043e-17,
      "section": "procedural",
      "subsystem": "dma"
    },
    "aw-arm-ldr-str": {
      "video_score": 9.999998989515043e-17,
      "section": "procedural",
      "subsystem": "cpu"
    },
    "celeste-gameplay": {
      "video_score": 0.00833802001542501,
      "audio_score": 0.0,
      "section": "replay",
      "subsystem": "gameplay"
    },
    "tonc-snd1": {
      "video_score": 0.000048101474450216,
      "audio_score": 0.0,
      "section": "replay",
      "subsystem": "gameplay"
    },
    "aw-thumb-alu": {
      "video_score": 9.999998989515043e-17,
      "section": "procedural",
      "subsystem": "cpu"
    },
    "goodboy-gameplay": {
      "video_score": 0.014958767220899654,
      "audio_score
… [truncated 969 chars]
```

**→ Bash**
```
{"command": "cd /tmp; O=/var/folders/cy/fjp0hj050ysc_1rs7g1tyvx80000gn/T/tmp.Zv3glCZYiu; python3 -c \"\nimport json;s=json.load(open('$O/summary.json'));print({k:v for k,v in s.items() if k!='per_testcase'})\""}
```

**← Bash result**
```
{'candidate': 'gba_emu', 'candidate_sha256': 'dbae01a6a16cd1521c95425ffc79660c98bf1d319770c3b4e364a9aea767a614', 'graded_at': '2026-11-06T12:00:41Z', 'sections': {'procedural': {'score': 7.999999191612034e-17, 'weight': 0.2, 'subsystems': {'memory': 9.999998989515043e-17, 'comprehensive': 0.0, 'cpu': 9.99999898951504e-17, 'halt_irq': 9.999998989515043e-17, 'dma': 9.999998989515043e-17, 'timer': 9.999998989515043e-17}}, 'audio': {'score': 0.0, 'weight': 0.2, 'subsystems': {'psg': 0.0, 'game_audio': 0.0}}, 'replay': {'score': 0.014104651633281085, 'weight': 0.6, 'subsystems': {'gameplay': 0.014104651633281085}}}, 'overall': 0.008462790979968667}
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-emu/scripts\ncat > worst.py <<'E'\n#!/usr/bin/env python3\n\"\"\"worst.py <results_dir> [--line NAME]: per-subsystem worst testcases, or a one-line scores.md entry.\"\"\"\nimport json, sys, os, collections\nd = sys.argv[1]\ns = json.load(open(os.path.join(d, \"summary.json\")))\nsec = s[\"sections\"]\nif \"--line\" in sys.argv:\n    n = sys.argv[sys.argv.index(\"--line\") + 1]\n    f = lambda k: sec[k][\"score\"]\n    print(f\"- {s['graded_at']} {n} sha={s['candidate_sha256'][:8]} procedural={f('procedural'):.4f} \"\n          f\"replay={f('replay'):.4f} audio={f('audio'):.4f} overall={s['overall']:.4f} dir={d}\")\n    sys.exit()\nprint(f\"overall {s['overall']:.4f}  \" + \"  \".join(f\"{k}={v['score']:.4f}(w{v['weight']})\" for k, v in sec.items()))\ng = collections.defaultdict(list)\nfor t, v in s[\"per_testcase\"].items():\n    g[(v[\"section\"], v[\"subsystem\"])].append((v[\"video_score\"], v.get(\"audio_score\"), t))\nfor (sc, sub), L in sorted(g.items()):\n    w = None\n    print(f\"\\n[{sc}/{sub}] subsystem score {sec[sc]['subsystems'].get(sub, 0):.4f}\")\n    for vs, a, t in sorted(L, key=lambda x: x[0])[:5]:\n        try:\n            j = json.load(open(os.path.join(d, t + \".json\")))\n            ex = f\" first_diverge={j.get('first_diverge_frame')} diverging={j.get('diverging_frames')}/{j.get('n_frames')}\"\n        except Exception:\n            ex = \"\"\n        print(f\"  {t:28s} video={vs:.4f}\" + (f\" audio={a:.4f}\" if a
… [truncated 222 chars]
```

**← Bash result**
```
║  procedural   0.0000 ░░░░░░░░░░░░░░░░░░░░
║  replay       0.0141 ░░░░░░░░░░░░░░░░░░░░
║  audio        0.0000 ░░░░░░░░░░░░░░░░░░░░
╚══════════════════════════════════════════════════════════╝
written to "~/work/Gbe-test/gba-emu/results/20261106T140124"
overall 0.0085  replay=0.0141(w0.6)  procedural=0.0000(w0.2)  audio=0.0000(w0.2)

[procedural/cpu] subsystem score 0.0000
  aw-arm-ldm-stm               video=0.0000 first_diverge=1 diverging=399/400
  mgba-shifter                 video=0.0000 first_diverge=0 diverging=600/600
  aw-arm-ldr-str               video=0.0000 first_diverge=1 diverging=599/600
  mgba-carry                   video=0.0000 first_diverge=0 diverging=600/600
  aw-arm-alu                   video=0.0000 first_diverge=1 diverging=599/600

[procedural/dma] subsystem score 0.0000
  dma-priority                 video=0.0000 first_diverge=1 diverging=59/60
  mgba-dma                     video=0.0000 first_diverge=0 diverging=600/600

[procedural/halt_irq] subsystem score 0.0000
  ppu-vram-mirror              video=0.0000 first_diverge=0 diverging=1163/1200

[procedural/memory] subsystem score 0.0000
  memory                       video=0.0000 first_diverge=0 diverging=60/60
  mgba-io-read                 video=0.0000 first_diverge=0 diverging=600/600

[procedural/timer] subsystem score 0.0000
  timer-reload                 video=0.0000 first_diverge=0 diverging=60/60
  timer-start-stop             video=0.0000 first_diverge=0 diverging=60/60

[replay/gameplay] subsystem score 0.0141
  tonc-snd1                    video=0.0000 audio=0.0000 first_diverge=0 diverging=60/60
  collie-defense-gameplay      video=0.0002 audio=0.0000 first_diverge=0 diverging=6426/6427
  waimanu-gameplay             video=0.0002 audio=0.0000 first_diverge=0 diverging=4085/4086
  celeste-gameplay             video=0.0083 audio=0.0000 first_diverge=0 diverging=7378/7439
  spout-gameplay               video=0.0111 audio=0.0000 first_diverge=0 diverging=4430/4475
results: ~/work/Gbe-test/gba-emu/results/20261106T140124
scripts/grade.sh 2>&1  480.50s user 5.11s system 757% cpu 1:04.11 total
tail -40  0.00s user 0.00s system 0% cpu 1:04.11 total
- 2026-11-06T12:01:24Z gba_emu.wasm sha=3a77152e procedural=0.0000 replay=0.0141 audio=0.0000 overall=0.0085 dir=~/work/Gbe-test/gba-emu/results/20261106T140124
237M	results
4.1G	..
410M	~/work/Gbe-test/.cache
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-emu\ncat >> scripts/grade.sh <<'E'\n# disk is tight: each run's PNGs are ~230MB; keep PNGs only for the 2 newest runs\nls -1d \"$EMU\"/results/*/ | head -n -2 | while read -r d; do rm -f \"$d\"/*.png; done\nE\n# macOS head lacks negative -n; use sed\nsed -i '' 's/head -n -2/sed -e :a -e '\"'\"'$d;N;2,2ba'\"'\"' -e '\"'\"'P;D'\"'\"'/' scripts/grade.sh; tail -2 scripts/grade.sh\nls -1d results/*/ | sed -e :a -e '$d;N;2,2ba' -e 'P;D'\ncat > GRADING.md <<'E'\n# Grading gba-emu locally\n\nNative grader from `gba-eval` (read-only; no docker, no git-lfs). Source of truth: `gba-eval/README.md` \"How grading works\".\n\n## Commands (all in `gba-emu/scripts/`)\n| script | what |\n|---|---|\n| `build.sh` | `cargo build --release --lib --target wasm32-unknown-unknown` -> `target/wasm32-unknown-unknown/release/gba_emu.wasm` |\n| `precompute.sh [--force]` | builds grader once (`CARGO_TARGET_DIR=<repo>/.cache/grader-target`, ~50 s) and regenerates reference caches |\n| `grade.sh [wasm]` | grades (default wasm above) against `gba-eval/corpus` with `reference/mesen.wasm`; writes `results/<timestamp>/` (per-testcase JSON, cand/ref PNGs, `summary.json`, `grader.log`); appends a line to `~/work/Gbe-test/.claude-flow/data/scores.md`; prints the worst-testcases report; deletes PNGs of all but the 2 newest runs |\n| `worst.py <results_dir>` | per-subsystem worst testcases with `first_diverge` frame (fix lowest/earliest first); `--line NAME` prints the scores.md l
… [truncated 4365 chars]
```

**← Bash result**
```
Permission for this command was denied by a built-in Claude Code safety check, not by the user. The check stops removals that can delete far more than intended: a system, home or workspace directory, or a target it cannot resolve, such as a shell variable that, if unset or empty, turns this into `rm -rf /` or `rm -rf /*`. Only a person may approve such a removal, and no person did (the permission prompt timed out, or none was shown). The command was NOT run; do not claim it succeeded. Do not work around the check by splitting, scripting, or re-issuing the removal through another tool or shell: the check exists because a removal like this can destroy the user's data, and getting past it would not make it safe. If the text below suggests a safe rewrite, run that instead; it goes through the same check. Otherwise finish the rest of the task without this removal, tell the user what you wanted to delete and why, and leave the removal to them. What was flagged: Dangerous rm operation detected in `rm -f "$d"/*.png`. The target '"$d"/*.png' is a shell variable expansion: when $d is unset or empty it becomes `/`, `/*` or a top-level path. This requires explicit approval and cannot be auto-allowed by permission rules.

This check does not fire on a target that cannot expand to the filesystem root: rewrite it as `"${d:?}"/*.png`, which makes the shell stop with an error instead of running rm when $d is unset or empty, or use a literal absolute path.
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-emu; ls scripts GRADING.md 2>&1; tail -3 scripts/grade.sh"}
```

**← Bash result**
```
ls: GRADING.md: No such file or directory
scripts:
build.sh
env.sh
grade.sh
precompute.sh
worst.py
python3 "$EMU/scripts/worst.py" "$OUT" --line "$(basename "$WASM")" >> "$SCORES"
python3 "$EMU/scripts/worst.py" "$OUT"
echo "results: $OUT"
```

### Assistant · 2026-11-06 14:03
Nothing ran. Redo with safe rm form via Edit/Write.

**→ Edit**
```
{"replace_all": false, "file_path": "~/work/Gbe-test/gba-emu/scripts/grade.sh", "old_string": "echo \"results: $OUT\"", "new_string": "echo \"results: $OUT\"\n# disk is tight: each run's PNGs are ~230MB; keep PNGs only for the 2 newest runs\nfor d in $(ls -1dr \"$EMU\"/results/*/ | tail -n +3); do rm -f \"${d:?}\"*.png; done"}
```

**→ Write**
```
{"file_path": "~/work/Gbe-test/gba-emu/GRADING.md", "content": "# Grading gba-emu locally\n\nNative grader from `gba-eval` (read-only; no docker, no git-lfs). Source of truth: `gba-eval/README.md` \"How grading works\".\n\n## Commands (all in `gba-emu/scripts/`)\n| script | what |\n|---|---|\n| `build.sh` | `cargo build --release --lib --target wasm32-unknown-unknown` -> `target/wasm32-unknown-unknown/release/gba_emu.wasm` |\n| `precompute.sh [--force]` | builds grader once (`CARGO_TARGET_DIR=<repo>/.cache/grader-target`, ~50 s) and regenerates reference caches |\n| `grade.sh [wasm]` | grades (default wasm above) against `gba-eval/corpus` with `reference/mesen.wasm`; writes `results/<timestamp>/` (per-testcase JSON, cand/ref PNGs, `summary.json`, `grader.log`); appends a line to `~/work/Gbe-test/.claude-flow/data/scores.md`; prints the worst-testcases report; deletes PNGs of all but the 2 newest runs |\n| `worst.py <results_dir>` | per-subsystem worst testcases with `first_diverge` frame (fix earliest-diverging first); `--line NAME` prints the scores.md line |\n| `env.sh` | shared paths, rustup PATH |\n\nTypical loop: `scripts/build.sh && scripts/grade.sh` (about 1 min for the 27-case corpus on a multi-core Mac; slower once the emulator does real work).\n\n## Reference cache (one-time, done)\n`gba-eval/corpus/reference-cache/*.refcache` were git-LFS pointers. `precompute.sh` regenerated them **in place inside gba-eval/corpus** (51 files, 227 MB, 2m40s, ~5 cores busy). Redo on
… [truncated 3146 chars]
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test; ./node_modules/.bin/ruflo memory store -n shared -k grader-howto --value \"Grading: gba-emu/scripts/{build,precompute,grade}.sh + worst.py; see gba-emu/GRADING.md. Grader prebuilt at .cache/grader-target; refcache regenerated in place (done). grade.sh ~1 min, appends to .claude-flow/data/scores.md, prints worst cases per subsystem. Stub baseline overall 0.0085. Weights: replay 0.6 (11 gameplay ROMs), procedural 0.2 (cpu .35 mem .15 timer .12 dma .08 halt_irq .10; comprehensive has no ROM), audio 0.2 (psg 2 ROMs, game_audio 7). Procedural=endstate (all-or-nothing).\" 2>&1 | tail -3; bash -n gba-emu/scripts/grade.sh && echo ok"}
```

**← Edit result**
```
The file ~/work/Gbe-test/gba-emu/scripts/grade.sh has been updated successfully.
```

**← Write result**
```
File created successfully at: ~/work/Gbe-test/gba-emu/GRADING.md (file state is current in your context — no need to Read it back)
```

**← Bash result**
```
+------------+----------------------+

[OK] Data stored successfully
ok
```

### Assistant · 2026-11-06 14:03
The grading loop runs locally and repeatably. The current stub wasm scores **overall 0.0085** (procedural 0.0000, replay 0.0141, audio 0.0000), as expected. There were no blockers. I never opened `candidates/gba-core`, and I ran no git or git-lfs commands.

- **Scripts** (in `~/work/Gbe-test/gba-emu/scripts/`):
  - `build.sh` builds the release wasm.
  - `precompute.sh` builds the grader once and regenerates the reference cache.
  - `grade.sh [wasm]` grades the wasm, writes `gba-emu/results/<timestamp>/`, appends a line to `.claude-flow/data/scores.md`, and prints the worst failures. That line carries the per-section scores plus the date, wasm name, hash and results directory.
  - `worst.py` lists the worst-failing testcases per subsystem with each one's first diverging frame, so devs can fix the earliest-diverging first. `env.sh` holds shared paths.
- **Docs:** `~/work/Gbe-test/gba-emu/GRADING.md` covers all of the above, tool versions, how an outside reviewer reruns it, and the weights table. I also stored the key findings in shared memory under `grader-howto`.
- **Grader build:** it builds in 50 s with rustc/cargo 1.99.1, even though gba-eval's docs say 1.87. It lives in `~/work/Gbe-test/.cache/grader-target` (410 MB).
- **Reference cache:** the `.refcache` files were git-LFS pointers, so I regenerated all 51 in place under `gba-eval/corpus/reference-cache`. That took 2m40s and the cache is now 227 MB. This is the only thing written inside gba-eval, and it needs redoing only if the reference wasm or `testcases.json` changes.
- **Grading time:** one full grade takes about 64 s.
- **Disk:** each results run is about 237 MB, mostly PNGs, and `grade.sh` deletes PNGs from all but the 2 newest runs. About 18 GB was free at the start.
- **Weights:** overall is 0.60 replay + 0.20 procedural + 0.20 audio. All 11 replays are `gameplay` (video, plus audio on 9 of them). Procedural cases check only the last frame, so they are nearly all-or-nothing.
  - **Procedural weights:** cpu 0.35 with 9 ROMs, memory 0.15 with 2, timer 0.12 with 2, dma 0.08 with 2, halt_irq 0.10 with 1.
  - **Audio weights:** psg 0.5 with 2 ROMs, game_audio 0.5 with 7.
- **Corpus gap:** the `comprehensive` subsystem is 20% of procedural, or 4% of overall, and no testcase uses it. The practical maximum is therefore about 0.96.
- **Minor quirks:** the grader resolves `--reference` relative to the current directory, so the scripts pass an absolute path. `testcases.json` has 27 cases, not the 44 replay files.

### Agent report · 2026-11-06 19:26
<teammate-message teammate_id="team-lead" summary="Round 2: handoff docs and regression guard (t10)">
Round 2, task t10 (handoff docs) plus a regression guard.
1) Write in ~/work/Gbe-test/gba-emu: README.md (what it is, build/run/grade in 5 commands, layout), PLAN.md (original plan and how it changed: scaffold, parallel module agents with ARCH.md contract, grader loop, round 2), DECISIONS.md (key decisions: module split, 6 warmup frames in reset, open_bus approach, 1006-cycle HBlank, save-type detection, refcache regenerated in place because git-lfs unavailable), KNOWN_LIMITATIONS.md, NEXT_STEPS.md. Be factual; pull numbers from .claude-flow/data/scores.md and results/.
2) A 'Disclosure' section: mesen.wasm (the grader's reference) was used only as a black-box oracle for trace/frame comparison by the cpu and bus agents, no Mesen code was read or copied; gba-eval/candidates/gba-core was never read; no git in this project; gba-eval/corpus/reference-cache regenerated in place; tools/versions (rustc, cargo, node, ruflo 3.72.1); human time and model usage are not available to us (say so). No credentials.
3) Regression guard: make scripts/grade.sh print the delta vs the best previous overall in scores.md and warn loudly if overall dropped by more than 0.01, and save the source snapshot automatically (tar of src/ examples/ scripts/ to .cache/snapshots/score-<overall>-src.tgz) whenever a new best is reached.
No git. Don't touch src/. Report briefly when done.
</teammate-message>

**→ Bash**
```
{"command": "cd ~/work/Gbe-test; cat .claude-flow/data/scores.md; ls gba-emu gba-emu/src gba-emu/results; wc -l gba-emu/src/*.rs; cat gba-emu/ARCH.md | head -80; ./node_modules/.bin/ruflo --version 2>&1 | tail -1; node -v; ls .cache/snapshots 2>&1; ls gba-emu/examples 2>&1"}
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test; ./node_modules/.bin/ruflo memory search -q \"decision warmup open_bus hblank save type\" -n shared 2>&1 | head -60; ls gba-emu/*.md"}
```

**← Bash result**
```
- 2026-11-06T12:01:24Z gba_emu.wasm sha=3a77152e procedural=0.0000 replay=0.0141 audio=0.0000 overall=0.0085 dir=~/work/Gbe-test/gba-emu/results/20261106T140124
- 2026-11-06T12:10:30Z gba_emu.wasm sha=7187284d procedural=0.0778 replay=0.7860 audio=0.4280 overall=0.5728 dir=~/work/Gbe-test/gba-emu/results/20261106T141030
- 2026-11-06T12:17:45Z gba_emu.wasm sha=9e6d08ae procedural=0.5061 replay=0.7891 audio=0.4280 overall=0.6603 dir=~/work/Gbe-test/gba-emu/results/20261106T141745
- 2026-11-06T12:28:58Z gba_emu.wasm sha=663a5a87 procedural=0.6061 replay=0.7891 audio=0.4280 overall=0.6803 dir=~/work/Gbe-test/gba-emu/results/20261106T142858
- 2026-11-06T13:02:34Z gba_emu.wasm sha=b97339ae procedural=0.4861 replay=0.8495 audio=0.4465 overall=0.6962 dir=~/work/Gbe-test/gba-emu/results/20261106T150234
- 2026-11-06T13:04:39Z gba_emu.wasm sha=4c645504 procedural=0.4861 replay=0.8495 audio=0.4465 overall=0.6962 dir=~/work/Gbe-test/gba-emu/results/20261106T150439
- 2026-11-06T13:25:31Z gba_emu.wasm sha=3347b2a0 procedural=0.6061 replay=0.6255 audio=0.3007 overall=0.5567 dir=~/work/Gbe-test/gba-emu/results/20261106T152531
- 2026-11-06T13:27:30Z gba_emu.wasm sha=a59fc313 procedural=0.6061 replay=0.6255 audio=0.3007 overall=0.5567 dir=~/work/Gbe-test/gba-emu/results/20261106T152730
- 2026-11-06T13:31:58Z gba_emu.wasm sha=e9fb40f0 procedural=0.6061 replay=0.7200 audio=0.4476 overall=0.6428 dir=~/work/Gbe-test/gba-emu/results/20261106T153158
- 2026-11-06T13:42:03Z gba_emu.wasm sha=bb164adb procedural=0.6061 replay=0.7200 audio=0.4476 overall=0.6428 dir=~/work/Gbe-test/gba-emu/results/20261106T154203
- 2026-11-06T13:45:42Z gba_emu.wasm sha=bb164adb procedural=0.6061 replay=0.7200 audio=0.4476 overall=0.6428 dir=~/work/Gbe-test/gba-emu/results/20261106T154542
gba-emu:
ARCH.md
Cargo.lock
Cargo.toml
GRADING.md
assets
examples
results
scripts
src
target
tests

gba-emu/results:
20261106T140124
20261106T141030
20261106T141745
20261106T142858
20261106T150234
20261106T150439
20261106T152531
20261106T152730
20261106T153158
20261106T154203
20261106T154542
20261106T192634

gba-emu/src:
apu.rs
bus
bus.rs
cpu.rs
emu.rs
lib.rs
ppu.rs
     427 gba-emu/src/apu.rs
     592 gba-emu/src/bus.rs
    1161 gba-emu/src/cpu.rs
      89 gba-emu/src/emu.rs
      95 gba-emu/src/lib.rs
     485 gba-emu/src/ppu.rs
    2849 total
# gba-emu architecture (contract between modules)

Crate: wasm32 cdylib, `std` available, no threads/fs. Frame = 280896 cycles @16.78MHz (228 lines x 1232 cycles).
Files and own
… [truncated 5272 chars]
```

**← Bash result**
```
ruflo installation is incomplete: @claude-flow/cli could not be found from ~/work/Gbe-test/node_modules/ruflo. Stop running Ruflo MCP servers and their hosts, then reinstall with npm install -g ruflo@latest.
gba-emu/ARCH.md
gba-emu/GRADING.md
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-emu; ls scripts tests assets; cat scripts/grade.sh | tail -15; python3 scripts/worst.py results/20261106T192634 2>&1 | head -60; ls results/20261106T192634 | head -3; ls ../.claude-flow/data; grep -n \"save\\|Flash\\|EEPROM\" src/bus/*.rs | head -8; ls src/bus"}
```

**← Bash result**
```
assets:
bios.bin

scripts:
build.sh
env.sh
grade.sh
lockstep.sh
mesteps.mjs
precompute.sh
worst.py

tests:
m4.rs
[ -x "$GRADER" ] || { echo "grader not built: run scripts/precompute.sh" >&2; exit 1; }
if head -c 40 "$EVAL/corpus/reference-cache/aw-arm-alu.refcache" | grep -q '^version https://git-lfs'; then
  echo "refcache is LFS pointers: run scripts/precompute.sh" >&2; exit 1; fi
TS=$(date +%Y%m%dT%H%M%S)
OUT=$EMU/results/$TS
mkdir -p "$OUT"
cd "$EVAL"
"$GRADER" --reference "$EVAL/reference/mesen.wasm" "$WASM" corpus/ "$OUT" 2>&1 | tee "$OUT/grader.log" | tail -5 || true
[ -f "$OUT/summary.json" ] || { echo "no summary.json, see $OUT/grader.log" >&2; exit 1; }
mkdir -p "$(dirname "$SCORES")"
python3 "$EMU/scripts/worst.py" "$OUT" --line "$(basename "$WASM")" >> "$SCORES"
python3 "$EMU/scripts/worst.py" "$OUT"
echo "results: $OUT"
# disk is tight: each run's PNGs are ~230MB; keep PNGs only for the 2 newest runs
for d in $(ls -1dr "$EMU"/results/*/ | tail -n +3); do rm -f "${d:?}"*.png; done
Traceback (most recent call last):
  File "~/work/Gbe-test/gba-emu/scripts/worst.py", line 5, in <module>
    s = json.load(open(os.path.join(d, "summary.json")))
                  ~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
FileNotFoundError: [Errno 2] No such file or directory: 'results/20261106T192634/summary.json'
dma-priority.cand.png
dma-priority.json
dma-priority.ref.png
auto-memory-store.json
autopilot-log.json
autopilot-state.json
checklist.json
classic-loop.json
pending-insights.jsonl
scores.md
verified-trajectories.jsonl
1://! Cartridge backup memory: SRAM, Flash 64K/128K, EEPROM 512B/8K.
3:pub enum Kind { Sram, Flash64, Flash128, Eeprom }
20:        let kind = if find(rom, b"FLASH1M_V") { Kind::Flash128 }
21:            else if find(rom, b"FLASH512_V") || find(rom, b"FLASH_V") { Kind::Flash64 }
22:            else if find(rom, b"EEPROM_V") { Kind::Eeprom }
27:        let (n, fill) = match kind { Kind::Sram => (0x8000, 0xFF), Kind::Flash64 => (0x10000, 0xFF), Kind::Flash128 => (0x20000, 0xFF), Kind::Eeprom => (0x2000, 0xFF) };
35:            Kind::Flash64 | Kind::Flash128 => {
37:                    let id: [u8; 2] = if self.kind == Kind::Flash128 { [0x62, 0x13] } else { [0x32, 0x1B] };
save.rs
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-emu; ls -l assets; grep -rn \"bios\" src/*.rs | head -5; grep -n \"1006\\|warmup\" src/*.rs src/bus/*.rs | head; cat Cargo.toml; python3 scripts/worst.py results/20261106T154542 | head -50; ls examples scripts/lockstep.sh"}
```

**← Bash result**
```
total 32
-rw-r--r--@ 1 USER  staff  16384 Nov  6 13:54 bios.bin
src/bus.rs:8:static BIOS: &[u8] = include_bytes!("../assets/bios.bin");
src/bus.rs:33:    /// protected (return `bios_latch`) when it is >= 0x4000.
src/bus.rs:37:    bios_latch: u32,
src/bus.rs:83:            fetch_pc: 0x0800_0000, open_bus: 0, bios_latch: 0xE129_F000,
src/bus.rs:103:        self.fetch_pc = 0x0800_0000; self.open_bus = 0; self.bios_latch = 0xE129_F000;
src/emu.rs:20:    pub warmup_frames: usize,
src/emu.rs:28:            carry: 0, pending: Vec::new(), warmup_frames: 6,
src/emu.rs:51:            for _ in 0..self.warmup_frames { self.run_frame(); }
src/bus.rs:379:            let d = d.min(self.next_timer_event()).min(if lp < 1006 { 1006 - lp } else { 1232 - lp });
src/ppu.rs:3://! HBlank begins at cycle 1006 of each 1232-cycle line (960 + 46 delay), the line is rendered then.
src/ppu.rs:12:const HB: u32 = 1006;
src/lib.rs:78:    m.warmup_frames = 0;
[package]
name = "gba-emu"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
overall 0.6428  audio=0.4476(w0.2)  replay=0.7200(w0.6)  procedural=0.6061(w0.2)

[procedural/cpu] subsystem score 0.8889
  mgba-multiply-long           video=0.0000 first_diverge=2 diverging=489/600
  mgba-shifter                 video=1.0000 first_diverge=2 diverging=8/600
  aw-arm-alu                   video=1.0000 first_diverge=2 diverging=4/600
  aw-thumb-alu                 video=1.0000 first_diverge=2 diverging=16/400
  aw-arm-ldr-str               video=1.0000 first_diverge=2 diverging=12/600

[procedural/dma] subsystem score 0.0003
  dma-priority                 video=0.0001 first_diverge=1 diverging=59/60
  mgba-dma                     video=0.0006 first_diverge=2 diverging=375/600

[procedural/halt_irq] subsystem score 1.0000
  ppu-vram-mirror              video=1.0000 first_diverge=0 diverging=3/1200

[procedural/memory] subsystem score 0.5000
  mgba-io-read                 video=0.0000 first_diverge=2 diverging=576/600
  memory                       video=1.0000 first_diverge=None diverging=0/60

[procedural/timer] subsystem score 1.0000
  timer-start-stop             video=1.0000 first_diverge=None diverging=0/60
  timer-reload                 video=1.0000 first_diverge=1 diverging=1/60

[replay/gameplay] subsystem score 0.7200
  celeste-gameplay             video=0.2038 audio=0.7806 first_diverge=2 diverging=7061/7439
  spout-gameplay      
… [truncated 426 chars]
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-emu; shasum assets/bios.bin; xxd assets/bios.bin | head -4; grep -c . assets/bios.bin >/dev/null; strings -n 6 assets/bios.bin | head; grep -rln \"bios.bin\" . --include=* 2>/dev/null | grep -v target | head; tail -4 ../.claude-flow/data/scores.md | cut -c1-60; ls ~/work/Gbe-test/.cache/snapshots"}
```

**← Bash result**
```
0ba8ec6cee4f62d3dff599b56b0d18d396bb681a  assets/bios.bin
00000000: 9404 00ea a304 00ea 5400 00ea a104 00ea  ........T.......
00000010: a004 00ea 0000 0000 4200 00ea 9d04 00ea  ........B.......
00000020: 0000 0000 0000 0000 0000 0000 0000 0000  ................
00000030: 0000 0000 0000 0000 0000 0000 0000 0000  ................
$"(%'(
y}z]{*|
|*|]{}z
+'((%$"
(eval):1: no matches found: --include=*
- 2026-11-06T13:27:30Z gba_emu.wasm sha=a59fc313 procedural=
- 2026-11-06T13:31:58Z gba_emu.wasm sha=e9fb40f0 procedural=
- 2026-11-06T13:42:03Z gba_emu.wasm sha=bb164adb procedural=
- 2026-11-06T13:45:42Z gba_emu.wasm sha=bb164adb procedural=
score-0.643-src.tgz
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-emu; python3 - <<'E'\np='scripts/grade.sh'\ns=open(p).read()\nold='python3 \"$EMU/scripts/worst.py\" \"$OUT\" --line \"$(basename \"$WASM\")\" >> \"$SCORES\"\\n'\nnew='''# regression guard: compare with best previous overall, snapshot source on a new best\nNEW=$(python3 -c 'import json,sys;print(\"%.4f\"%json.load(open(sys.argv[1]))[\"overall\"])' \"$OUT/summary.json\")\nBEST=$(grep -o 'overall=[0-9.]*' \"$SCORES\" 2>/dev/null | cut -d= -f2 | sort -g | tail -1)\nBEST=${BEST:-0}\npython3 \"$EMU/scripts/worst.py\" \"$OUT\" --line \"$(basename \"$WASM\")\" >> \"$SCORES\"\npython3 - \"$NEW\" \"$BEST\" <<'PY'\nimport sys\nn, b = float(sys.argv[1]), float(sys.argv[2])\nprint(f\"\\\\noverall {n:.4f}  best-previous {b:.4f}  delta {n-b:+.4f}\")\nif b - n > 0.01:\n    print(\"!\" * 70 + f\"\\\\n!!! REGRESSION: overall dropped {b-n:.4f} below best previous ({b:.4f}) !!!\\\\n\" + \"!\" * 70)\nPY\nif python3 -c 'import sys;sys.exit(not float(sys.argv[1])>float(sys.argv[2]))' \"$NEW\" \"$BEST\"; then\n  mkdir -p \"$ROOT/.cache/snapshots\"\n  tar czf \"$ROOT/.cache/snapshots/score-$NEW-src.tgz\" -C \"$EMU\" src examples scripts\n  echo \"new best: snapshot .cache/snapshots/score-$NEW-src.tgz\"\nfi\n'''\nassert old in s\nopen(p,'w').write(s.replace(old,new))\nE\nbash -n scripts/grade.sh && echo ok\n# test the guard logic against the last good results dir with a fake scores file\ncp ../.claude-flow/data/scores.md /tmp/scores.bak 2>/dev/null; sed -n '/REGRES
… [truncated 38 chars]
```

**← Bash result**
```
ok
    print("!" * 70 + f"\n!!! REGRESSION: overall dropped {b-n:.4f} below best previous ({b:.4f}) !!!\n" + "!" * 70)
PY
if python3 -c 'import sys;sys.exit(not float(sys.argv[1])>float(sys.argv[2]))' "$NEW" "$BEST"; then
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-emu; BEST=$(grep -o 'overall=[0-9.]*' ../.claude-flow/data/scores.md | cut -d= -f2 | sort -g | tail -1); echo best=$BEST; python3 - 0.6428 $BEST <<'PY'\nimport sys\nn, b = float(sys.argv[1]), float(sys.argv[2])\nprint(f\"\\noverall {n:.4f}  best-previous {b:.4f}  delta {n-b:+.4f}\")\nif b - n > 0.01:\n    print(\"!\" * 70 + f\"\\n!!! REGRESSION: overall dropped {b-n:.4f} below best previous ({b:.4f}) !!!\\n\" + \"!\" * 70)\nPY\npython3 -c 'import sys;sys.exit(not float(sys.argv[1])>float(sys.argv[2]))' 0.70 $BEST && echo newbest"}
```

**← Bash result**
```
best=0.6962

overall 0.6428  best-previous 0.6962  delta -0.0534
!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
!!! REGRESSION: overall dropped 0.0534 below best previous (0.6962) !!!
!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
newbest
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-emu\ncat > README.md <<'E'\n# gba-emu\n\nAn original Game Boy Advance emulator written from scratch in Rust, built as a `wasm32-unknown-unknown` cdylib that implements the gba-eval wasm ABI (`src/lib.rs`). Modules: ARM7TDMI CPU, PPU (modes 0-5, sprites, affine, windows, blending, mosaic), APU (PSG + direct sound), bus (memory map, waitstates, timers, DMA, IRQ, save types). Contract between modules: `ARCH.md`.\n\nBest graded overall so far: **0.6962** (procedural 0.486, replay 0.850, audio 0.447); latest logged: 0.6428. See `.claude-flow/data/scores.md` (one line per graded run, so the current state may be higher).\n\n## Build / run / grade\n```\nexport PATH=/opt/homebrew/opt/rustup/bin:$PATH\nscripts/build.sh                      # -> target/wasm32-unknown-unknown/release/gba_emu.wasm\nscripts/precompute.sh                 # once: grader build + reference cache (see GRADING.md)\nscripts/grade.sh                      # ~1 min; results/<ts>/, scores.md line, worst-testcase report, regression guard\ncargo run --release --example run -- rom.gba 600 out.ppm   # native run, writes a frame (also: trace, dump, steps, ppu_test)\nscripts/lockstep.sh rom.gba           # per-instruction (pc, cycles) comparison vs the reference wasm\n```\n\n## Layout\n- `src/cpu.rs` CPU; `src/ppu.rs` video; `src/apu.rs` audio; `src/bus.rs` + `src/bus/save.rs` memory map, timers, DMA, IRQ, SRAM/Flash/EEPROM; `src/emu.rs` machine loop; `src/lib.rs` wasm ABI.\n- `assets/bi
… [truncated 7744 chars]
```

**← Bash result**
```
ARCH.md
Cargo.lock
Cargo.toml
DECISIONS.md
GRADING.md
KNOWN_LIMITATIONS.md
NEXT_STEPS.md
PLAN.md
README.md
assets
examples
results
scripts
src
target
tests
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-emu; python3 - <<'E'\np='KNOWN_LIMITATIONS.md';s=open(p).read()\ns=s.replace(\"Native examples are developer tools; there is no UI, rewind, or link cable. Open-source audio resampling is simple (see apu.rs); no per-sample high-pass modelling beyond what apu.rs documents.\",\"Native examples are developer tools; there is no UI, rewind, or link cable. Audio accuracy is whatever apu.rs implements (not independently audited here).\")\nopen(p,'w').write(s)\nE\ngrep -c audited KNOWN_LIMITATIONS.md"}
```

**← Bash result**
```
1
```

### Assistant · 2026-11-06 19:28
The five docs and the regression guard are written. I tested the guard's comparison logic on its own with `0.6428` against the logged best; a full `grade.sh` run has not exercised it yet.

**Docs** (all in `~/work/Gbe-test/gba-emu/`):
- `README.md` has the five build, run and grade commands, the layout, and the Disclosure section you asked for.
  - It covers mesen.wasm as a black-box oracle only, `gba-core` never read, and no git.
  - It also covers the refcache regenerated in place.
  - Tool versions are rustc 1.99.1, cargo 1.99.1, node v22.23.2 and ruflo 3.72.1.
  - Human time and model usage are stated as unavailable, and there are no credentials.
- `PLAN.md`, `DECISIONS.md`, `KNOWN_LIMITATIONS.md` and `NEXT_STEPS.md` use numbers from `scores.md` and `results/`. The best overall is 0.6962, the stub baseline is 0.0085, and the last logged run is 0.6428.

**Regression guard in `scripts/grade.sh`:**
- Each run prints the overall score, the best previous overall, and the delta.
- It prints a loud `!!! REGRESSION` banner if overall drops more than 0.01 below the best.
- On a new best it writes `.cache/snapshots/score-<overall>-src.tgz`, a tar of `src`, `examples` and `scripts`.

**Open items:**
- **BIOS image:** `assets/bios.bin` is 16 KB with a vector table that differs from the retail BIOS. Whoever wrote it needs to say how it was produced, and `DECISIONS.md` should record that. I flagged it in the README and `NEXT_STEPS.md`.
- **No snapshot for the best run:** the 0.6962 run predates the snapshot feature, so there is no tarball for it. The only existing snapshot is `score-0.643`.
- **Unverified claims:** I did not verify the warmup claim that `lib.rs` sets `warmup_frames = 0` for a path that must not warm up. I wrote that line in `DECISIONS.md` from a grep. `KNOWN_LIMITATIONS.md` uses the last fully recorded per-testcase run (`results/20261106T154542`), because `results/20261106T192634` has no `summary.json`.
- **ruflo:** `ruflo memory` currently fails with "installation is incomplete", so I could not read shared memory this round. The version number comes from your message.

