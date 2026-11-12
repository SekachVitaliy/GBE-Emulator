# Subagent: opus-reviewer

### Agent report · 2026-11-09 15:43
<teammate-message teammate_id="team-lead" summary="Opus review of GBA emulator">
You are an independent senior reviewer. Review the ORIGINAL Game Boy Advance emulator in ~/work/Gbe-test/gba-emu (Rust, wasm32-unknown-unknown cdylib, ABI in ~/work/Gbe-test/gba-eval/spec/ABI.md, task in ~/work/Gbe-test/task.md). Current graded score: overall 0.9343 (scripts/grade.sh; GRADING.md). Read README.md, ARCH.md, DECISIONS.md, KNOWN_LIMITATIONS.md, NEXT_STEPS.md first. Build/test: `export PATH=/opt/homebrew/opt/rustup/bin:$PATH; cd gba-emu; cargo build --release --target wasm32-unknown-unknown; cargo test --release`.

READ-ONLY REVIEW: do NOT edit any file under gba-emu/ and do not run grade.sh (other work may be in flight; a review report is the only output). You may write scratch files only under your scratchpad. Never read gba-eval/candidates/gba-core. No git in this project. Other agents may still be editing src/ concurrently; note the time you read each file.

Review on two axes, ranked by severity with file:line and a concrete failure scenario for each finding:
A) LOGIC / CORRECTNESS vs hardware and the grader: ABI conformance (all 10 exports; rom buffer >=32 MiB stable pointer; emu_load_rom/emu_reset identical state, including the 6 warmup frames and the 1-frame key lag; emu_audio_samples drain semantics and buffer overflow; emu_set_keys latching; framebuffer format), determinism and hidden-test robustness (code that is fitted to specific test ROMs or operands rather than hardware behaviour: e.g. mull_carry partial fit, IRQ_DELAY/LAT constants, any ROM-specific hacks or magic numbers, save-type detection, behaviour with ROMs not in the corpus such as larger ROMs, odd sizes, 32 MiB, ROMs with no save, unsupported modes), traps/panics (panic=abort means any panic or out-of-bounds is a wasm trap that loses frames; look for unwrap, indexing, integer overflow/underflow in release=wrapping vs debug, shifts >=32, division), fuel/performance cliffs, state that survives reset incorrectly, uninitialised/stale buffers, 16 MiB vs 32 MiB ROM mirroring and open-bus beyond ROM size.
B) CODE QUALITY: unsafe/static mut usage in lib.rs, dead debug code (env::var, MEMLOG, eprintln, trace hooks) in non-test paths, file size (project rule: files < 500 lines; list offenders), duplicated logic, test coverage gaps (what would silently break the score), fragile cross-module contracts (e.g. fetch16/fetch32 overrides in cpu.rs, sync_to(self.mark) rule), documentation vs reality mismatches (claims in README/DECISIONS that the code contradicts, stale numbers), originality/compliance risks for the handoff (anything that looks copied, reading of forbidden sources, or undisclosed oracle usage).

Deliver a concise report (<= 60 lines): top 5 must-fix items first, then the rest grouped by A/B, and a short list of what looks solid. Be specific and skeptical; do not pad with praise.
</teammate-message>

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-emu; date; ls; wc -l src/*.rs src/**/*.rs 2>/dev/null | sort -n; wc -l *.md; cat Cargo.toml", "description": "List project files and line counts"}
```

**← Bash result**
```
Mon Nov  9 15:43:35 EET 2026
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
      91 src/emu.rs
      91 src/emu.rs
      95 src/lib.rs
      95 src/lib.rs
     119 src/bus/save.rs
     457 src/apu.rs
     457 src/apu.rs
     499 src/ppu.rs
     499 src/ppu.rs
     732 src/bus.rs
     732 src/bus.rs
    1226 src/cpu.rs
    1226 src/cpu.rs
    6319 total
      55 ARCH.md
      20 DECISIONS.md
      55 GRADING.md
      23 KNOWN_LIMITATIONS.md
      14 NEXT_STEPS.md
       8 PLAN.md
      37 README.md
     212 total
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
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-emu; cat README.md ARCH.md DECISIONS.md KNOWN_LIMITATIONS.md NEXT_STEPS.md; ls src src/* tests scripts examples; cat ../gba-eval/spec/ABI.md", "description": "Read docs and ABI spec"}
```

**← Bash result**
```
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
- Tools: rustc 1.99.1, cargo 1.99.1, node v22.23.2, ruflo 3.72.1 (agent memor
… [truncated 18688 chars]
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-emu; date; cat -n src/lib.rs src/emu.rs src/bus/save.rs; cat GRADING.md; cat ../task.md | head -80", "description": "Read lib.rs, emu.rs, save.rs, grading and task"}
```

**← Bash result**
```
Mon Nov  9 15:43:42 EET 2026
     1	//! Original GBA emulator core. C ABI per gba-eval/spec/ABI.md.
     2	pub mod apu;
     3	pub mod bus;
     4	pub mod cpu;
     5	pub mod emu;
     6	pub mod ppu;
     7	use emu::Emu;
     8	
     9	// ponytail: single global instance; wasm is single-threaded, so static mut is safe here.
    10	static mut EMU: Option<Emu> = None;
    11	
    12	#[allow(static_mut_refs)]
    13	fn e() -> &'static mut Emu {
    14	    unsafe { EMU.get_or_insert_with(Emu::new) }
    15	}
    16	
    17	#[no_mangle] pub extern "C" fn emu_init() -> i32 { e(); 1 }
    18	#[no_mangle] pub extern "C" fn emu_rom_buffer() -> *mut u8 { e().rom_ptr() }
    19	#[no_mangle] pub extern "C" fn emu_load_rom(len: i32) -> i32 { e().load_rom(len as usize) as i32 }
    20	#[no_mangle] pub extern "C" fn emu_reset() -> i32 { e().reset() as i32 }
    21	#[no_mangle] pub extern "C" fn emu_set_keys(k: u32) { e().keys = k; }
    22	#[no_mangle] pub extern "C" fn emu_run_frame() { e().run_frame(); }
    23	#[no_mangle] pub extern "C" fn emu_framebuffer() -> *const u32 { e().fb.as_ptr() }
    24	#[no_mangle] pub extern "C" fn emu_audio_buffer() -> *const i16 { e().audio.as_ptr() }
    25	#[no_mangle] pub extern "C" fn emu_audio_samples() -> i32 { e().drain_audio() }
    26	#[no_mangle] pub extern "C" fn emu_audio_rate() -> i32 { emu::AUDIO_RATE }
    27	
    28	/// Native helper for examples/run.rs: run `frames` frames, write a PPM, print a report.
    29	#[allow(dead_code)]
    30	pub fn run(rom: &[u8], frames: usize, keys: &str, out: &str) {
    31	    // keys: hex mask held for all frames, or a schedule "hexKEYxFRAMES,..." (e.g. "0x20,1x2,0x30")
    32	    let sched: Vec<(u32, usize)> = if std::path::Path::new(keys).is_file() {
    33	        // replay file: "<frame> <keyshex>" lines
    34	        let ev: Vec<(usize, u32)> = std::fs::read_to_string(keys).unwrap().lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()).map(|l| { let mut w = l.split_whitespace(); (w.next().unwrap().parse().unwrap(), u32::from_str_radix(w.next().unwrap(), 16).unwrap()) }).collect();
    35	        ev.iter().enumerate().map(|(i, &(f, k))| (k, ev.get(i + 1).map_or(usize::MAX, |n| n.0) .saturating_sub(f))).collect()
    36	    } else if keys.contains('x') { keys.split(',').map(|t| { let (k, n) = t.split_once('x').unwrap(); (u32::from_str_radix(k, 16).unwrap_or(0), n.parse().unwrap()) }).collect() } else { vec![(u32::from_str_radix(keys, 16).unwrap_or(0), usize::MAX)] };
    3
… [truncated 22154 chars]
```

**→ Read**
```
{"file_path": "~/work/Gbe-test/gba-emu/src/bus.rs"}
```

**← Read result**
```
1	//! Memory bus (lead-owned): memory map, waitstates, IO dispatch, IRQ regs, timers, DMA.
2	//! Also owns Ppu and Apu; `sync()` advances every component by the cycles the CPU consumed.
3	pub mod save;
4	use crate::apu::Apu;
5	use crate::ppu::Ppu;
6	use save::Save;
7	
8	static BIOS: &[u8] = include_bytes!("../assets/bios.bin");
9	const PRESCALE: [u32; 4] = [0, 6, 8, 10];
10	// calibration knobs (checked against nba-hw timer tests)
11	const VCOUNT_ADJ: u64 = 0;
12	const VBLANK_DELAY: u32 = 8;
13	const ABORT_ON_DISABLE: bool = false;
14	const IRQ_DELAY: u64 = 6;
15	const PF_CAP: u64 = 8;
16	const PF_MISS_EXTRA: u32 = 0;
17	const START_DELAY: u32 = 1;
18	const READ_ADJ: u32 = 2;
19	// prefetch fit (bulletgba/heartwrench lockstep): a branch waits for an in-flight halfword fetch only if it already started;
20	// a data read from ROM restarts the producer N+1 cycles after the access ends
21	const PF_DATA_WAIT: bool = false;
22	const PF_DATA_RESTART: u64 = 1;
23	// a halfword/word data read from ROM discards the prefetch buffer (verified on xniq/bulletgba/collie lockstep)
24	const FLSZ: u32 = 2;
25	const PPU_IRQ_EARLY: u64 = 2;
26	const EV_X: u64 = 5;
27	const NWAIT: [u32; 4] = [4, 3, 2, 8];
28	
29	#[derive(Clone, Copy, Default)]
30	struct Timer { reload: u16, counter: u16, ctrl: u16, acc: u32, on: bool, delay: u32 }
31	#[derive(Clone, Copy, Default)]
32	struct Dma { src: u32, dst: u32, left: u32 }
33	
34	pub struct Bus {
35	    pub cycles: u64,
36	    synced: u64,
37	    pub ppu: Ppu,
38	    pub apu: Apu,
39	    pub rom: Vec<u8>,
40	    pub rom_len: usize,
41	    pub save: Save,
42	    /// CPU may set this to the address of the executing instruction; BIOS reads are
43	    /// protected (return `bios_latch`) when it is >= 0x4000.
44	    pub fetch_pc: u32,
45	    /// Value returned for unmapped reads; CPU may update it with the last prefetched opcode.
46	    pub open_bus: u32,
47	    bios_latch: u32,
48	    ewram: Vec<u8>,
49	    iwram: Vec<u8>,
50	    io: [u16; 0x200],
51	    ie: u16, iflag: u16, ime: u16,
52	    halt: bool,
53	    /// just left halt: first instruction costs +2 and cannot take the IRQ (cleared by the next sync)
54	    woke: bool, wake_pen: bool,
55	    /// batching: components are only ticked once `cycles` reaches `next_ev` (next timer/line/256-cycle event); `mark` = cycles at the last instruction boundary
56	    next_ev: u64, mark: u64, eq: u64, ppu_early: u64,
57	    postflg: u8,
58	    waitcnt: u16,
59	    rom_n: [u32; 3], rom_s: [u32; 3], sra
… [truncated 38124 chars]
```

**→ Bash**
```
{"command": "date", "description": "Record time of bus.rs read"}
```

**← Bash result**
```
Mon Nov  9 15:43:52 EET 2026
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-emu; date; grep -nE 'env::var|eprintln|println|unwrap|expect\\(|static mut|unsafe|panic|unreachable|todo!|MEMLOG|trace' src/*.rs src/bus/*.rs | grep -v '^src/lib.rs:(3|4|5|6|7|8|9)[0-9]:' ; echo ---; grep -n 'restart\\|PF_MISS_EXTRA\\|ABORT_ON_DISABLE\\|PF_DATA_WAIT\\|VCOUNT_ADJ\\|irq_age\\|peek32' src/*.rs | head -40", "description": "Grep risky patterns and dead knobs"}
```

**← Bash result**
```
Mon Nov  9 15:45:06 EET 2026
src/bus.rs:67:    pub trace_mem: bool,
src/bus.rs:105:            keys: 0, key_cond: false, frame_done: false, cond_since: u64::MAX, ppu_pos: 0, trace_mem: false, memlog: Vec::new(), timers: [Timer::default(); 4], dma: [Dma::default(); 4],
src/bus.rs:189:        if self.trace_mem { self.memlog.push((self.ppu_pos + (self.cycles - self.synced) as u32, r, sz, write)); }
src/bus.rs:237:        if self.trace_mem { self.memlog.push((self.ppu_pos + (now - self.synced) as u32, a >> 24, sz, false)); }
src/cpu.rs:1112:        fn read32(&mut self, a: u32, _: bool) -> u32 { self.cyc += 1; let a = (a & !3) as usize; u32::from_le_bytes(self.m[a..a + 4].try_into().unwrap()) }
src/cpu.rs:1184:        eprintln!("60M steps: {:?} cyc={}", t.elapsed(), m.cyc);
src/cpu.rs:1190:        let data = std::fs::read(std::env::var("ROM").unwrap()).unwrap();
src/cpu.rs:1191:        let frames: u64 = std::env::var("FRAMES").map(|v| v.parse().unwrap()).unwrap_or(60);
src/cpu.rs:1199:        let nosync = std::env::var("NOSYNC").is_ok();
src/cpu.rs:1201:        let mut tr2: i64 = std::env::var("TRACE2").map(|v| v.parse().unwrap()).unwrap_or(0);
src/cpu.rs:1203:        let mut ks = std::env::var("KEYSEQ").unwrap_or_default();
src/cpu.rs:1204:        if let Ok(rp) = std::env::var("REPLAY") {
src/cpu.rs:1205:            ks = std::fs::read_to_string(rp).unwrap().lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty())
src/cpu.rs:1206:                .map(|l| { let mut it = l.split_whitespace(); format!("{}:{}", it.next().unwrap(), it.next().unwrap()) }).collect::<Vec<_>>().join(",");
src/cpu.rs:1209:            .map(|x| { let (f, m) = x.split_once(':').unwrap(); (f.parse().unwrap(), u32::from_str_radix(m, 16).unwrap()) }).collect();
src/cpu.rs:1213:            if std::env::var("TRACE").is_ok() && n < 6000000 { n += 1; eprintln!("{:08x} {:08x} cpsr={:08x} r0={:x} r1={:x} r2={:x} sp={:x} lr={:x}", cpu.r[15].wrapping_sub(if cpu.cpsr & FT != 0 {4} else {8}), cpu.pipe[0], cpu.cpsr, cpu.r[0], cpu.r[1], cpu.r[2], cpu.r[13], cpu.r[14]); }
src/cpu.rs:1216:            if tr2 > 0 && p0 >= 0x0800_0000 { if !seen_rom { seen_rom = true; eprintln!("rom entry cycles {}", c0); } tr2 -= 1; eprintln!("T {:08x} {}", p0, bus.cycles - c0); }
src/cpu.rs:1221:        std::fs::write(std::env::var("OUT").unwrap(), out).unwrap();
src/cpu.rs:1222:        eprintln!("io={:04x?}", (0x08..0x30).step_by(2).map(|o| bus.ppu.read_io(o)).collect::<Vec<_>>());
src/cpu.rs:1223:        eprintln!("
… [truncated 6255 chars]
```

**→ Read**
```
{"file_path": "~/work/Gbe-test/gba-emu/src/cpu.rs", "limit": 460}
```

**← Read result**
```
1	//! ARM7TDMI core: ARM + THUMB, banked registers, exceptions.
2	//!
3	//! Model: `r[15]` always holds the pipeline-visible PC (instr + 8 in ARM, instr + 4 in
4	//! THUMB). Each instruction pays its own fetch; a pipeline flush pays the two refill
5	//! fetches (N + S) so branches cost 2S+1N in total like on hardware.
6	use crate::bus::Bus;
7	
8	/// Memory/timing interface the core needs (implemented for `Bus`).
9	pub trait CpuBus {
10	    fn read8(&mut self, a: u32, seq: bool) -> u32;
11	    fn read16(&mut self, a: u32, seq: bool) -> u32;
12	    fn read32(&mut self, a: u32, seq: bool) -> u32;
13	    fn write8(&mut self, a: u32, v: u32, seq: bool);
14	    fn write16(&mut self, a: u32, v: u32, seq: bool);
15	    fn write32(&mut self, a: u32, v: u32, seq: bool);
16	    fn idle(&mut self, c: u32);
17	    fn irq_line(&self) -> bool;
18	    fn halted(&self) -> bool;
19	    /// Tell the bus the address/opcode of the instruction being executed (BIOS protection, open bus).
20	    fn note_fetch(&mut self, _pc: u32, _op: u32) {}
21	    fn note_pc(&mut self, _pc: u32) {}
22	    /// Untimed peek of the word at `a` (ROM only); used to set the open-bus value.
23	    fn peek32(&mut self, _a: u32) -> Option<u32> { None }
24	    fn set_open(&mut self, _v: u32) {}
25	    fn fetch16(&mut self, a: u32, seq: bool) -> u32 { self.read16(a, seq) }
26	    fn fetch32(&mut self, a: u32, seq: bool) -> u32 { self.read32(a, seq) }
27	}
28	
29	/// Any non-ROM bus access makes the next ROM fetch non-sequential (idle(0) marks the break).
30	#[inline(always)]
31	fn brk(b: &mut Bus, a: u32) {
32	    let _ = (b, a);
33	}
34	impl CpuBus for Bus {
35	    #[inline] fn read8(&mut self, a: u32, s: bool) -> u32 { let v = Bus::read8(self, a, s); brk(self, a); v }
36	    #[inline] fn read16(&mut self, a: u32, s: bool) -> u32 { let v = Bus::read16(self, a, s); brk(self, a); v }
37	    #[inline] fn read32(&mut self, a: u32, s: bool) -> u32 { let v = Bus::read32(self, a, s); brk(self, a); v }
38	    #[inline] fn write8(&mut self, a: u32, v: u32, s: bool) { Bus::write8(self, a, v, s); brk(self, a) }
39	    #[inline] fn write16(&mut self, a: u32, v: u32, s: bool) { Bus::write16(self, a, v, s); brk(self, a) }
40	    #[inline] fn write32(&mut self, a: u32, v: u32, s: bool) { Bus::write32(self, a, v, s); brk(self, a) }
41	    #[inline] fn idle(&mut self, c: u32) { Bus::idle(self, c) }
42	    #[inline] fn irq_line(&self) -> bool { Bus::irq_line(self) }
43	    #[inline] fn halted(&self) -> bool { Bus::halted(
… [truncated 15185 chars]
```

**→ Bash**
```
{"command": "date", "description": "Record time of cpu.rs read"}
```

**← Bash result**
```
Mon Nov  9 15:45:10 EET 2026
```

**→ Read**
```
{"file_path": "~/work/Gbe-test/gba-emu/src/cpu.rs", "offset": 460, "limit": 420}
```

**← Read result**
```
460	    #[inline]
461	    fn ld16s<B: CpuBus>(&mut self, b: &mut B, a: u32, seq: bool) -> u32 {
462	        if a & 1 != 0 {
463	            b.read8(a, seq) as u8 as i8 as u32
464	        } else {
465	            b.read16(a, seq) as u16 as i16 as u32
466	        }
467	    }
468	    #[inline]
469	    fn ld8s<B: CpuBus>(&mut self, b: &mut B, a: u32, seq: bool) -> u32 {
470	        b.read8(a, seq) as u8 as i8 as u32
471	    }
472	
473	    // ---------------------------------------------------------------- ARM
474	    fn exec_arm<B: CpuBus>(&mut self, b: &mut B, op: u32) {
475	        match (op >> 25) & 7 {
476	            0 => {
477	                if op & 0x90 == 0x90 {
478	                    if op & 0x60 == 0 {
479	                        // multiply / swap
480	                        if op & 0x0100_0000 == 0 {
481	                            if op & 0x0080_0000 == 0 { self.arm_mul(b, op) } else { self.arm_mull(b, op) }
482	                        } else {
483	                            self.arm_swp(b, op)
484	                        }
485	                    } else {
486	                        self.arm_half(b, op)
487	                    }
488	                } else if op & 0x0FFF_FFF0 == 0x012F_FF10 {
489	                    let t = self.r[(op & 15) as usize];
490	                    if t & 1 != 0 { self.cpsr |= FT } else { self.cpsr &= !FT }
491	                    self.set_pc(b, t);
492	                } else if op & 0x0190_0000 == 0x0100_0000 {
493	                    self.arm_psr(b, op)
494	                } else {
495	                    self.arm_dp(b, op)
496	                }
497	            }
498	            1 => {
499	                if op & 0x0190_0000 == 0x0100_0000 {
500	                    self.arm_psr(b, op)
501	                } else {
502	                    self.arm_dp(b, op)
503	                }
504	            }
505	            2 => self.arm_sdt(b, op),
506	            3 => {
507	                if op & 0x10 != 0 { self.arm_undef(b) } else { self.arm_sdt(b, op) }
508	            }
509	            4 => self.arm_block(b, op),
510	            5 => {
511	                let off = (((op << 8) as i32) >> 6) as u32;
512	                if op & (1 << 24) != 0 {
513	                    self.r[14] = self.r[15].wrapping_sub(4);
514	                }
515	                let t = self.r[15].wrapping_add(off);
516	                self.set_pc(b, t);
517	            }
518	            6 => self.arm_undef(b),
519	            _ => {
520	                
… [truncated 14886 chars]
```

**→ Read**
```
{"file_path": "~/work/Gbe-test/gba-emu/src/cpu.rs", "offset": 880, "limit": 346}
```

**← Read result**
```
880	                    5 => { self.r[rd] = self.ld16(b, a, false); b.idle(1); }
881	                    6 => { self.r[rd] = b.read8(a, false) & 0xFF; b.idle(1); }
882	                    _ => { self.r[rd] = self.ld16s(b, a, false); b.idle(1); }
883	                }
884	            }
885	            12..=15 => {
886	                let rb = self.r[((op >> 3) & 7) as usize];
887	                let rd = (op & 7) as usize;
888	                let imm = (op >> 6) & 31;
889	                match (op >> 11) & 3 {
890	                    0 => b.write32(rb.wrapping_add(imm << 2), self.r[rd], false),
891	                    1 => { let a = rb.wrapping_add(imm << 2); self.r[rd] = self.ld32(b, a, false); b.idle(1); }
892	                    2 => b.write8(rb.wrapping_add(imm), self.r[rd] & 0xFF, false),
893	                    _ => { self.r[rd] = b.read8(rb.wrapping_add(imm), false) & 0xFF; b.idle(1); }
894	                }
895	            }
896	            16 | 17 => {
897	                let rb = self.r[((op >> 3) & 7) as usize];
898	                let rd = (op & 7) as usize;
899	                let a = rb.wrapping_add(((op >> 6) & 31) << 1);
900	                if op & 0x0800 == 0 {
901	                    b.write16(a, self.r[rd] & 0xFFFF, false);
902	                } else {
903	                    self.r[rd] = self.ld16(b, a, false);
904	                    b.idle(1);
905	                }
906	            }
907	            18 | 19 => {
908	                let rd = ((op >> 8) & 7) as usize;
909	                let a = self.r[13].wrapping_add((op & 0xFF) << 2);
910	                if op & 0x0800 == 0 {
911	                    b.write32(a, self.r[rd], false);
912	                } else {
913	                    self.r[rd] = self.ld32(b, a, false);
914	                    b.idle(1);
915	                }
916	            }
917	            20 | 21 => {
918	                let rd = ((op >> 8) & 7) as usize;
919	                let base = if op & 0x0800 == 0 { self.r[15] & !3 } else { self.r[13] };
920	                self.r[rd] = base.wrapping_add((op & 0xFF) << 2);
921	            }
922	            22 | 23 => {
923	                if op & 0x0F00 == 0 {
924	                    let off = (op & 0x7F) << 2;
925	                    self.r[13] = if op & 0x80 != 0 { self.r[13].wrapping_sub(off) } else { self.r[13].wrapping_add(off) };
926	                } else if op & 0x0600 == 0x0400 {
927	                    self.thumb_pushpop(b, op);
928	                } else {
929	  
… [truncated 14085 chars]
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-emu; date; grep -n 'irq_prev\\|AUDIO_RATE\\|0.9962\\|resol\\|>> 14\\|fn drain\\|fn tick\\|fn timer_overflow\\|fn fifo_write\\|Vec\\|len()\\|\\[.*as usize\\]' src/apu.rs | head -60", "description": "Grep apu.rs for rate, buffers and indexing"}
```

**← Bash result**
```
Mon Nov  9 15:45:48 EET 2026
6:const HP: f32 = 0.9962;
52:    out: Vec<i16>,
59:            fs_step: 0, fs_left: FS_PERIOD, win_left: WIN, hx: [512.0; 2], hy: [0.0; 2], cur: [512; 2], dirty: true, pend: 0, next_ev: 0, out: Vec::new(),
228:            for i in 0..4 { if cl >> (en_sh + i) & 1 != 0 { s += outs[i as usize]; } }
229:            let scale = [1, 2, 4, 4][(ch & 3) as usize];
235:                    v += self.fifo[f as usize].cur as i32 * if full { 2 } else { 1 };
238:            self.cur[side as usize] = (v + bias).clamp(0, 0x3FF);
245:    pub fn tick(&mut self, cycles: u32) {
280:                // bias resolution bits: 9..6-bit, output rate 32768<<res (Mesen emits at that rate)
297:            return self.wave[1 - ((self.r[0x10] >> 6) & 1) as usize][(off - 0x90) as usize];
314:            self.wave[bank][(off - 0x90) as usize] = v;
318:            let f = &mut self.fifo[((off - 0xA0) >> 2) as usize];
366:    pub fn fifo_write(&mut self, ch: usize, word: u32) {
373:    pub fn timer_overflow(&mut self, timer: usize, count: u32) -> u8 {
388:    pub fn drain(&mut self, out: &mut Vec<i16>) {
409:        let mut o = Vec::new();
411:        let l: Vec<i16> = o.iter().step_by(2).copied().collect();
414:        assert!((12..=14).contains(&edges), "edges {edges} len {} first {:?}", l.len(), &l[..300].iter().step_by(10).collect::<Vec<_>>()); // ~12.8 cycles in 0.1 s
435:        let mut o = Vec::new();
452:        let mut o = Vec::new();
```

**→ Read**
```
{"file_path": "~/work/Gbe-test/gba-emu/src/apu.rs", "offset": 240, "limit": 165}
```

**← Read result**
```
240	    }
241	
242	    /// Lazy: nothing observable changes before the next channel/frame-sequencer/sample-window event,
243	    /// so cycles are banked and applied in one go at the event or before any state access.
244	    #[inline]
245	    pub fn tick(&mut self, cycles: u32) {
246	        self.pend += cycles;
247	        if self.pend >= self.next_ev { self.flush(); }
248	    }
249	
250	    fn flush(&mut self) {
251	        let n = std::mem::take(&mut self.pend);
252	        if n > 0 { self.run(n); }
253	        let mut e = self.win_left.min(self.fs_left);
254	        if self.master() { for c in &self.ch { if c.on { e = e.min(c.timer.max(1)); } } }
255	        self.next_ev = e.max(1);
256	    }
257	
258	    fn run(&mut self, cycles: u32) {
259	        let mut rem = cycles;
260	        let master = self.master();
261	        while rem > 0 {
262	            if self.dirty { self.recompute(); }
263	            let mut dt = rem.min(self.win_left).min(self.fs_left);
264	            if master {
265	                for c in &self.ch { if c.on && c.timer < dt { dt = c.timer.max(1); } }
266	            }
267	            rem -= dt;
268	            self.win_left -= dt;
269	            self.fs_left -= dt;
270	            if master {
271	                for i in 0..4 {
272	                    if self.ch[i].on {
273	                        self.ch[i].timer -= dt.min(self.ch[i].timer);
274	                        if self.ch[i].timer == 0 { self.step_chan(i); }
275	                    }
276	                }
277	            }
278	            if self.fs_left == 0 { self.fs_left = FS_PERIOD; if master { self.frame_step(); } }
279	            if self.win_left == 0 {
280	                // bias resolution bits: 9..6-bit, output rate 32768<<res (Mesen emits at that rate)
281	                let res = (self.r[0x29] >> 6) as u32;
282	                self.win_left = WIN >> res;
283	                for s in 0..2 {
284	                    let x = ((self.cur[s] >> res) << res) as f32;
285	                    // DC blocker (matches Mesen's slow droop on flat tops)
286	                    let y = x - self.hx[s] + HP * self.hy[s];
287	                    self.hx[s] = x;
288	                    self.hy[s] = y;
289	                    self.out.push((y * 64.0).clamp(-32768.0, 32767.0) as i16);
290	                }
291	            }
292	        }
293	    }
294	
295	    pub fn read_io(&self, off: u32) -> u8 {
296	        if (0x90..0xA0).contains(&off) {
297	            return self.wave[1 
… [truncated 4182 chars]
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-eval; ls; ls grader* 2>/dev/null; grep -rn \"emu_audio_rate\\|emu_audio_samples\\|emu_reset\\|emu_set_keys\\|fuel\\|warmup\\|WARMUP\" --include=*.rs . | grep -v candidates | grep -v target | head -40", "description": "Find grader ABI usage"}
```

**← Bash result**
```
CONTACT.md
Cargo.lock
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
(eval):1: no matches found: grader*
(eval):1: no matches found: --include=*.rs
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-eval; grep -rln \"emu_audio_rate\" harness quickstart spec 2>/dev/null | head; ", "description": "Locate grader source files using audio rate"}
```

**← Bash result**
```
harness/grader/src/wasm_candidate.rs
spec/ABI.md
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-eval/harness/grader/src; ls; grep -n \"audio_rate\\|audio_samples\\|emu_reset\\|set_keys\\|fuel\\|warm\\|resampl\\|rate\" *.rs | head -60", "description": "Inspect grader usage of audio and reset"}
```

**← Bash result**
```
corpus.rs
lib.rs
main.rs
passfail.rs
ref_cache.rs
wasm_candidate.rs
lib.rs:1://! Library surface for the grader crate.
lib.rs:3://! Exposes the wasmtime + fuel + import-stub wiring used by the grader
corpus.rs:119:            return Some(crate::ref_cache::sha256_hex(&[]));
corpus.rs:122:        crate::ref_cache::sha256_file(&path)
main.rs:33:    /// Escape hatch for candidates that need warmup beyond what their
main.rs:36:    fuel_per_frame: u64,
main.rs:37:    fuel_load_rom: u64,
main.rs:56:            fuel_per_frame: 500_000_000,
main.rs:57:            fuel_load_rom: 60_000_000_000,
main.rs:153:        config.fuel_per_frame,
main.rs:154:        config.fuel_load_rom,
main.rs:398:            let _ = writeln!(log, "  note: {} cache absent/stale; warming", tc.id);
main.rs:401:                config.fuel_per_frame, config.fuel_load_rom,
main.rs:410:            let _ = writeln!(log, "  note: {} warm failed: {e:#}", tc.id);
main.rs:423:        config.fuel_per_frame,
main.rs:424:        config.fuel_load_rom,
main.rs:433:    // Reference warmup is not burned here — lockstep() does it from
main.rs:491:        if let Err(e) = write_wav_capped(&ref_wav, &output.ref_audio, output.audio_rate, WAV_SIZE_CAP) {
main.rs:494:        if let Err(e) = write_wav_capped(&cand_wav, &output.cand_audio, output.audio_rate, WAV_SIZE_CAP) {
main.rs:590:                    config.fuel_per_frame, config.fuel_load_rom,
wasm_candidate.rs:3://! infinite-loops traps cleanly via the per-call fuel budget.
wasm_candidate.rs:46:    f_set_keys: TypedFunc<i32, ()>,
wasm_candidate.rs:50:    f_audio_samples: TypedFunc<(), i32>,
wasm_candidate.rs:51:    cached_audio_rate: u32,
wasm_candidate.rs:59:    fuel_per_frame: u64,
wasm_candidate.rs:60:    fuel_load_rom: u64,
wasm_candidate.rs:64:    pub fn new(wasm_bytes: &[u8], label: String, fuel_per_frame: u64, fuel_load_rom: u64) -> Result<Self> {
wasm_candidate.rs:65:        // 0 is almost certainly a yaml typo (`fuel_per_frame:` with no
wasm_candidate.rs:68:        if fuel_per_frame == 0 {
wasm_candidate.rs:69:            bail!("grader config: fuel_per_frame must be > 0 (got 0 — typo in grader.yaml?)");
wasm_candidate.rs:71:        if fuel_load_rom == 0 {
wasm_candidate.rs:72:            bail!("grader config: fuel_load_rom must be > 0 (got 0 — typo in grader.yaml?)");
wasm_candidate.rs:77:        config.consume_fuel(true);
wasm_candidate.rs:84:        store.set_fuel(fuel_per_frame).context("initial fuel")?;
wasm_candidate.rs:89:        // list, genera
… [truncated 2099 chars]
```

**→ Read**
```
{"file_path": "~/work/Gbe-test/gba-eval/harness/grader/src/wasm_candidate.rs", "offset": 225, "limit": 200}
```

**← Read result**
```
225	        }
226	
227	        // Audio exports — resolve handles now, call per-frame in drain_audio.
228	        let f_audio_buffer = get_typed::<(), i32>(&instance, &mut store, "emu_audio_buffer")?;
229	        let f_audio_samples = get_typed::<(), i32>(&instance, &mut store, "emu_audio_samples")?;
230	        let f_audio_rate = get_typed::<(), i32>(&instance, &mut store, "emu_audio_rate")?;
231	
232	        // Optional boot-frames export. Present on candidates that wrap a
233	        // pre-existing emulator with its own warmup cost. Missing → default
234	        // 0 per the ABI (a conformant `load_rom()` leaves you at frame 0).
235	        let f_boot_frames = instance.get_typed_func::<(), i32>(&mut store, "emu_boot_frames").ok();
236	
237	        // ─── Call init, cache stable pointers ────────────────────────────
238	        // emu_init internally calls emu_load_rom(256) as a warmup, so it
239	        // needs the full load_rom fuel budget, not just per-frame.
240	        store.set_fuel(fuel_load_rom)?;
241	        let ok = f_init.call(&mut store, ())
242	            .context("emu_init trapped")?;
243	        if ok == 0 {
244	            bail!("emu_init returned 0 (failure)");
245	        }
246	
247	        let rom_buf_ptr = f_rom_buffer.call(&mut store, ())? as u32;
248	        let fb_ptr = f_framebuffer.call(&mut store, ())? as u32;
249	        let cached_audio_rate = f_audio_rate.call(&mut store, ()).unwrap_or(32768) as u32;
250	        let cached_boot_frames = match &f_boot_frames {
251	            Some(f) => f.call(&mut store, ()).unwrap_or(0).max(0) as u32,
252	            None => 0,
253	        };
254	
255	        // Sanity: framebuffer must fit in current memory. (rom_buffer we
256	        // check at load_rom time, since memory might grow between now
257	        // and then.)
258	        let mem_size = memory.data_size(&store);
259	        let fb_end = fb_ptr as usize + GBA_PIXELS * 4;
260	        if fb_end > mem_size {
261	            bail!(
262	                "emu_framebuffer() = {fb_ptr:#x} but memory is only {mem_size} bytes \
263	                 (framebuffer would end at {fb_end:#x})"
264	            );
265	        }
266	
267	        Ok(Self {
268	            label,
269	            store,
270	            memory,
271	            rom_buf_ptr,
272	            fb_ptr,
273	            f_load_rom,
274	            f_set_keys,
275	            f_run_frame,
276	            f_framebuffer,
277	            f_audio_buffer,
278	            f_audio_s
… [truncated 5977 chars]
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-eval/harness; grep -rn \"boot_frames\\|audio_rate\" --include=*.rs . | grep -v target | grep -v \"wasm_candidate.rs\" | head -30", "description": "Find boot_frames and audio_rate usage in harness"}
```

**← Bash result**
```
(eval):1: no matches found: --include=*.rs
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-eval/harness; grep -rn -e \"boot_frames\" -e \"audio_rate\" -e \"resample\" . 2>/dev/null | grep -v \"/target/\" | grep -v \"wasm_candidate.rs\" | head -30", "description": "Find boot_frames/audio_rate usage in harness"}
```

**← Bash result**
```
lockstep/src/mesen.rs:35:    fn mesen_audio_rate() -> i32;
lockstep/src/mesen.rs:105:    fn audio_rate(&self) -> u32 {
lockstep/src/mesen.rs:106:        let r = unsafe { mesen_audio_rate() };
lockstep/src/mesen.rs:110:    fn boot_frames(&self) -> u32 {
lockstep/src/audio.rs:8://!    1024). STFT hop = `audio_rate / 60`, one analysis frame per
lockstep/src/audio.rs:100:    audio_rate: u32,
lockstep/src/audio.rs:109:    pub fn new(audio_rate: u32) -> Self {
lockstep/src/audio.rs:113:        let filters = mel_filterbank(audio_rate);
lockstep/src/audio.rs:114:        let hop = (audio_rate as usize / 60).max(1);
lockstep/src/audio.rs:115:        Self { audio_rate, fft, window, filters, hop }
lockstep/src/audio.rs:118:    pub fn audio_rate(&self) -> u32 {
lockstep/src/audio.rs:119:        self.audio_rate
lockstep/src/audio.rs:144:fn mel_filterbank(audio_rate: u32) -> Vec<Vec<f64>> {
lockstep/src/audio.rs:146:    let fmax = (audio_rate as f64 / 2.0).min(LOGMEL_F_MAX_CAP);
lockstep/src/audio.rs:153:        boundaries.push(hz * LOGMEL_FFT_SIZE as f64 / audio_rate as f64);
lockstep/src/audio.rs:237:/// `audio_rate/60`, window = `LOGMEL_FFT_SIZE`). Also returns a parallel
lockstep/src/audio.rs:372:    audio_rate: u32,
lockstep/src/audio.rs:375:    score_logmel_detailed(ref_buf, cand_buf, audio_rate, tau).map(|s| s.mean)
lockstep/src/audio.rs:384:    audio_rate: u32,
lockstep/src/audio.rs:387:    let ctx = LogMelContext::new(audio_rate);
lockstep/src/audio.rs:438:pub fn derive_threshold(ref_buf: &[i16], audio_rate: u32) -> f64 {
lockstep/src/audio.rs:439:    let ctx = LogMelContext::new(audio_rate);
grader/src/ref_cache.rs:58:    audio_rate: u32,
grader/src/ref_cache.rs:61:    boot_frames: u32,
grader/src/ref_cache.rs:163:    for _ in 0..reference.boot_frames() {
grader/src/ref_cache.rs:191:    let audio_rate = reference.audio_rate();
grader/src/ref_cache.rs:192:    let audio_diff_threshold = derive_audio_threshold(&audio_flat, audio_rate) as f32;
grader/src/ref_cache.rs:200:        audio_rate,
grader/src/ref_cache.rs:201:        boot_frames: reference.boot_frames(),
grader/src/ref_cache.rs:300:            // lockstep() burns `boot_frames()` before the grading loop;
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-eval/harness; sed -n 95,120p lockstep/src/mesen.rs; sed -n 150,200p grader/src/ref_cache.rs; grep -n \"audio_rate\\|boot_frames\\|cand_audio\\|ref_audio\" grader/src/main.rs lockstep/src/lib.rs 2>/dev/null | head -30", "description": "Check how reference boot frames and audio rate are used"}
```

**← Bash result**
```
let pairs = unsafe { mesen_audio_samples() };
        if pairs <= 0 {
            return Vec::new();
        }
        let ptr = unsafe { mesen_audio_buffer() };
        let count = pairs as usize * 2; // stereo: L,R per pair
        // SAFETY: static buffer, valid until next run_frame. We copy out.
        unsafe { std::slice::from_raw_parts(ptr, count) }.to_vec()
    }

    fn audio_rate(&self) -> u32 {
        let r = unsafe { mesen_audio_rate() };
        if r > 0 { r as u32 } else { 32768 }
    }

    fn boot_frames(&self) -> u32 {
        // Mesen's SkipBootScreen leaves ~5 frames of internal post-reset
        // warmup before the first game pixel; lockstep aligns by
        // burning these.
        6
    }
}
    reference: &mut dyn Reference,
    replay: &lockstep::InputReplay,
    frame_count: u32,
    ref_wasm_sha256: String,
    rom_sha256: String,
    replay_sha256: String,
) -> Result<()> {
    let mut frames_flat = Vec::with_capacity(frame_count as usize * GBA_BYTES);
    let mut audio_samples_per_frame = Vec::with_capacity(frame_count as usize);
    let mut audio_flat = Vec::new();
    let mut ref_defects: Vec<f32> = Vec::with_capacity(frame_count as usize);
    let mut prev_fb: [u32; GBA_PIXELS] = [0; GBA_PIXELS];

    for _ in 0..reference.boot_frames() {
        reference.run_frame();
        let _ = reference.drain_audio(); // discard boot-period audio
    }

    for frame in 0..frame_count {
        reference.set_keys(replay.keys_at(frame));
        reference.run_frame();

        let fb = reference.framebuffer();
        // SAFETY: GBA_PIXELS u32s = GBA_BYTES bytes; same layout.
        let bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(fb.as_ptr() as *const u8, GBA_BYTES)
        };
        frames_flat.extend_from_slice(bytes);

        if frame > 0 && video::ref_in_motion(&prev_fb, fb) {
            ref_defects.push(video::ssim_floored(&prev_fb, fb));
        }
        prev_fb.copy_from_slice(fb);

        let samples = reference.drain_audio();
        let pairs = (samples.len() / 2) as u32;
        audio_samples_per_frame.push(pairs);
        audio_flat.extend(samples);
    }

    let frame_diff_threshold = defect_threshold_clamped(&mut ref_defects);
    let audio_rate = reference.audio_rate();
    let audio_diff_threshold = derive_audio_threshold(&audio_flat, audio_rate) as f32;

    let cache = ReferenceCache {
        version: CACHE_VERSION,
        ref_wasm_sha256,
        rom_sha256,
        replay_sha256,
    
… [truncated 2011 chars]
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-emu; date; grep -n '\\[' src/ppu.rs | grep -v '^\\s*//' | grep -E 'vram\\[|pal\\[|oam\\[|line\\[|fb\\[|buf\\[|\\] =|as usize\\]' | head -70", "description": "Grep ppu.rs indexing sites"}
```

**← Bash result**
```
Mon Nov  9 15:46:33 EET 2026
14:const SIZES: [[(i32, i32); 4]; 3] = [
130:        self.r[i] = nv;
132:            0x14 | 0x15 => self.refx[0] = sext28(self.r[0x14] as u32 | (self.r[0x15] as u32) << 16),
133:            0x16 | 0x17 => self.refy[0] = sext28(self.r[0x16] as u32 | (self.r[0x17] as u32) << 16),
134:            0x1C | 0x1D => self.refx[1] = sext28(self.r[0x1C] as u32 | (self.r[0x1D] as u32) << 16),
135:            0x1E | 0x1F => self.refy[1] = sext28(self.r[0x1E] as u32 | (self.r[0x1F] as u32) << 16),
159:                    self.refx[0] = sext28(self.r[0x14] as u32 | (self.r[0x15] as u32) << 16);
160:                    self.refy[0] = sext28(self.r[0x16] as u32 | (self.r[0x17] as u32) << 16);
161:                    self.refx[1] = sext28(self.r[0x1C] as u32 | (self.r[0x1D] as u32) << 16);
162:                    self.refy[1] = sext28(self.r[0x1E] as u32 | (self.r[0x1F] as u32) << 16);
177:        self.vram[if a >= 0x18000 { a - 0x8000 } else { a }]
185:        (self.pal[i] as u16 | (self.pal[i + 1] as u16) << 8) & 0x7FFF
197:                s.refx[i] = s.refx[i].wrapping_add(s.r[0x11 + 8 * i] as i16 as i32);
198:                s.refy[i] = s.refy[i].wrapping_add(s.r[0x13 + 8 * i] as i16 as i32);
202:            for p in &mut self.fb[y * 240..y * 240 + 240] { *p = 0xFFFF_FFFF; }
230:                    if y % bgv == 0 { self.snap[k] = (self.refx[k], self.refy[k]); }
238:                for x in 0..240 { bg[i][x] = bg[i][x - x % bgh]; }
254:                if mask & (1 << i) != 0 && (self.r[4 + i as usize] & 3) as u8 == p {
255:                    list[nl] = (p, i);
300:                    c[n] = (ocol[x], 4);
306:                    let px = bg[idx as usize][x];
308:                        c[n] = (px, idx);
314:            if n < 2 && !odone { c[n] = (ocol[x], 4); n += 1; }
315:            if n < 2 { c[n] = (backdrop, 5); n += 1; }
316:            if n < 2 { c[1] = (backdrop, 5); n = 1; }
331:            self.fb[y * 240 + x] = rgb(out);
367:                    out[x] = self.pc(2 * if bpp8 { idx } else { pb + idx });
393:                        3 => out[x] = self.v16((py * 240 + px) * 2) & 0x7FFF,
396:                            if idx != 0 { out[x] = self.pc(idx * 2); }
398:                        _ => out[x] = self.v16(frame + (py * 160 + px) * 2) & 0x7FFF,
411:                    if idx != 0 { out[x] = self.pc(idx * 2); }
432:            let (w, h) = SIZES[shape][(a1 >> 14) as usize];
491:                    owin[xu] = true;
492:              
… [truncated 171 chars]
```

**→ Read**
```
{"file_path": "~/work/Gbe-test/gba-emu/src/ppu.rs", "offset": 170, "limit": 329}
```

**← Read result**
```
170	        }
171	        ev
172	    }
173	
174	    #[inline]
175	    fn v8(&self, a: usize) -> u8 {
176	        let a = a & 0x1FFFF;
177	        self.vram[if a >= 0x18000 { a - 0x8000 } else { a }]
178	    }
179	    #[inline]
180	    fn v16(&self, a: usize) -> u16 {
181	        self.v8(a) as u16 | (self.v8(a + 1) as u16) << 8
182	    }
183	    #[inline]
184	    fn pc(&self, i: usize) -> u16 {
185	        (self.pal[i] as u16 | (self.pal[i + 1] as u16) << 8) & 0x7FFF
186	    }
187	
188	    fn render_line(&mut self) {
189	        let y = self.line as usize;
190	        let dc = self.r[0];
191	        let mode = (dc & 7) as u32;
192	        let mosaic = self.r[0x26];
193	        let (bgh, bgv) = ((mosaic & 15) as usize + 1, ((mosaic >> 4) & 15) as usize + 1);
194	
195	        let advance = |s: &mut Ppu| {
196	            for i in 0..2 {
197	                s.refx[i] = s.refx[i].wrapping_add(s.r[0x11 + 8 * i] as i16 as i32);
198	                s.refy[i] = s.refy[i].wrapping_add(s.r[0x13 + 8 * i] as i16 as i32);
199	            }
200	        };
201	        if dc & 0x80 != 0 {
202	            for p in &mut self.fb[y * 240..y * 240 + 240] { *p = 0xFFFF_FFFF; }
203	            advance(self);
204	            return;
205	        }
206	
207	        // which BG layers produce pixels
208	        let mut mask = 0u8;
209	        match mode {
210	            0 => mask = 0xF,
211	            1 => mask = 0x7,
212	            2 => mask = 0xC,
213	            3..=5 => mask = 0x4,
214	            _ => {}
215	        }
216	        mask &= ((dc >> 8) & 0xF) as u8;
217	
218	        let mut bg = [[TRANSP; 240]; 4];
219	        for i in 0..4usize {
220	            if mask & (1 << i) == 0 { continue; }
221	            let cnt = self.r[4 + i];
222	            let mos = cnt & 0x40 != 0;
223	            let affine = (mode == 1 && i == 2) || mode == 2 || mode >= 3;
224	            if !affine {
225	                let ye = if mos { y - y % bgv } else { y };
226	                self.render_text(i, ye, &mut bg[i]);
227	            } else {
228	                let k = i - 2;
229	                let (cx, cy) = if mos && bgv > 1 {
230	                    if y % bgv == 0 { self.snap[k] = (self.refx[k], self.refy[k]); }
231	                    self.snap[k]
232	                } else {
233	                    (self.refx[k], self.refy[k])
234	                };
235	                self.render_affine(k, mode, cx, cy, &mut bg[i]);
236	            }
237	            if mos && bgh > 1 {
238	         
… [truncated 12708 chars]
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-eval; grep -rn \"fuel\" corpus/grader.yaml harness/grader/src/main.rs | head; grep -n \"MAX_ROM\" harness/grader/src/*.rs | head -3", "description": "Check fuel config and max ROM"}
```

**← Bash result**
```
harness/grader/src/main.rs:36:    fuel_per_frame: u64,
harness/grader/src/main.rs:37:    fuel_load_rom: u64,
harness/grader/src/main.rs:56:            fuel_per_frame: 500_000_000,
harness/grader/src/main.rs:57:            fuel_load_rom: 60_000_000_000,
harness/grader/src/main.rs:153:        config.fuel_per_frame,
harness/grader/src/main.rs:154:        config.fuel_load_rom,
harness/grader/src/main.rs:401:                config.fuel_per_frame, config.fuel_load_rom,
harness/grader/src/main.rs:423:        config.fuel_per_frame,
harness/grader/src/main.rs:424:        config.fuel_load_rom,
harness/grader/src/main.rs:590:                    config.fuel_per_frame, config.fuel_load_rom,
harness/grader/src/wasm_candidate.rs:29:const MAX_ROM: usize = 32 * 1024 * 1024;
harness/grader/src/wasm_candidate.rs:288:        if rom.len() > MAX_ROM {
harness/grader/src/wasm_candidate.rs:289:            bail!("ROM too large: {} > {MAX_ROM}", rom.len());
```

**→ Bash**
```
{"command": "export PATH=/opt/homebrew/opt/rustup/bin:$PATH; S=/private/tmp/claude-501/-Users-USER-work-Gbe-test/2cbb11f1-a7a2-402f-bc9b-6a4faabf55e9/scratchpad; mkdir -p $S; cd ~/work/Gbe-test/gba-emu; date; CARGO_TARGET_DIR=$S/tgt cargo build --release --lib --target wasm32-unknown-unknown 2>&1 | tail -3; CARGO_TARGET_DIR=$S/tgt cargo test --release 2>&1 | grep -E \"test result|FAILED|panicked|warning: unused\" | head -20; ls -la $S/tgt/wasm32-unknown-unknown/release/gba_emu.wasm", "description": "Build wasm and run tests in scratch target dir", "timeout": 600000}
```

**← Bash result**
```
Mon Nov  9 15:47:00 EET 2026

warning: `gba-emu` (lib) generated 6 warnings (run `cargo fix --lib -p gba-emu` to apply 2 suggestions)
    Finished `release` profile [optimized] target(s) in 1.42s
warning: unused variable: `a`
warning: unused variable: `seq`
test result: ok. 12 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
-rwxr-xr-x@ 1 USER  wheel  106793 Nov  9 15:47 /private/tmp/claude-501/-Users-USER-work-Gbe-test/2cbb11f1-a7a2-402f-bc9b-6a4faabf55e9/scratchpad/tgt/wasm32-unknown-unknown/release/gba_emu.wasm
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-eval/corpus; ls roms/*/ | head -60; ls replays 2>/dev/null | head", "description": "List corpus ROMs"}
```

**← Bash result**
```
roms/homebrew/:
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

roms/test/:
armwrestler.gba
destoer
fuzzarm.gba
jsmolka
mgba-suite.gba
misc
nba-hw
tonc
aw-arm-alu.txt
aw-arm-ldm-stm.txt
aw-arm-ldr-str.txt
aw-thumb-alu.txt
aw-thumb-ldm-stm.txt
aw-thumb-ldr-str.txt
bulletgba-boot-spam-a.txt
bulletgba-gameplay.txt
celeste-boot-spam-a.txt
celeste-demo.txt
```

**→ Write**
```
{"file_path": "/private/tmp/claude-501/-Users-USER-work-Gbe-test/2cbb11f1-a7a2-402f-bc9b-6a4faabf55e9/scratchpad/abi.mjs", "content": "// ABI probe: node abi.mjs <wasm> <rom> [frames]\nimport fs from 'fs';\nimport crypto from 'crypto';\nconst [wasmPath, romPath, nf = '120'] = process.argv.slice(2);\nconst N = +nf;\nconst mod = await WebAssembly.compile(fs.readFileSync(wasmPath));\nconsole.log('imports', WebAssembly.Module.imports(mod).map(i => i.module + '.' + i.name));\nconsole.log('exports', WebAssembly.Module.exports(mod).map(e => e.name).join(' '));\nfunction inst() {\n  const imp = {};\n  for (const i of WebAssembly.Module.imports(mod)) { (imp[i.module] ??= {})[i.name] = () => { throw new Error('import called ' + i.name); }; }\n  return new WebAssembly.Instance(mod, imp).exports;\n}\nfunction load(e, rom) {\n  e.emu_init();\n  const p = e.emu_rom_buffer();\n  new Uint8Array(e.memory.buffer, p, rom.length).set(rom);\n  return e.emu_load_rom(rom.length);\n}\nfunction run(e, n, keysAt = () => 0) {\n  const h = crypto.createHash('sha1'); const ah = crypto.createHash('sha1'); let pairs = [];\n  for (let f = 0; f < n; f++) {\n    e.emu_set_keys(keysAt(f)); e.emu_run_frame();\n    h.update(new Uint8Array(e.memory.buffer, e.emu_framebuffer(), 240 * 160 * 4));\n    const k = e.emu_audio_samples(); pairs.push(k);\n    ah.update(new Uint8Array(e.memory.buffer, e.emu_audio_buffer(), k * 4));\n  }\n  return { v: h.digest('hex').slice(0, 12), a: ah.digest('hex').slice(0, 12), pairs
… [truncated 1227 chars]
```

**← Write result**
```
File created successfully at: /private/tmp/claude-501/-Users-USER-work-Gbe-test/2cbb11f1-a7a2-402f-bc9b-6a4faabf55e9/scratchpad/abi.mjs (file state is current in your context — no need to Read it back)
```

**→ Bash**
```
{"command": "S=/private/tmp/claude-501/-Users-USER-work-Gbe-test/2cbb11f1-a7a2-402f-bc9b-6a4faabf55e9/scratchpad; W=$S/tgt/wasm32-unknown-unknown/release/gba_emu.wasm; R=~/work/Gbe-test/gba-eval/corpus/roms/homebrew; cd $S; for r in celeste-classic goodboy-advance; do echo \"== $r\"; node abi.mjs $W $R/$r.gba 120 2>&1; done", "description": "Run ABI probe on two ROMs", "timeout": 600000}
```

**← Bash result**
```
== celeste-classic
imports []
exports memory emu_audio_buffer emu_audio_rate emu_audio_samples emu_framebuffer emu_init emu_load_rom emu_reset emu_rom_buffer emu_run_frame emu_set_keys
load 1 rate 32768
fresh   { v: '8284c9ac1def', a: '261c34502412', pairs: [ 548, 549 ] }
reset 1
reset   { v: '8284c9ac1def', a: '261c34502412', pairs: [ 548, 549 ] } IDENTICAL
reload 1
reload  { v: '8284c9ac1def', a: '261c34502412', pairs: [ 548, 549 ] } IDENTICAL
32MiB load 1 { v: '835d77a0726d', a: '76a65e55e6b0', pairs: [ 548, 549 ] }
odd load 1 { v: '835d77a0726d', a: '76a65e55e6b0', pairs: [ 548, 549 ] }
over load 0 { v: 'c710eb162d2d', a: 'da39a3ee5e6b', pairs: [ 0, 0 ] }
undrained 30 frames -> pairs 4096 then 0
== goodboy-advance
imports []
exports memory emu_audio_buffer emu_audio_rate emu_audio_samples emu_framebuffer emu_init emu_load_rom emu_reset emu_rom_buffer emu_run_frame emu_set_keys
load 1 rate 32768
fresh   { v: '7cfefc66666c', a: '6f89cbd9ba80', pairs: [ 548, 549 ] }
reset 1
reset   { v: '7cfefc66666c', a: '6f89cbd9ba80', pairs: [ 548, 549 ] } IDENTICAL
reload 1
reload  { v: '7cfefc66666c', a: '6f89cbd9ba80', pairs: [ 548, 549 ] } IDENTICAL
32MiB load 1 { v: '83063ae88f67', a: '70421de9a4cb', pairs: [ 548, 549 ] }
odd load 1 { v: '83063ae88f67', a: '70421de9a4cb', pairs: [ 548, 549 ] }
over load 0 { v: 'c710eb162d2d', a: 'da39a3ee5e6b', pairs: [ 0, 0 ] }
undrained 30 frames -> pairs 4096 then 0
```

**→ Bash**
```
{"command": "S=/private/tmp/claude-501/-Users-USER-work-Gbe-test/2cbb11f1-a7a2-402f-bc9b-6a4faabf55e9/scratchpad; cd $S; sed -i '' \"s|// 32 MiB padded, odd-size|{ const f = inst(); load(f, rom); console.log('orig30', run(f, 30, keys)); }|\" abi.mjs; W=$S/tgt/wasm32-unknown-unknown/release/gba_emu.wasm; R=~/work/Gbe-test/gba-eval/corpus/roms/homebrew; node abi.mjs $W $R/celeste-classic.gba 10 2>&1 | grep -E \"orig30|32MiB|odd\"; ls -la $R | head -20", "description": "Compare 30-frame hash for original vs padded ROMs", "timeout": 600000}
```

**← Bash result**
```
orig30 { v: '835d77a0726d', a: '76a65e55e6b0', pairs: [ 548, 549 ] }
32MiB load 1 { v: '835d77a0726d', a: '76a65e55e6b0', pairs: [ 548, 549 ] }
odd load 1 { v: '835d77a0726d', a: '76a65e55e6b0', pairs: [ 548, 549 ] }
total 139872
drwxr-xr-x@ 18 USER  staff       576 Nov  5 18:47 .
drwxr-xr-x@  4 USER  staff       128 Nov  5 18:47 ..
-rw-r--r--@  1 USER  staff   1710888 Nov  5 18:47 anguna.gba
-rw-r--r--@  1 USER  staff   2010358 Nov  5 18:47 another-world.gba
-rw-r--r--@  1 USER  staff  16565008 Nov  5 18:47 blindjump.gba
-rw-r--r--@  1 USER  staff   8388608 Nov  5 18:47 bulletgba.gba
-rw-r--r--@  1 USER  staff   5418424 Nov  5 18:47 celeste-classic.gba
-rw-r--r--@  1 USER  staff   4276412 Nov  5 18:47 chip-advance.gba
-rw-r--r--@  1 USER  staff   1207868 Nov  5 18:47 collie-defense.gba
-rw-r--r--@  1 USER  staff   2787328 Nov  5 18:47 goodboy-advance.gba
-rw-r--r--@  1 USER  staff   3990892 Nov  5 18:47 heartwrench-advance.gba
-rw-r--r--@  1 USER  staff   1710503 Nov  5 18:47 meteorain.gba
-rw-r--r--@  1 USER  staff   6614656 Nov  5 18:47 piugba.gba
-rw-r--r--@  1 USER  staff    752448 Nov  5 18:47 spout.gba
-rw-r--r--@  1 USER  staff   4854904 Nov  5 18:47 trogdor.gba
-rw-r--r--@  1 USER  staff   7975756 Nov  5 18:47 varooom-3d.gba
-rw-r--r--@  1 USER  staff   1739504 Nov  5 18:47 waimanu.gba
-rw-r--r--@  1 USER  staff   1584436 Nov  5 18:47 xniq.gba
[This command modified 1 file you've previously read: ../../../../private/tmp/claude-501/-Users-USER-work-Gbe-test/2cbb11f1-a7a2-402f-bc9b-6a4faabf55e9/scratchpad/abi.mjs. Call Read before editing.]
```

**→ Bash**
```
{"command": "S=/private/tmp/claude-501/-Users-USER-work-Gbe-test/2cbb11f1-a7a2-402f-bc9b-6a4faabf55e9/scratchpad; cd $S; W=$S/tgt/wasm32-unknown-unknown/release/gba_emu.wasm; R=~/work/Gbe-test/gba-eval/corpus/roms; for r in homebrew/spout homebrew/bulletgba homebrew/piugba homebrew/varooom-3d homebrew/blindjump homebrew/anguna test/mgba-suite; do echo \"== $r $(node abi.mjs $W $R/$r.gba 90 2>&1 | grep -E 'IDENTICAL|DIFFERENT|TRAP|undrained' | awk '{print $NF}' | tr '\\n' ' ')\"; done", "description": "Check reset/reload determinism across ROMs", "timeout": 600000}
```

**← Bash result**
```
== homebrew/spout IDENTICAL IDENTICAL 0 
== homebrew/bulletgba DIFFERENT DIFFERENT 0 
== homebrew/piugba IDENTICAL IDENTICAL 0 
== homebrew/varooom-3d IDENTICAL IDENTICAL 0 
== homebrew/blindjump IDENTICAL IDENTICAL 0 
== homebrew/anguna IDENTICAL IDENTICAL 0 
== test/mgba-suite IDENTICAL IDENTICAL 0
```

**→ Bash**
```
{"command": "S=/private/tmp/claude-501/-Users-USER-work-Gbe-test/2cbb11f1-a7a2-402f-bc9b-6a4faabf55e9/scratchpad; cd $S; W=$S/tgt/wasm32-unknown-unknown/release/gba_emu.wasm; R=~/work/Gbe-test/gba-eval/corpus/roms; node abi.mjs $W $R/homebrew/bulletgba.gba 90 2>&1 | sed -n 3,8p; grep -n \"fn reset\\|fn new\" -A12 ~/work/Gbe-test/gba-emu/src/apu.rs | head -40; grep -n \"fn reset\" -A10 ~/work/Gbe-test/gba-emu/src/ppu.rs", "description": "Inspect bulletgba reset diff and component reset fns", "timeout": 600000}
```

**← Bash result**
```
load 1 rate 32768
fresh   { v: 'e7aff128e64d', a: 'dbeb7367f704', pairs: [ 548, 550 ] }
reset 1
reset   { v: 'e7aff128e64d', a: 'afb1ff66611e', pairs: [ 548, 550 ] } DIFFERENT
reload 1
reload  { v: 'e7aff128e64d', a: 'afb1ff66611e', pairs: [ 548, 550 ] } DIFFERENT
56:    pub fn new() -> Self {
57-        let mut a = Apu {
58-            r: [0; 0x30], wave: [[0; 16]; 2], ch: [Chan::default(); 4], fifo: [Fifo::default(); 2],
59-            fs_step: 0, fs_left: FS_PERIOD, win_left: WIN, hx: [512.0; 2], hy: [0.0; 2], cur: [512; 2], dirty: true, pend: 0, next_ev: 0, out: Vec::new(),
60-        };
61-        a.reset();
62-        a
63-    }
64:    pub fn reset(&mut self) {
65-        self.r = [0; 0x30];
66-        self.r[0x28] = 0x00;
67-        self.r[0x29] = 0x02;
68-        self.wave = [[0; 16]; 2];
69-        self.ch = [Chan::default(); 4];
70-        self.fifo = [Fifo::default(); 2];
71-        self.fs_step = 0;
72-        self.fs_left = FS_PERIOD;
73-        self.win_left = WIN;
74-        self.hx = [512.0; 2];
75-        self.hy = [0.0; 2];
76-        self.dirty = true;
79:    pub fn reset(&mut self) {
80-        *self = Ppu::new();
81-    }
82-
83-    pub fn read_io(&self, off: u32) -> u16 {
84-        let i = ((off & !1) >> 1) as usize;
85-        match i {
86-            0 => self.r[0],
87-            1 => self.r[1] & 1,
88-            2 => {
89-                let lyc = self.r[2] >> 8;
```

**→ Bash**
```
{"command": "sed -n 76,84p ~/work/Gbe-test/gba-emu/src/apu.rs; sed -n 30,55p ~/work/Gbe-test/gba-emu/src/apu.rs", "description": "View rest of Apu::reset and struct"}
```

**← Bash result**
```
self.dirty = true;
        self.pend = 0;
        self.next_ev = 0;
        self.out.clear();
    }

    fn r16(&self, o: usize) -> u32 { self.r[o] as u32 | (self.r[o + 1] as u32) << 8 }
    fn master(&self) -> bool { self.r[0x24] & 0x80 != 0 }
    fn freq(&self, i: usize) -> u32 { self.r16([0x04, 0x0C, 0x14][i]) & 0x7FF }
#[derive(Default, Clone, Copy)]
struct Fifo {
    buf: [i8; 32],
    rd: usize,
    n: usize,
    cur: i8,
}

pub struct Apu {
    r: [u8; 0x30], // regs 0x60..0x8F
    wave: [[u8; 16]; 2],
    ch: [Chan; 4],
    fifo: [Fifo; 2],
    fs_step: u32,
    fs_left: u32,
    win_left: u32,
    hx: [f32; 2],
    hy: [f32; 2],
    cur: [i32; 2],
    dirty: bool,
    pend: u32,    // cycles not yet applied (lazy tick)
    next_ev: u32, // apply once pend reaches this; 0 = recompute
    out: Vec<i16>,
}

impl Apu {
```

**→ Bash**
```
{"command": "export PATH=/opt/homebrew/opt/rustup/bin:$PATH; S=/private/tmp/claude-501/-Users-USER-work-Gbe-test/2cbb11f1-a7a2-402f-bc9b-6a4faabf55e9/scratchpad; rm -rf $S/c1; mkdir -p $S/c1; cd ~/work/Gbe-test/gba-emu; cp -R Cargo.toml src assets $S/c1/; cd $S/c1; perl -0pi -e 's/self\\.set_waitcnt\\(0\\);\\n    \\}/self.after_rom_dma = false; self.set_waitcnt(0);\\n    }/' src/bus.rs; grep -n \"after_rom_dma = false; self.set_waitcnt\" src/bus.rs; CARGO_TARGET_DIR=$S/tgt cargo build --release --lib --target wasm32-unknown-unknown 2>&1 | tail -1; cd $S; node abi.mjs $S/tgt/wasm32-unknown-unknown/release/gba_emu.wasm ~/work/Gbe-test/gba-eval/corpus/roms/homebrew/bulletgba.gba 90 | grep -E \"IDENT|DIFF\"", "description": "Test hypothesis: stale after_rom_dma breaks reset determinism", "timeout": 600000}
```

**← Bash result**
```
128:        self.after_rom_dma = false; self.set_waitcnt(0);
    Finished `release` profile [optimized] target(s) in 0.67s
reset   { v: 'e7aff128e64d', a: 'dbeb7367f704', pairs: [ 548, 550 ] } IDENTICAL
reload  { v: 'e7aff128e64d', a: 'dbeb7367f704', pairs: [ 548, 550 ] } IDENTICAL
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-eval/harness/grader/src; sed -n 415,445p main.rs; grep -n \"WasmCandidate::new\" main.rs", "description": "Check whether grader reuses candidate instance"}
```

**← Bash result**
```
.with_context(|| format!("refcache for {} still missing after ensure_written", tc.id))?;
    let mut reference: Box<dyn Reference> = Box::new(cached);

    // Each worker owns its own candidate Store — a trap here only
    // poisons this testcase, not the run.
    let mut candidate = WasmCandidate::new(
        candidate_bytes,
        candidate_label.to_string(),
        config.fuel_per_frame,
        config.fuel_load_rom,
    ).context("instantiating candidate wasm")?;

    if let Err(e) = candidate.load_rom(&rom) {
        let msg = format!("{e:#}");
        let _ = writeln!(log, "  skip: candidate load_rom trapped for {}: {msg}", tc.id);
        return skip(log, format!("candidate load_rom trapped: {msg}"));
    }

    // Reference warmup is not burned here — lockstep() does it from
    // reference.boot_frames(). The candidate escape hatch runs on top.
    for _ in 0..config.candidate_boot_frames {
        candidate.run_frame();
    }

    // Optional video output: three ffmpeg children per testcase writing
    // <tc>.ref.mp4 / <tc>.cand.mp4 / <tc>.diff.mp4. Failure to spawn
    // (ffmpeg missing, permissions) demotes to a warning — scoring still
    // runs. Once spawned, per-frame write failures are swallowed inside
    // lockstep (see the `warning: video encoder failed…` log path).
    let video_base = out_dir.join(&tc.id);
    let mut encoder = if emit_video {
150:    let _ = WasmCandidate::new(
399:            let mut r = WasmCandidate::new(
420:    let mut candidate = WasmCandidate::new(
550:    // (via `WasmCandidate::new`) and writes to a unique path, so they
588:                let mut reference = WasmCandidate::new(
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-emu; sed -n 138,172p src/ppu.rs; grep -c \"\" src/*.rs examples/*.rs tests/*.rs scripts/* | sort -t: -k2 -n | tail -8", "description": "View PPU tick IRQ generation and file sizes"}
```

**← Bash result**
```
}

    pub fn tick(&mut self, mut cycles: u32) -> PpuEvents {
        let mut ev = PpuEvents { irq: self.pending_irq, hblank_start: false, vblank_start: false };
        self.pending_irq = 0;
        while cycles > 0 {
            let bound = if self.cyc < HB { HB } else { LINE };
            let step = cycles.min(bound - self.cyc);
            self.cyc += step;
            cycles -= step;
            if self.cyc == HB {
                if self.line < 160 {
                    self.render_line();
                    ev.hblank_start = true;
                }
                if self.r[2] & 0x10 != 0 { ev.irq |= 2; }
            } else if self.cyc == LINE {
                self.cyc = 0;
                self.line += 1;
                if self.line == 228 { self.line = 0; }
                if self.line == 0 {
                    self.refx[0] = sext28(self.r[0x14] as u32 | (self.r[0x15] as u32) << 16);
                    self.refy[0] = sext28(self.r[0x16] as u32 | (self.r[0x17] as u32) << 16);
                    self.refx[1] = sext28(self.r[0x1C] as u32 | (self.r[0x1D] as u32) << 16);
                    self.refy[1] = sext28(self.r[0x1E] as u32 | (self.r[0x1F] as u32) << 16);
                }
                if self.line == 160 {
                    ev.vblank_start = true;
                    if self.r[2] & 8 != 0 { ev.irq |= 1; }
                }
                if self.r[2] & 0x20 != 0 && self.line as u16 == self.r[2] >> 8 { ev.irq |= 4; }
            }
        }
        ev
    }
scripts/grade.sh:38
examples/ppu_test.rs:39
src/emu.rs:91
src/lib.rs:95
src/apu.rs:457
src/ppu.rs:499
src/bus.rs:732
src/cpu.rs:1226
```

**→ Bash**
```
{"command": "export PATH=/opt/homebrew/opt/rustup/bin:$PATH; S=/private/tmp/claude-501/-Users-USER-work-Gbe-test/2cbb11f1-a7a2-402f-bc9b-6a4faabf55e9/scratchpad; cd ~/work/Gbe-test/gba-emu; grep -n \"irq_prev\\|pf_buf\\|wasted\" src/cpu.rs src/bus.rs | grep -v \"self.wasted\" | head; head -30 tests/m4.rs; touch -c src/lib.rs 2>/dev/null; CARGO_TARGET_DIR=$S/tgt2 cargo build --release --lib --target wasm32-unknown-unknown 2>&1 | grep -A3 \"^warning\" | grep -E \"^warning|-->\" | head -20", "description": "Check unused fields, m4 test, compiler warnings", "timeout": 600000}
```

**← Bash result**
```
src/bus.rs:76:    pf_next: u32, pf_buf: u32, pf_last: u64,
src/bus.rs:107:            pf_next: 0, pf_buf: 0, pf_last: 0, pf_valid: false, pf_head: 0, pf_cnt: 0, pf_flight: false, pf_done: 0, pf_ws: 0,
src/bus.rs:127:        self.pf_next = 0; self.pf_buf = 0; self.pf_last = 0; self.pf_valid = false; self.pf_cnt = 0; self.pf_flight = false;
src/cpu.rs:129:    wasted: bool,
src/cpu.rs:130:    irq_prev: bool,
src/cpu.rs:222:            wasted: false,
src/cpu.rs:223:            irq_prev: false,
src/cpu.rs:346:    /// Branch: the fetch already in flight (at r15) is wasted, then the pipeline refills.
src/cpu.rs:348:        // ROM -> non-ROM branch: the wasted ROM fetch overlaps on the cartridge bus (1 cycle)
src/cpu.rs:359:    /// The wasted in-flight fetch; loads into PC issue it before their data cycles.
use gba_emu::emu::Emu;
#[test]
fn mode4() {
    let mut m = Emu::new();
    m.bus.rom_len = 16; m.bus.reset();
    m.bus.write16(0x0400_0000, 0x0404, false);
    m.bus.write32(0x0400_0020, 0x0100, false);
    m.bus.write32(0x0400_0024, 0x0, false);
    m.bus.write32(0x0400_0030, 0x0100, false);
    m.bus.write16(0x0500_0002, 0x001F, false);
    for i in 0..240u32 { m.bus.write16(0x0600_0000 + i * 2, 0x0101, false); }
    for _ in 0..3 { m.bus.cycles += 280896; m.bus.sync(); }
    let f = &m.bus.ppu.fb;
    println!("px0={:08x} px1={:08x} row50={:08x}", f[0], f[1], f[50*240]);
}
warning: unused variable: `a`
   --> src/bus.rs:584:38
warning: unused variable: `seq`
   --> src/bus.rs:584:46
warning: constant `INF` is never used
 --> src/apu.rs:4:7
warning: constant `PF_CAP` is never used
  --> src/bus.rs:15:7
warning: constant `PF_MISS_EXTRA` is never used
  --> src/bus.rs:16:7
warning: field `irq_prev` is never read
   --> src/cpu.rs:130:5
warning: `gba-emu` (lib) generated 6 warnings (run `cargo fix --lib -p gba-emu` to apply 2 suggestions)
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-emu; grep -n \"timer =\" src/apu.rs; grep -n \"fn period\\|fn step_chan\" -A8 src/apu.rs | head -40", "description": "Check APU channel timer reload logic"}
```

**← Bash result**
```
113:        c.timer = per;
274:                        if self.ch[i].timer == 0 { self.step_chan(i); }
90:    fn period(&self, i: usize) -> u32 {
91-        match i {
92-            0 | 1 => (2048 - self.freq(i)) * 16,
93-            2 => (2048 - self.freq(2)) * 8,
94-            _ => {
95-                let h = self.r[0x1C] as u32;
96-                let rr = h & 7;
97-                let base = if rr == 0 { 16 } else { 32 * rr };
98-                base << ((h >> 4) + 1).min(30)
--
179:    fn step_chan(&mut self, i: usize) {
180-        let per = self.period(i);
181-        match i {
182-            0 | 1 => { let c = &mut self.ch[i]; c.pos = (c.pos + 1) & 7; c.timer += per; }
183-            2 => { let c = &mut self.ch[2]; c.pos = (c.pos + 1) & 63; c.timer += per; }
184-            _ => {
185-                let w7 = self.r[0x1C] & 8 != 0;
186-                let c = &mut self.ch[3];
187-                let x = (c.lfsr ^ (c.lfsr >> 1)) & 1;
```

**→ Bash**
```
{"command": "cd ~/work/Gbe-test/gba-emu; grep -n \"fn rgb\" -A6 src/ppu.rs; grep -n \"fn blend\\|fn fade\" -A6 src/ppu.rs | head -20; sed -n 1,12p src/apu.rs", "description": "Check pixel format conversion and blend"}
```

**← Bash result**
```
39:fn rgb(c: u16) -> u32 {
40-    let r = (c & 31) as u32;
41-    let g = ((c >> 5) & 31) as u32;
42-    let b = ((c >> 10) & 31) as u32;
43-    let e = |v: u32| (v << 3) | (v >> 2);
44-    0xFF00_0000 | (e(b) << 16) | (e(g) << 8) | e(r)
45-}
48:fn blend(a: u16, b: u16, ea: u32, eb: u32) -> u16 {
49-    let ch = |s: u32| (((((a as u32) >> s) & 31) * ea + (((b as u32) >> s) & 31) * eb) >> 4).min(31);
50-    (ch(0) | (ch(5) << 5) | (ch(10) << 10)) as u16
51-}
52-
53-#[inline]
54:fn fade(a: u16, y: u32, up: bool) -> u16 {
55-    let ch = |s: u32| {
56-        let c = ((a as u32) >> s) & 31;
57-        if up { c + (((31 - c) * y) >> 4) } else { c - ((c * y) >> 4) }
58-    };
59-    (ch(0) | (ch(5) << 5) | (ch(10) << 10)) as u16
60-}
//! APU (apu agent): PSG ch1-4, Direct Sound A/B, mixer. Output 32768 Hz stereo = one sample / 512 cycles.
//! Event-driven: each window of 512 cycles integrates the mixed level, so square waves are box-filtered.

const INF: u32 = u32::MAX;
const WIN: u32 = 512;
const HP: f32 = 0.9962;
const FS_PERIOD: u32 = 32768; // 512 Hz frame sequencer
const DUTY: [u8; 4] = [0b0000_0001, 0b1000_0001, 0b1000_0111, 0b0111_1110];

// Readable bits per 16-bit register at offset 0x60 + 2*i.
const RMASK: [u16; 24] = [
    0x007F, 0xFFC0, 0x4000, 0, 0xFFC0, 0, 0x4000, 0, 0x00E0, 0xE000, 0x4000, 0, 0xFF00, 0, 0x40FF, 0,
```

**→ Bash**
```
{"command": "export PATH=/opt/homebrew/opt/rustup/bin:$PATH; S=/private/tmp/claude-501/-Users-USER-work-Gbe-test/2cbb11f1-a7a2-402f-bc9b-6a4faabf55e9/scratchpad; cd $S/c1; mkdir -p tests; cat > tests/res.rs <<'EOF'\nuse gba_emu::emu::Emu;\n#[test]\nfn res3() {\n    for res in 0..4u32 {\n        let mut m = Emu::new();\n        // ROM: strb r1,[r0] ; b . with r0=0x04000089, r1=res<<6|2  (mov via ldr literal)\n        let code: [u32; 5] = [0xE59F0008, 0xE59F1008, 0xE5C01000, 0xEAFFFFFE, 0x0400_0089];\n        let mut rom = vec![0u8; 0x200];\n        for (i, w) in code.iter().enumerate() { rom[i*4..i*4+4].copy_from_slice(&w.to_le_bytes()); }\n        rom[20..24].copy_from_slice(&((res << 6) | 2).to_le_bytes());\n        m.bus.rom[..rom.len()].copy_from_slice(&rom);\n        assert!(m.load_rom(rom.len()));\n        m.drain_audio();\n        let mut v = vec![];\n        for _ in 0..3 { m.run_frame(); v.push(m.drain_audio()); }\n        println!(\"res={res} pairs/frame={v:?} rate={}\", gba_emu::emu::AUDIO_RATE);\n    }\n}\nEOF\nCARGO_TARGET_DIR=$S/tgt cargo test --release --test res -- --nocapture 2>&1 | grep -E \"res=|panicked|error\"", "description": "Measure audio pairs per frame for each SOUNDBIAS resolution", "timeout": 600000}
```

**← Bash result**
```
res=0 pairs/frame=[548, 549, 549] rate=32768
res=1 pairs/frame=[1097, 1098, 1097] rate=32768
res=2 pairs/frame=[2194, 2195, 2195] rate=32768
res=3 pairs/frame=[4096, 4096, 4096] rate=32768
```

### Assistant · 2026-11-09 15:51
Review timestamps (EET): docs, lib.rs, emu.rs and save.rs read at 15:43; bus.rs at 15:43:52; cpu.rs at 15:45; apu.rs and ppu.rs at 15:45–15:46. I built and tested into a target directory in my scratchpad and edited nothing under gba-emu/. All 13 tests pass, with 6 compiler warnings.

## gba-emu review: top 5 must-fix

**1. `emu_reset` and a second `emu_load_rom` do not give the same state as a first load (ABI break, confirmed).**
- `Bus::reset` (bus.rs:114-129) never clears `after_rom_dma`. That flag is set at bus.rs:661 and makes the next non-sequential ROM access cost N-1 (bus.rs:212).
- Repro with my probe (`scratchpad/abi.mjs`): bulletgba, 90 frames, then reset or reload. Video hash is the same; audio hash `dbeb…` becomes `afb1…`. Adding `self.after_rom_dma=false` to a scratch copy makes it IDENTICAL.
- Other fields `reset` also skips: `ppu_early`, `restart`, `cur_dma`, `dma_ready`/`dma_lat`, `pf_head`/`pf_done`/`pf_ws`/`pf_buf`, and `Apu::cur`.
- The grader makes a new instance per testcase (grader main.rs:420), so the score does not move. A host that reuses one instance gets nondeterminism.
- Fix: rebuild the whole struct and keep only the ROM: `let rom=take(&mut self.rom); *self=Bus::new(); self.rom=rom;`.

**2. Audio rate and buffer size are wrong when SOUNDBIAS resolution is not 0.**
- apu.rs:280-282 emits at 32768<<res Hz. `emu_audio_rate` always returns 32768 (emu.rs:5).
- `AUDIO_CAP` is 4096 pairs (emu.rs:8). Measured pairs per frame for res 0/1/2/3: 548 / 1097 / 2194 / **4096 (capped)**. At res=3, about 293 pairs are silently dropped every frame.
- The grader scores with the reference's rate, so only the pair count matters. A hidden ROM that uses res=3 would drift and its audio score would collapse.
- apu.rs:1-2 still says "Output 32768 Hz, box-filtered". The code takes a point sample at the end of each window.
- Fix: raise the cap to at least 8192 pairs, and either resample to a fixed rate or report the real one.

**3. `mull_carry` is fitted to test operands, not derived from hardware (cpu.rs:191-207).**
- The branch `rs==1 && rm>>26&31==1` exists only to cover mgba-multiply-long's operand pairs. Every other 1-3 cycle multiplier returns 0.
- UMLAL/SMLAL reuse the same rule and ignore the accumulator. This is unverified.
- The comment at cpu.rs:192 says "exact for 4-cycle". KNOWN_LIMITATIONS says it matches about 91% (UMULL) and 87% (SMULL).
- A hidden multiply test with other operands will fail. Fix: derive the carry from the ARM7TDMI Booth / carry-save multiplier, or at least label it test-fitted everywhere.

**4. IRQ timing fits apply to IRQs they were not fitted for.**
- `ppu_early` (bus.rs:136, 508) is a one-shot set by any enabled PPU IRQ. The next IE&IF rising edge from *any* source consumes it: timer, DMA, keypad, or an IE write (bus.rs:439). That edge is then backdated by 2 cycles.
- `take` is sampled before the instruction runs (cpu.rs:407, 443). So the IRQ is entered even when that instruction just set CPSR.I, wrote IME=0, or was a SWI. In the SWI case the IRQ is taken at the SVC vector with I already set.
- DECISIONS.md already notes heartwrench@342930 fires one instruction early.
- Fix: backdate only when the edge comes from a PPU flag in the same chunk, and re-check CPSR.I after a write to CPSR.

**5. No automated test protects the ABI behaviour (this is why #1 went unnoticed).**
- tests/m4.rs has no assertions; it only prints.
- Nothing tests load==reset, golden frame/audio hashes, 32 MiB or odd-size ROMs, pairs per frame, or the SRAM path.
- grade.sh is the only regression guard, and it needs the reference wasm.

## A: Logic and correctness (other findings)
- **EEPROM covers the whole 0x0D region** when the save kind is Eeprom (bus.rs:315, 366). A 32 MiB EEPROM cart would read EEPROM bits instead of its upper 16 MiB of ROM. Hardware only puts EEPROM at 0x0DFFFF00+ for such carts.
- **Window wrap-around** (ppu.rs:262-268): x1>x2 or y1>y2 is clamped to [x1,240) instead of wrapping to [x1,240)∪[0,x2). GBATEK describes the clamp, but comparator-style hardware and accurate emulators wrap. Check this against the reference.
- **Shared DMA timing fields** (bus.rs:559, 574, 580): `dma_ready` and `dma_lat` are shared by all channels. An event trigger overwrites a pending immediate DMA's start time, and the reverse.
- **Odd-size ROM tail** (bus.rs:319): a word straddling `rom_len` returns the address-echo pattern for the whole word. Minor.
- **Calibration constants** fitted to the reference: IRQ_DELAY=6, EV_X=5, PPU_IRQ_EARLY=2, VBLANK_DELAY=8, START_DELAY/READ_ADJ, the +2/+1 branch penalties at cpu.rs:349-354, and the DC blocker 0.9962. They are disclosed, but which trace each was fitted to is not recorded next to it in code.
- **No trap paths found:**
  - PPU palette index stays ≤1023 and VRAM is masked.
  - IO index stays ≤0x1FF and the EEPROM bounds hold.
  - `pf_cnt -= 1` at bus.rs:250 cannot underflow while "flight || cnt==8" holds, but nothing asserts that invariant. A `debug_assert` would help.
- **Docs fuel figure:** README says a 300G load budget; the grader default is 60G (grader main.rs:57). The save scan (4×32 MiB per reset) still fits.

## B: Code quality
- **Files over the 500-line rule:** cpu.rs (1226) and bus.rs (732). ppu.rs (499) and apu.rs (457) stay under it partly through very long one-liners. lib.rs:58 is one line of about 600 characters.
- **`static mut` in lib.rs:9-15** with `allow(static_mut_refs)`. This becomes a hard error in edition 2024. Use a `thread_local!` RefCell or an UnsafeCell wrapper.
- **Debug code in the library API:**
  - lib.rs:28-95 (`run`, `trace`, `dump`, `steps`) uses `env::var` (MEMLOG, STEPDBG, DUMPAT), `unwrap`, `println` and a thread_local. It is stripped from the wasm (I checked: 0 imports), but belongs in examples/ or under `cfg(not(wasm32))`.
  - cpu.rs:1171-1225 holds env-driven `#[ignore]` tests that duplicate examples/run.rs.
  - The `trace_mem` checks run on the hot path (bus.rs:189, 237).
- **Dead code:**
  - `brk()` is a no-op, and its doc says "idle(0) marks the break", which ARCH.md forbids (cpu.rs:29-33).
  - Unused items: `irq_prev`, `_UNUSED` (cpu.rs:1103), `pf_buf`, `irq_age`.
  - The `restart` branch never runs because `restart` is never set (bus.rs:650).
  - Constants ABORT_ON_DISABLE, PF_DATA_WAIT, PF_CAP, PF_MISS_EXTRA, VCOUNT_ADJ and INF are dead or no-ops.
  - `dma_gate` ignores its `a` and `seq` arguments, while its doc still says "next data/non-sequential access" (bus.rs:583-584).
- **Unenforced contract:** the "call `sync_to(self.mark)` before reading ticked state" rule exists only in DECISIONS.md. DISPSTAT/VCOUNT reads use `sync_to(cycles)` instead (bus.rs:377). The `fetch16`/`fetch32` override does have a guard test, which is good.
- **Stale docs:**
  - KNOWN_LIMITATIONS opens with the 0.6428 run (multiply-long, dma and io-read ≈0), contradicting "all other procedural = 1.0" further down.
  - NEXT_STEPS still lists multiply-long as a priority.
  - The cpu.rs:192 comment, apu.rs:1, and the README fuel figure are wrong (above).
  - README points to `.claude-flow/data/scores.md`, which is outside the gba-emu repo.
- **Handoff and compliance risks:**
  - The task asks for Git history; there is none, only tarballs.
  - Behaviour fitted by sweeping random inputs through Mesen (MULL carry, the DC coefficient) is disclosed only in scattered places. List it in one place as "reference-fitted, not hardware-derived".
  - Regenerating the refcache inside gba-eval is disclosed.

## What looks solid
- The wasm has all 10 exports and 0 imports. ROM and framebuffer pointers are stable. Lengths that are negative or over 32 MiB return 0 cleanly.
- A ROM padded to 32 MiB, or one byte longer than its real size, gives the same frames as the original. The 32 MiB mirror mask and open bus past the ROM end look right.
- Reset and reload are deterministic on 6 of 7 ROMs tested; bulletgba is the exception (#1).
- The audio drain works as specified: a call returns the count and empties the buffer, and when the host doesn't drain, the oldest samples are dropped at 4096 pairs.
- Batched sync, the explicit prefetch model, and the fetch-override guard test are well thought out.

Scratch probe and repro test: `/private/tmp/claude-501/-Users-USER-work-Gbe-test/2cbb11f1-a7a2-402f-bc9b-6a4faabf55e9/scratchpad/abi.mjs` and `/private/tmp/claude-501/-Users-USER-work-Gbe-test/2cbb11f1-a7a2-402f-bc9b-6a4faabf55e9/scratchpad/c1/tests/res.rs`.

### Agent report · 2026-11-09 15:51
<teammate-message teammate_id="team-lead" summary="Send the truncated remainder of the review">
Your report was truncated after the first two 'A: Logic and correctness (other findings)' bullets (EEPROM region, window wrap-around). Please resend everything after that point: remaining A findings, all of axis B (unsafe/static mut, debug code in non-test paths, files over 500 lines, duplication, test gaps, fragile contracts, doc-vs-code mismatches, originality/disclosure risks) and the 'looks solid' list, as a compact list with file:line, <= 45 lines. Also, for each of your top-5 items, tell me the smallest safe fix and whether it can change the grade.sh score.
</teammate-message>

**→ SendMessage**
```
{"to": "team-lead", "summary": "Rest of review plus top-5 fixes and score impact", "message": "Rest of the gba-emu review (files read 15:43-15:46 EET), plus the smallest safe fix for each top-5 item.\n\nTOP-5 FIXES AND WHETHER THEY CAN MOVE grade.sh\n1. emu_reset/reload not identical to a first load (after_rom_dma is never cleared). Smallest fix: in Bus::reset (bus.rs:128) add `self.after_rom_dma=false; self.ppu_early=0; self.restart=0; self.dma_ready=0; self.dma_lat=2; self.cur_dma=0; self.pf_head=0; self.pf_done=0; self.pf_ws=0;`. The fuller version is `let rom=take(&mut self.rom); *self=Bus::new(); self.rom=rom;` followed by the existing post-BIOS writes. Score: no change. The grader creates a fresh instance per testcase (gba-eval grader main.rs:420), and on a first load these fields already hold their new() values. Add the load==reset test from #5 alongside it.\n2. Audio cap and rate. Smallest fix: raise AUDIO_CAP (emu.rs:8) to 16384*2 i16. Score: no change for the corpus, because truncation only happens at SOUNDBIAS res=3 and no corpus ROM hit the cap in my runs. Changing what emu_audio_rate reports is also score-neutral: the grader uses the reference's rate (ref_cache.rs:191, lockstep lib.rs:210). Do NOT resample to a fixed 32768: the pair counts would no longer match the reference and audio scores would drop.\n3. mull_carry fitted to test operands. Smallest safe fix: correct the docs only (cpu.rs:192 says \"exact for 4-cycle\"; KNOWN_LIMITATIONS says ~91%/87%; mark it 
… [truncated 5826 chars]
```

**← SendMessage result**
```
{"success":true,"message":"Message sent to team-lead's inbox","msg_id":"97fa371e-433b-4cd7-bf93-4fcd0886f0fb","routing":{"sender":"opus-reviewer","senderColor":"purple","target":"@team-lead","targetColor":"red","summary":"Rest of review plus top-5 fixes and score impact","content":"Rest of the gba-emu review (files read 15:43-15:4…"}}
```

### Assistant · 2026-11-09 15:51
I sent the team lead the rest of the review, plus the smallest safe fix for each top-5 item and whether it can move the grade.sh score.

| # | Problem | Smallest safe fix | Can it change the score? |
|---|---|---|---|
| 1 | `emu_reset` and a second `emu_load_rom` don't give the same state as a first load. `after_rom_dma` and a few other fields are never cleared. | Clear the missing fields in `Bus::reset` (`bus.rs:128`), or rebuild the whole struct and keep only the ROM. | No. The grader makes a fresh emulator per testcase. |
| 2 | The audio buffer holds 4096 sample pairs, so at SOUNDBIAS res=3 about 293 pairs are dropped every frame. The reported rate is always 32768. | Raise `AUDIO_CAP` (`emu.rs:8`) to 16384 pairs. | No for the corpus, which never fills the buffer. Don't resample to a fixed 32768: that breaks the match with the reference and lowers the audio score. |
| 3 | `mull_carry` is fitted to the test's operands, not derived from how the hardware works. | Correct the comment and the docs so they say it's test-fitted. | Not for the doc fix. A real derivation could break `mgba-multiply-long`, which passes now, so grade it before keeping it. |
| 4 | A timing fit for display interrupts (`ppu_early`) also fires 2 cycles early for unrelated interrupts. | Apply it only when the interrupt comes from the display. | Yes, replay scores can move either way, so grade it. If there's no time, document it instead. |
| 5 | No tests check reset against a first load, or audio samples per frame. | Add those two tests and give `tests/m4.rs` real assertions. | No. |

The second part also covers the remaining logic findings and all the code-quality ones: `static mut`, debug code, files over 500 lines, dead code, duplication, test gaps, out-of-date docs and disclosure risks. It ends with the list of what looks solid. Every item has a file:line.

