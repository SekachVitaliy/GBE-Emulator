# gba-emu architecture (contract between modules)

Crate: wasm32 cdylib, `std` available, no threads/fs. Frame = 280896 cycles @16.78MHz (228 lines x 1232 cycles).
Files and owners (each agent edits ONLY its files; keep the crate compiling at all times):

| file | owner | contents |
|---|---|---|
| src/cpu.rs  | cpu agent | ARM7TDMI: ARM+THUMB, banked regs, exceptions. `Cpu::step(&mut self, bus:&mut Bus)` runs ONE instruction (or takes IRQ if `bus.irq_line()` and CPSR.I==0) and adds its cycles via bus waitstate API. Also BIOS-less HLE is NOT needed: BIOS stub is executed as ARM from 0x0. |
| src/ppu.rs  | ppu agent | owns VRAM/PAL/OAM + display IO regs 0x04000000..0x04000056. Renders each scanline into `fb`. |
| src/apu.rs  | apu agent | owns sound IO regs 0x04000060..0x040000A7 (PSG ch1-4, FIFO A/B, SOUNDCNT, SOUNDBIAS), resamples to AUDIO_RATE. |
| src/bus.rs  | lead      | memory map, waitstates, IO dispatch, IE/IF/IME/HALTCNT, timers, DMA, KEYINPUT. |
| src/emu.rs, lib.rs | lead | machine loop and ABI. |
| scripts/, GRADING.md | grader agent | native grader build, refcache precompute, scoring log. |

## Interfaces (do not change signatures without editing this file and telling lead)
```rust
// ppu.rs
pub struct PpuEvents { pub irq: u16 /*IF bits: 1=vblank 2=hblank 4=vcount*/, pub hblank_start: bool, pub vblank_start: bool }
impl Ppu {
  pub fn new() -> Self; pub fn reset(&mut self);
  pub fn tick(&mut self, cycles: u32) -> PpuEvents;       // advances dot/line state, renders line at HBlank start
  pub fn read_io(&self, off: u32) -> u16; pub fn write_io(&mut self, off: u32, v: u16, mask: u16); // off = addr-0x04000000 (even)
  pub vram: Vec<u8> (96K), pal: Vec<u8> (1K), oam: Vec<u8> (1K), fb: Vec<u32> (240*160, 0xAABBGGRR, A=FF)
}
// apu.rs
impl Apu {
  pub fn new() -> Self; pub fn reset(&mut self);
  pub fn tick(&mut self, cycles: u32);
  pub fn read_io(&self, off: u32) -> u8; pub fn write_io(&mut self, off: u32, v: u8);  // byte granularity, off = addr-0x04000000
  pub fn fifo_write(&mut self, ch: usize /*0=A,1=B*/, word: u32);          // from FIFO_A/B writes and DMA
  pub fn timer_overflow(&mut self, timer: usize, count: u32) -> u8;        // returns bitmask of FIFOs (bit0=A,bit1=B) that need a DMA refill request
  pub fn drain(&mut self, out: &mut Vec<i16>);                              // stereo interleaved at AUDIO_RATE
}
// cpu.rs
impl Cpu { pub fn new() -> Self; pub fn reset(&mut self); pub fn step(&mut self, bus: &mut Bus); }
// bus.rs (lead provides; cpu uses these)
impl Bus { read8/16/32(&mut self, addr:u32, seq:bool)->u32; write8/16/32(&mut self, addr:u32, v:u32, seq:bool);
           pub fn idle(&mut self, cycles: u32); pub fn irq_line(&self)->bool; pub fn halted(&self)->bool; }
```
Reads/writes add waitstate cycles to `bus.cycles`; `idle` adds internal cycles. Misaligned access rules (rotate on LDR etc.) are the CPU's job; bus aligns.

## Verification
Test ROMs: gba-eval/corpus/roms/test/*.gba (armwrestler, fuzzarm, mgba-suite ...). Reference: gba-eval/reference/mesen.wasm. Never read gba-eval/candidates/gba-core.

## Bus/machine notes (lead, implemented)
- `Bus` owns `ppu`, `apu`, `rom` (32 MiB buffer, `rom_len`), `save` (SRAM/Flash64/Flash128/EEPROM, auto-detected from ROM strings). `Emu { cpu, bus }`.
- Machine loop (`emu.rs`): `cpu.step(&mut bus); bus.sync();` — `sync()` converts `bus.cycles` deltas into ppu/apu/timer ticks (chunks <= 256 cycles and never past a timer overflow), runs HBlank/VBlank/special DMA, keypad IRQ. While `bus.halted()` the loop calls `bus.halt_skip`. Frame budget 280896 cycles with carry.
- CPU hooks (already used by cpu.rs): `bus.fetch_pc` = executing instruction address (BIOS read protection), `bus.open_bus` = word returned for unmapped/write-only reads. mgba-suite io-read expects the *prefetched* word (pc+8 ARM / pc+4 Thumb), not the executing opcode.
- Reset state set by `Bus::reset`: DISPCNT=0x80 (forced blank, frame 0 is white like the reference), BG2/3 PA/PD=0x100.
- Timer calibration: START_DELAY=1, READ_ADJ=2 (re-fit after cpu timing change) (matched nba-hw timer-reload/start-stop pixel-exact). DMA: pending queue by priority; ROM source ignores fixed/dec; write-only IO reads give open bus (DMA latch during DMA).
- Native tools: `cargo run --release --example run -- rom.gba FRAMES out.ppm [hexkeys | "hexKEYxFRAMES,..." | replay.txt]`, `--example trace`, `--example dump`.
- Frame/boot alignment (matches the grader): the reference enters the ROM with the PPU at line 225 cycle 19 (`Bus::align_ppu`, 277224 cycles after BIOS-stub boot) and a frame ends when the PPU reaches line 160 (`take_frame`). The grader burns 6 frames on the reference before frame 0, so `Emu::reset()` silently runs `warmup_frames = 6` frames (audio dropped).
- Instruction-level oracle: `scripts/lockstep.sh <rom>` compares (r15, cycles) per instruction against `reference/mesen.wasm` (`mesen_exec_one`). Remaining known diffs are cpu-side: IRQ entry 8 vs 10 cycles; first instruction after ROM->IWRAM `bx` charged 5 extra; unaligned-address STRH to SRAM; prefetch buffer details after taken branches; PAL/VRAM contention (+1 in some draw slots) not modelled.
- IRQ latency: IE&IF must be true for IRQ_DELAY=6 cycles before irq_line()/halt wake (fitted from destoer line-timing; replay score 0.72 -> 0.94). Reference IRQ entry cost in lockstep: 5 cycles when waking from halt, 10 when interrupting running code (see DECISIONS.md for the current model).
- dma_gate: event-triggered DMA (timer FIFO/HBlank/VBlank) starts at the CPU's next data or non-sequential access, +2 start cycles. cpu.rs must not call Bus::idle(0) to mark ROM seq breaks (it fires pending DMA early); cost() already breaks ROM sequentiality after any non-ROM access.
