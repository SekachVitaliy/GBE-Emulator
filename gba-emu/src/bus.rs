//! Memory bus (lead-owned): memory map, waitstates, IO dispatch, IRQ regs, timers, DMA.
//! Also owns Ppu and Apu; `sync()` advances every component by the cycles the CPU consumed.
pub mod save;
mod dma;
mod io;
mod prefetch;
mod timers;
use crate::apu::Apu;
use crate::ppu::Ppu;
use save::Save;

static BIOS: &[u8] = include_bytes!("../assets/bios.bin");
const PRESCALE: [u32; 4] = [0, 6, 8, 10];
// calibration knobs (checked against nba-hw timer tests)
const VBLANK_DELAY: u32 = 8;
const IRQ_DELAY: u64 = 6;
const START_DELAY: u32 = 1;
const READ_ADJ: u32 = 2;
// prefetch fit (bulletgba/heartwrench lockstep): a branch waits for an in-flight halfword fetch only if it already started;
// a data read from ROM restarts the producer N+1 cycles after the access ends
const PF_DATA_RESTART: u64 = 1;
// a halfword/word data read from ROM discards the prefetch buffer (verified on xniq/bulletgba/collie lockstep)
const FLSZ: u32 = 2;
const PPU_IRQ_EARLY: u64 = 2;
const EV_X: u64 = 5;
const NWAIT: [u32; 4] = [4, 3, 2, 8];

#[derive(Clone, Copy, Default)]
struct Timer { reload: u16, counter: u16, ctrl: u16, acc: u32, on: bool, delay: u32 }
#[derive(Clone, Copy, Default)]
struct Dma { src: u32, dst: u32, left: u32 }

pub struct Bus {
    pub cycles: u64,
    synced: u64,
    pub ppu: Ppu,
    pub apu: Apu,
    pub rom: Vec<u8>,
    pub rom_len: usize,
    pub save: Save,
    /// CPU may set this to the address of the executing instruction; BIOS reads are
    /// protected (return `bios_latch`) when it is >= 0x4000.
    pub fetch_pc: u32,
    /// Value returned for unmapped reads; CPU may update it with the last prefetched opcode.
    pub open_bus: u32,
    bios_latch: u32,
    ewram: Vec<u8>,
    iwram: Vec<u8>,
    io: [u16; 0x200],
    ie: u16, iflag: u16, ime: u16,
    halt: bool,
    /// just left halt: first instruction costs +2 and cannot take the IRQ (cleared by the next sync)
    woke: bool, wake_pen: bool,
    /// batching: components are only ticked once `cycles` reaches `next_ev` (next timer/line/256-cycle event); `mark` = cycles at the last instruction boundary
    next_ev: u64, mark: u64, eq: u64, ppu_early: u64,
    postflg: u8,
    waitcnt: u16,
    rom_n: [u32; 3], rom_s: [u32; 3], sram_w: u32, prefetch: bool,
    pub keys: u32,
    key_cond: bool,
    /// set when the PPU reaches line 160 (end of visible frame); cleared by `take_frame`
    frame_done: bool,
    cond_since: u64,
    /// PPU position within the frame in cycles (tracked here to model flag delays)
    ppu_pos: u32,
    /// native-only lockstep memory-access log (compiled out of the wasm build)
    #[cfg(not(target_arch = "wasm32"))]
    pub trace_mem: bool,
    #[cfg(not(target_arch = "wasm32"))]
    pub memlog: Vec<(u32, u32, u32, bool)>,
    timers: [Timer; 4],
    dma: [Dma; 4],
    dma_latch: u32,
    latches: [u32; 4],
    in_dma: bool,
    dma_pending: u8, fifo_mask: u8, cur_dma: usize, after_rom_dma: bool, dma_ready: [u64; 4], dma_lat: [u64; 4],
    fifo_buf: [u32; 2],
    pf_next: u32, pf_last: u64,
    // explicit prefetch buffer: pf_cnt halfwords at pf_head, optional in-flight halfword fetch finishing at pf_done
    pf_valid: bool, pf_head: u32, pf_cnt: u32, pf_flight: bool, pf_done: u64, pf_ws: usize,
}

/// 96 KiB VRAM: the top 32 KiB of the 128 KiB window mirror 0x10000..0x18000
#[inline] fn vram_off(o: u32) -> usize { (if o >= 0x18000 { o - 0x8000 } else { o }) as usize }
fn ld(b: &[u8], i: usize, sz: u32) -> u32 {
    match sz {
        1 => b[i] as u32,
        2 => u16::from_le_bytes([b[i], b[i + 1]]) as u32,
        _ => u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]),
    }
}
fn st(b: &mut [u8], i: usize, sz: u32, v: u32) {
    match sz {
        1 => b[i] = v as u8,
        2 => b[i..i + 2].copy_from_slice(&(v as u16).to_le_bytes()),
        _ => b[i..i + 4].copy_from_slice(&v.to_le_bytes()),
    }
}

impl Bus {
    pub fn new() -> Self {
        let mut b = Bus {
            cycles: 0, synced: 0, ppu: Ppu::new(), apu: Apu::new(),
            rom: vec![0; 32 << 20], rom_len: 0, save: Save::new(save::Kind::Sram),
            fetch_pc: 0x0800_0000, open_bus: 0, bios_latch: 0xE129_F000,
            ewram: vec![0; 0x40000], iwram: vec![0; 0x8000], io: [0; 0x200],
            ie: 0, iflag: 0, ime: 0, halt: false, woke: false, wake_pen: false, next_ev: 0, mark: 0, eq: 0, ppu_early: 0, postflg: 0, waitcnt: 0,
            rom_n: [5; 3], rom_s: [3, 5, 9], sram_w: 5, prefetch: false,
            keys: 0, key_cond: false, frame_done: false, cond_since: u64::MAX, ppu_pos: 0, #[cfg(not(target_arch = "wasm32"))] trace_mem: false, #[cfg(not(target_arch = "wasm32"))] memlog: Vec::new(), timers: [Timer::default(); 4], dma: [Dma::default(); 4],
            dma_latch: 0, latches: [0; 4], in_dma: false, dma_pending: 0, fifo_mask: 0, cur_dma: 0, after_rom_dma: false, dma_ready: [0; 4], dma_lat: [2; 4], fifo_buf: [0; 2],
            pf_next: 0, pf_last: 0, pf_valid: false, pf_head: 0, pf_cnt: 0, pf_flight: false, pf_done: 0, pf_ws: 0,
        };
        b.set_waitcnt(0);
        b
    }

    /// Reset everything except ROM contents. Detects the save type from the ROM.
    pub fn reset(&mut self) {
        let (rom, rom_len) = (std::mem::take(&mut self.rom), self.rom_len);
        #[cfg(not(target_arch = "wasm32"))]
        let trace_mem = self.trace_mem;
        *self = Bus::new();
        self.rom = rom; self.rom_len = rom_len;
        #[cfg(not(target_arch = "wasm32"))]
        { self.trace_mem = trace_mem; }
        self.ppu.reset(); self.apu.reset();
        // post-BIOS register state
        self.ppu.write_io(0, 0x80, 0xFFFF); // forced blank until the game sets DISPCNT
        for o in [0x20, 0x26, 0x30, 0x36] { self.ppu.write_io(o, 0x100, 0xFFFF); }
        self.save = Save::detect(&self.rom[..self.rom_len]);
    }

    pub fn dbg(&self) -> String { format!("ie={:x} if={:x} ime={:x} halt={} timers={:?} dispcnt={:x} dispstat={:x}", self.ie, self.iflag, self.ime, self.halt, self.timers.iter().map(|t| (t.on, t.counter)).collect::<Vec<_>>(), self.ppu.read_io(0), self.ppu.read_io(4)) }
    /// IE&IF has been non-zero for at least IRQ_DELAY cycles
    fn irq_ready(&self) -> bool { self.cond_since != u64::MAX && self.cycles >= self.cond_since + IRQ_DELAY }
    fn upd_cond(&mut self, t: u64) {
        let c = self.ie & self.iflag & 0x3FFF != 0;
        if !c { self.cond_since = u64::MAX } else if self.cond_since == u64::MAX { self.cond_since = t.saturating_sub(std::mem::take(&mut self.ppu_early)); }
    }
    /// untimed, side-effect-free 32-bit read of memory-like regions (None for IO/unmapped/open bus)
    pub fn peek32(&self, a: u32) -> Option<u32> {
        let a = a & !3;
        match a >> 24 {
            0 if a < 0x4000 => Some(ld(BIOS, a as usize, 4)),
            2 => Some(ld(&self.ewram, (a & 0x3FFFF) as usize, 4)),
            3 => Some(ld(&self.iwram, (a & 0x7FFF) as usize, 4)),
            5 => Some(ld(&self.ppu.pal, (a & 0x3FF) as usize, 4)),
            6 => { let o = a & 0x1FFFF; Some(ld(&self.ppu.vram, vram_off(o), 4)) }
            7 => Some(ld(&self.ppu.oam, (a & 0x3FF) as usize, 4)),
            8..=0xD => { let o = (a & 0x1FF_FFFF) as usize; if o + 4 <= self.rom_len { Some(ld(&self.rom, o, 4)) } else { None } }
            _ => None,
        }
    }
    pub fn irq_line(&self) -> bool { !self.woke && self.ime & 1 != 0 && self.irq_ready() }
    pub fn halted(&self) -> bool { self.halt && !self.irq_ready() }
    pub fn idle(&mut self, c: u32) { self.dma_gate(); self.cycles += c as u64; }
    pub fn take_frame(&mut self) -> bool { std::mem::take(&mut self.frame_done) }

    /// Re-phase the PPU as if the real BIOS had just finished: the reference emulator enters the ROM
    /// with the PPU 277219 cycles into the frame (line 225, cycle 19). Called once the boot stub reached the ROM.
    pub fn align_ppu(&mut self) {
        self.sync_to(self.cycles); self.next_ev = 0; // flush batched ticks so `synced` == `cycles` when the PPU is re-phased
        self.ppu.reset();
        self.ppu.write_io(0, 0x80, 0xFFFF);
        for o in [0x20, 0x26, 0x30, 0x36] { self.ppu.write_io(o, 0x100, 0xFFFF); }
        let mut left = 277224u32;
        self.ppu_pos = left;
        while left > 0 { let d = left.min(200); self.ppu.tick(d); left -= d; }
        self.iflag &= !7;
        self.frame_done = false;
        self.bios_latch = 0xE129_F000; // what the real BIOS leaves on the bus after boot
    }
    /// Total cycles already distributed to the peripherals.
    pub fn synced_cycles(&self) -> u64 { self.synced }

    // ---------------------------------------------------------------- waitstates
    fn set_waitcnt(&mut self, v: u16) {
        self.waitcnt = v & 0x7FFF;
        self.sram_w = 1 + NWAIT[(v & 3) as usize];
        self.rom_n = [1 + NWAIT[(v >> 2 & 3) as usize], 1 + NWAIT[(v >> 5 & 3) as usize], 1 + NWAIT[(v >> 8 & 3) as usize]];
        self.rom_s = [if v >> 4 & 1 != 0 { 2 } else { 3 }, if v >> 7 & 1 != 0 { 2 } else { 5 }, if v >> 10 & 1 != 0 { 2 } else { 9 }];
        let pf = v >> 14 & 1 != 0;
        if pf != self.prefetch { self.pf_next = u32::MAX; self.pf_valid = false; self.pf_cnt = 0; self.pf_flight = false; } // buffer starts empty when (de)activated
        self.prefetch = pf;
    }

    fn cost(&mut self, a: u32, sz: u32, seq: bool, write: bool) -> u64 {
        let r = a >> 24;
        #[cfg(not(target_arch = "wasm32"))]
        if self.trace_mem { self.memlog.push((self.ppu_pos + (self.cycles - self.synced) as u32, r, sz, write)); }
        (match r {
            2 => if sz == 4 { 6 } else { 3 },
            5 | 6 => if sz == 4 { 2 } else { 1 }, // ponytail: PPU/CPU VRAM+PAL contention (+1 cycle in some draw slots) not modelled
            8..=0xD => {
                let ws = ((r - 8) >> 1) as usize;
                let seq = seq && a & 0x1FFFF != 0;
                let sw = self.rom_s[ws];
                let c = if self.prefetch && !write {
                    // data (non-fetch) ROM read with prefetch on: wait for the in-flight halfword, then a normal N/S access;
                    // the buffer is kept but its producer restarts non-sequentially afterwards
                    self.pf_advance(self.cycles);
                                        let seq = seq && a == self.pf_next;
                    let first = if seq { sw } else { self.rom_n[ws] };
                    let c = first + if sz == 4 { sw } else { 0 };
                    if sz >= FLSZ { self.pf_valid = false; self.pf_cnt = 0; self.pf_flight = false; }
                    self.pf_ws = ws;
                    if self.pf_valid && sz < FLSZ { self.pf_flight = true; self.pf_done = self.cycles + c as u64 + self.rom_n[ws] as u64 + PF_DATA_RESTART; }
                    c
                } else {
                    // an idle bus cycle since the last ROM access makes the next one non-sequential
                    let seq = seq && a == self.pf_next && (self.in_dma || self.cycles == self.pf_last);
                    let first = if seq { sw } else { self.rom_n[ws] - self.after_rom_dma as u32 };
                    self.after_rom_dma = false;
                    if sz == 4 { first + sw } else { first }
                };
                self.pf_next = a.wrapping_add(sz);
                if !self.prefetch { self.pf_last = self.cycles + c as u64; }
                if self.in_dma { self.pf_valid = false; self.pf_cnt = 0; self.pf_flight = false; }
                return c as u64;
            }
            0xE | 0xF => self.sram_w,
            _ => 1,
        }) as u64
    }


    // ---------------------------------------------------------------- timed CPU/DMA API
    pub fn read8(&mut self, a: u32, seq: bool) -> u32 { self.dma_gate(); self.cycles += self.cost(a, 1, seq, false); self.rd(a, 1) }
    pub fn read16(&mut self, a: u32, seq: bool) -> u32 { self.dma_gate(); self.cycles += self.cost(a, 2, seq, false); self.rd(a & !1, 2) }
    pub fn read32(&mut self, a: u32, seq: bool) -> u32 { self.dma_gate(); self.cycles += self.cost(a, 4, seq, false); self.rd(a & !3, 4) }
    pub fn write8(&mut self, a: u32, v: u32, seq: bool) { self.dma_gate(); self.cycles += self.cost(a, 1, seq, true); self.wr(a, v, 1) }
    pub fn write16(&mut self, a: u32, v: u32, seq: bool) { self.dma_gate(); self.cycles += self.cost(a, 2, seq, true); self.wr(if a >> 24 >= 0xE { a } else { a & !1 }, v, 2) }
    pub fn write32(&mut self, a: u32, v: u32, seq: bool) { self.dma_gate(); self.cycles += self.cost(a, 4, seq, true); self.wr(if a >> 24 >= 0xE { a } else { a & !3 }, v, 4) }

    // ---------------------------------------------------------------- untimed memory
    fn open(&self, a: u32, sz: u32) -> u32 {
        let v = if self.in_dma { self.dma_latch } else { self.open_bus };
        match sz { 1 => (v >> ((a & 3) * 8)) & 0xFF, 2 => (v >> ((a & 2) * 8)) & 0xFFFF, _ => v }
    }

    /// EEPROM answers the whole 0x0D region on carts <= 16 MiB, only 0x0DFFFF00.. on bigger ones
    fn is_eeprom(&self, a: u32) -> bool {
        a >> 24 == 0xD && self.save.kind == save::Kind::Eeprom && (self.rom_len <= 16 << 20 || a & 0x01FF_FF00 == 0x01FF_FF00)
    }
    fn rd(&mut self, a: u32, sz: u32) -> u32 {
        match a >> 24 {
            0 => {
                if a >= 0x4000 { return self.open(a, sz); }
                if self.fetch_pc < 0x4000 { self.bios_latch = ld(BIOS, (a & !3) as usize, 4); }
                let v = self.bios_latch >> ((a & 3) * 8);
                if sz == 1 { v & 0xFF } else if sz == 2 { v & 0xFFFF } else { v }
            }
            2 => ld(&self.ewram, (a & 0x3FFFF) as usize, sz),
            3 => ld(&self.iwram, (a & 0x7FFF) as usize, sz),
            4 => {
                let off = a & 0xFFFFFF;
                if off >= 0x400 { return self.open(a, sz); }
                self.sync_to(self.mark);
                let ob = if self.in_dma { self.dma_latch } else { self.open_bus };
                let mut r = |o: u32, sh: u32| self.io_read16(o).map_or((ob >> sh) & 0xFFFF, |v| v as u32);
                match sz {
                    1 => (r(off & !1, (off & 2) * 8) >> ((off & 1) * 8)) & 0xFF,
                    2 => r(off, (off & 2) * 8),
                    _ => r(off, 0) | r(off + 2, 16) << 16,
                }
            }
            5 => ld(&self.ppu.pal, (a & 0x3FF) as usize, sz),
            6 => {
                let o = a & 0x1FFFF;
                // observed on hardware: in bitmap modes 0x18000-0x1BFFF reads as 0 (writes dropped below)
                if (0x18000..0x1C000).contains(&o) && self.ppu.read_io(0) & 7 >= 3 { return 0; }
                ld(&self.ppu.vram, vram_off(o), sz)
            }
            7 => ld(&self.ppu.oam, (a & 0x3FF) as usize, sz),
            8..=0xD => {
                if self.is_eeprom(a) {
                    return self.save.eeprom_read() as u32;
                }
                let o = (a & 0x1FF_FFFF) as usize;
                if o + sz as usize <= self.rom_len { ld(&self.rom, o, sz) } else {
                    // beyond the cartridge: address bus echoes halfword index
                    let h = |o: usize| ((o >> 1) & 0xFFFF) as u32;
                    match sz { 1 => h(o) >> ((o & 1) * 8) & 0xFF, 2 => h(o), _ => h(o) | h(o + 2) << 16 }
                }
            }
            0xE | 0xF => {
                let b = self.save.read(a & 0xFFFF) as u32;
                match sz { 1 => b, 2 => b * 0x0101, _ => b * 0x0101_0101 }
            }
            _ => self.open(a, sz),
        }
    }

    fn wr(&mut self, a: u32, v: u32, sz: u32) {
        // PPU draws progressively: bring it up to the instruction boundary before VRAM/PAL/OAM change
        if (5..=7).contains(&(a >> 24)) { self.sync_to(self.mark); }
        match a >> 24 {
            2 => st(&mut self.ewram, (a & 0x3FFFF) as usize, sz, v),
            3 => st(&mut self.iwram, (a & 0x7FFF) as usize, sz, v),
            4 => {
                let off = a & 0xFFFFFF;
                if off >= 0x400 { return; }
                self.sync_to(self.mark); self.next_ev = 0;
                match sz {
                    1 => { let s = (off & 1) * 8; self.io_write16(off & !1, (v & 0xFF) << s, 0xFF << s); }
                    2 => self.io_write16(off, v, 0xFFFF),
                    _ => {
                        if off == 0xA0 || off == 0xA4 { self.apu.fifo_write(((off - 0xA0) >> 2) as usize, v); }
                        else { self.io_write16(off, v & 0xFFFF, 0xFFFF); self.io_write16(off + 2, v >> 16, 0xFFFF); }
                    }
                }
            }
            5 => {
                let i = (a & 0x3FF) as usize;
                if sz == 1 { self.ppu.pal[i & !1] = v as u8; self.ppu.pal[i | 1] = v as u8; } else { st(&mut self.ppu.pal, i, sz, v); }
            }
            6 => {
                let o = a & 0x1FFFF;
                let i = vram_off(o);
                if (0x18000..0x1C000).contains(&o) && self.ppu.read_io(0) & 7 >= 3 { return; }
                if sz == 1 {
                    let obj = if self.ppu.read_io(0) & 7 >= 3 { 0x14000 } else { 0x10000 };
                    if i < obj { self.ppu.vram[i & !1] = v as u8; self.ppu.vram[i | 1] = v as u8; }
                } else { st(&mut self.ppu.vram, i, sz, v); }
            }
            7 => if sz != 1 { st(&mut self.ppu.oam, (a & 0x3FF) as usize, sz, v) },
            0xD if self.is_eeprom(a) => self.save.eeprom_write(v as u8),
            0xE | 0xF => self.save.write(a & 0xFFFF, (v >> ((a & (sz - 1)) * 8)) as u8),
            _ => {}
        }
    }

    // ---------------------------------------------------------------- keypad
    pub fn set_keys(&mut self, k: u32) { self.keys = k & 0x3FF; self.check_keypad(); }
    fn check_keypad(&mut self) {
        let cnt = self.io[0x99];
        let mask = (cnt & 0x3FF) as u32;
        let hit = if cnt & 0x8000 != 0 { mask != 0 && self.keys & mask == mask } else { self.keys & mask != 0 };
        let cond = cnt & 0x4000 != 0 && hit;
        if cond && !self.key_cond { self.iflag |= 0x1000; }
        self.key_cond = cond;
    }

    // ---------------------------------------------------------------- machine stepping
    /// Distribute all cycles consumed since the last call to PPU/APU/timers (and run triggered DMAs).
    pub fn sync(&mut self) { self.woke = false; self.mark = self.cycles; self.eq = self.cycles;
        if self.cycles >= self.next_ev { self.sync_to(self.cycles); self.next_ev = self.synced + self.next_chunk() as u64; } self.upd_cond(self.cycles); if self.halt && self.dma_pending != 0 { self.dma_run_pending(); }
        if self.halt && self.irq_ready() { self.halt = false; self.woke = true; self.wake_pen = true; } }

    /// Distribute cycles up to `target` only (register reads observe the state slightly before the access ends).
    fn sync_to(&mut self, target: u64) {
        self.eq = self.eq.max(target); // eq = what `synced` would be without batching (keeps the DMA preempt cadence)
        while self.synced < target {
            let d = (target - self.synced).min(self.next_chunk() as u64) as u32;
            self.synced += d as u64;
            self.tick(d);
            self.upd_cond(self.synced); self.ppu_early = 0; // backdating applies only to an edge raised by a PPU flag in this chunk
        }
    }

    /// cycles until the next timer overflow / line boundary (max 256)
    fn next_chunk(&self) -> u32 {
        let lp = self.ppu_pos % 1232;
        self.next_timer_event().min(if lp < 1006 { 1006 - lp } else { 1232 - lp })
    }

    /// Fast-forward while halted: burn up to `max` cycles until the next event.
    pub fn halt_skip(&mut self, max: u64) {
        let mut d = (max.min(256) as u32).min(self.next_timer_event()).max(1);
        if self.cond_since != u64::MAX && self.cond_since + IRQ_DELAY > self.cycles { d = d.min((self.cond_since + IRQ_DELAY - self.cycles) as u32); }
        self.cycles += d as u64;
        self.sync();
    }


    fn tick(&mut self, d: u32) {
        let ev = self.ppu.tick(d);
        self.ppu_pos = (self.ppu_pos + d) % 280_896;
        if ev.irq & 7 != 0 { self.ppu_early = PPU_IRQ_EARLY; }
        self.iflag |= ev.irq & 7;
        self.apu.tick(d);
        self.timers_tick(d);
        if ev.vblank_start { self.frame_done = true; self.dma_trigger(1); }
        if ev.hblank_start {
            let vc = self.ppu.read_io(6) & 0xFF;
            if vc < 160 { self.dma_trigger(2); }
            if (2..162).contains(&vc) { self.dma_trigger(3); if vc == 161 { self.io[0xDE >> 1] &= !0x8000; } }
        }
        self.check_keypad();
    }


    // ---------------------------------------------------------------- DMA




}

#[cfg(test)]
mod tests {
    use super::*;
    fn bus() -> Bus { let mut b = Bus::new(); b.rom_len = 0x100; b.reset(); b }

    #[test]
    fn mirrors_and_widths() {
        let mut b = bus();
        b.write32(0x0300_0000, 0xAABBCCDD, false);
        assert_eq!(b.read32(0x0300_8000, false), 0xAABBCCDD);
        assert_eq!(b.read8(0x0300_7FE1, false), 0); // untouched
        b.write16(0x0200_0002, 0x1234, false);
        assert_eq!(b.read16(0x0204_0002, false), 0x1234);
        b.write8(0x0500_0001, 0x7F, false);
        assert_eq!(b.read16(0x0500_0000, false), 0x7F7F);
    }

    #[test]
    fn waitstates() {
        let mut b = bus();
        let c = b.cycles; b.read32(0x0200_0000, false); assert_eq!(b.cycles - c, 6);
        let c = b.cycles; b.read16(0x0800_0000, false); assert_eq!(b.cycles - c, 5);
        let c = b.cycles; b.read16(0x0800_0002, true); assert_eq!(b.cycles - c, 3);
        let c = b.cycles; b.read32(0x0800_0004, true); assert_eq!(b.cycles - c, 6);
    }

    #[test]
    fn timer_irq_and_cascade() {
        let mut b = bus();
        b.write32(0x0400_0100, 0xFFF0 | 0xC1 << 16, false); // reload FFF0, irq, on, /64? (ctrl 0xC1)
        b.write32(0x0400_0104, 0xFFFE | 0x84 << 16, false); // cascade, on
        b.idle(64 * 16 + 1); b.sync();
        assert_eq!(b.iflag & 8, 8);
        assert_eq!(b.timers[1].counter, 0xFFFF);
    }

    #[test]
    fn dma_immediate_and_ifack() {
        let mut b = bus();
        for i in 0..4u32 { b.write32(0x0200_0000 + i * 4, 0x01010101 * (i + 1), false); }
        b.write32(0x0400_00D4, 0x0200_0000, false);
        b.write32(0x0400_00D8, 0x0300_0000, false);
        b.write32(0x0400_00DC, 4 | (0x8000 | 0x4000 | 0x400) << 16, false);
        b.idle(2); // DMA starts ~2 cycles after the enabling write
        assert_eq!(b.read32(0x0300_000C, false), 0x04040404);
        assert_eq!(b.iflag & 0x800, 0x800);
        b.write16(0x0400_0202, 0x800, false);
        assert_eq!(b.iflag, 0);
        assert_eq!(b.io[0xDE >> 1] & 0x8000, 0);
    }

    #[test]
    fn flash_and_eeprom() {
        let mut s = Save::new(save::Kind::Flash64);
        for (o, v) in [(0x5555, 0xAA), (0x2AAA, 0x55), (0x5555, 0xA0), (0x123, 0x42)] { s.write(o, v); }
        assert_eq!(s.read(0x123), 0x42);
        let mut e = Save::new(save::Kind::Eeprom);
        e.eeprom_hint(73);
        let mut w = vec![1, 0, 0, 0, 0, 0, 1, 1];
        for i in 0..64 { w.push(((0xA5u8 >> (7 - i % 8)) & 1) as u8); }
        w.push(0);
        for b in w { e.eeprom_write(b); }
        for b in [1, 1, 0, 0, 0, 0, 1, 1, 0] { e.eeprom_write(b); }
        let bits: Vec<u16> = (0..68).map(|_| e.eeprom_read()).collect();
        assert_eq!(&bits[4..12], &[1, 0, 1, 0, 0, 1, 0, 1]);
    }
}
