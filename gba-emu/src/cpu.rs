//! ARM7TDMI core: ARM + THUMB, banked registers, exceptions.
//!
//! Model: `r[15]` always holds the pipeline-visible PC (instr + 8 in ARM, instr + 4 in
//! THUMB). Each instruction pays its own fetch; a pipeline flush pays the two refill
//! fetches (N + S) so branches cost 2S+1N in total like on hardware.
use crate::bus::Bus;

/// Memory/timing interface the core needs (implemented for `Bus`).
pub trait CpuBus {
    fn read8(&mut self, a: u32, seq: bool) -> u32;
    fn read16(&mut self, a: u32, seq: bool) -> u32;
    fn read32(&mut self, a: u32, seq: bool) -> u32;
    fn write8(&mut self, a: u32, v: u32, seq: bool);
    fn write16(&mut self, a: u32, v: u32, seq: bool);
    fn write32(&mut self, a: u32, v: u32, seq: bool);
    fn idle(&mut self, c: u32);
    fn irq_line(&self) -> bool;
    fn halted(&self) -> bool;
    /// Tell the bus the address/opcode of the instruction being executed (BIOS protection, open bus).
    fn note_fetch(&mut self, _pc: u32, _op: u32) {}
    fn note_pc(&mut self, _pc: u32) {}
    /// Untimed peek of the word at `a` (ROM only); used to set the open-bus value.
    fn peek32(&mut self, _a: u32) -> Option<u32> { None }
    fn set_open(&mut self, _v: u32) {}
    fn fetch16(&mut self, a: u32, seq: bool) -> u32 { self.read16(a, seq) }
    fn fetch32(&mut self, a: u32, seq: bool) -> u32 { self.read32(a, seq) }
}

impl CpuBus for Bus {
    #[inline] fn read8(&mut self, a: u32, s: bool) -> u32 { Bus::read8(self, a, s) }
    #[inline] fn read16(&mut self, a: u32, s: bool) -> u32 { Bus::read16(self, a, s) }
    #[inline] fn read32(&mut self, a: u32, s: bool) -> u32 { Bus::read32(self, a, s) }
    #[inline] fn write8(&mut self, a: u32, v: u32, s: bool) { Bus::write8(self, a, v, s) }
    #[inline] fn write16(&mut self, a: u32, v: u32, s: bool) { Bus::write16(self, a, v, s) }
    #[inline] fn write32(&mut self, a: u32, v: u32, s: bool) { Bus::write32(self, a, v, s) }
    #[inline] fn idle(&mut self, c: u32) { Bus::idle(self, c) }
    #[inline] fn irq_line(&self) -> bool { Bus::irq_line(self) }
    #[inline] fn halted(&self) -> bool { Bus::halted(self) }
    #[inline] fn note_pc(&mut self, pc: u32) { self.fetch_pc = pc; }
    #[inline] fn fetch16(&mut self, a: u32, s: bool) -> u32 { Bus::fetch16(self, a, s) }
    #[inline] fn fetch32(&mut self, a: u32, s: bool) -> u32 { Bus::fetch32(self, a, s) }
    #[inline] fn set_open(&mut self, v: u32) { self.open_bus = v; }
    #[inline] fn peek32(&mut self, a: u32) -> Option<u32> { Bus::peek32(self, a) }
    #[inline] fn note_fetch(&mut self, pc: u32, op: u32) { self.fetch_pc = pc; self.open_bus = op; }
}

const N: u32 = 1 << 31;
const Z: u32 = 1 << 30;
const C: u32 = 1 << 29;
const V: u32 = 1 << 28;
const FT: u32 = 1 << 5;
const FI: u32 = 1 << 7;

const M_USR: u32 = 0x10;
const M_FIQ: u32 = 0x11;
const M_IRQ: u32 = 0x12;
const M_SVC: u32 = 0x13;
const M_UND: u32 = 0x1B;

#[inline]
fn bank(mode: u32) -> usize {
    match mode & 0x1F {
        0x11 => 1,
        0x12 => 2,
        0x13 => 3,
        0x17 => 4,
        0x1B => 5,
        _ => 0,
    }
}

/// Condition table: bit `flags(NZCV)` of COND[c] says whether condition c passes.
const COND: [u16; 16] = {
    let mut t = [0u16; 16];
    let mut c = 0;
    while c < 16 {
        let mut f = 0;
        while f < 16 {
            let n = f & 8 != 0;
            let z = f & 4 != 0;
            let cc = f & 2 != 0;
            let v = f & 1 != 0;
            let ok = match c {
                0 => z,
                1 => !z,
                2 => cc,
                3 => !cc,
                4 => n,
                5 => !n,
                6 => v,
                7 => !v,
                8 => cc && !z,
                9 => !cc || z,
                10 => n == v,
                11 => n != v,
                12 => !z && n == v,
                13 => z || n != v,
                14 => true,
                _ => false,
            };
            if ok {
                t[c] |= 1 << f;
            }
            f += 1;
        }
        c += 1;
    }
    t
};

#[derive(Clone)]
pub struct Cpu {
    pub r: [u32; 16],
    pub cpsr: u32,
    spsr: [u32; 6],
    bank_sp: [u32; 6],
    bank_lr: [u32; 6],
    fiq_r: [u32; 5],
    usr_r: [u32; 5],
    flushed: bool,
    cold: bool,
    step_thumb: bool,
    wasted: bool,
    pipe: [u32; 2],
}

impl Cpu {
    pub fn new() -> Self {
        let mut c = Cpu {
            r: [0; 16],
            cpsr: 0xD3,
            spsr: [0; 6],
            bank_sp: [0; 6],
            bank_lr: [0; 6],
            fiq_r: [0; 5],
            usr_r: [0; 5],
            flushed: false,
            cold: true,
            step_thumb: false,
            wasted: false,
            pipe: [0; 2],
        };
        c.r[15] = 8;
        c
    }
    pub fn reset(&mut self) {
        *self = Cpu::new();
    }

    /// Run one instruction (or take an IRQ).
    pub fn step(&mut self, bus: &mut Bus) {
        self.step_g(bus)
    }

    // ---------------------------------------------------------------- state helpers
    #[inline]
    fn thumb(&self) -> bool {
        self.cpsr & FT != 0
    }

    fn switch_mode(&mut self, new: u32) {
        let old = self.cpsr & 0x1F;
        let (ob, nb) = (bank(old), bank(new));
        if ob == nb {
            return;
        }
        self.bank_sp[ob] = self.r[13];
        self.bank_lr[ob] = self.r[14];
        if old == M_FIQ {
            self.fiq_r.copy_from_slice(&self.r[8..13]);
            self.r[8..13].copy_from_slice(&self.usr_r);
        }
        if new & 0x1F == M_FIQ {
            self.usr_r.copy_from_slice(&self.r[8..13]);
            self.r[8..13].copy_from_slice(&self.fiq_r);
        }
        self.r[13] = self.bank_sp[nb];
        self.r[14] = self.bank_lr[nb];
    }

    fn set_cpsr(&mut self, v: u32) {
        self.switch_mode(v);
        self.cpsr = v;
    }

    fn restore_spsr(&mut self) {
        let b = bank(self.cpsr);
        if b != 0 {
            let s = self.spsr[b];
            self.set_cpsr(s);
        }
    }

    fn user_reg(&self, i: usize) -> u32 {
        let m = self.cpsr & 0x1F;
        if bank(m) == 0 { return self.r[i]; }
        match i {
            8..=12 if m == M_FIQ => self.usr_r[i - 8],
            13 => self.bank_sp[0],
            14 => self.bank_lr[0],
            _ => self.r[i],
        }
    }
    fn set_user_reg(&mut self, i: usize, v: u32) {
        let m = self.cpsr & 0x1F;
        if bank(m) == 0 { self.r[i] = v; return; }
        match i {
            8..=12 if m == M_FIQ => self.usr_r[i - 8] = v,
            13 => self.bank_sp[0] = v,
            14 => self.bank_lr[0] = v,
            _ => self.r[i] = v,
        }
    }

    #[inline]
    fn cond_ok(&self, c: u32) -> bool {
        COND[c as usize] >> (self.cpsr >> 28) & 1 != 0
    }

    #[inline]
    fn set_nz(&mut self, v: u32) {
        self.cpsr = (self.cpsr & !(N | Z)) | (v & N) | if v == 0 { Z } else { 0 };
    }
    #[inline]
    fn set_c(&mut self, c: bool) {
        self.cpsr = (self.cpsr & !C) | if c { C } else { 0 };
    }

    /// a + b + cin with optional flag update.
    #[inline]
    fn adc(&mut self, a: u32, b: u32, cin: u32, s: bool) -> u32 {
        let r64 = a as u64 + b as u64 + cin as u64;
        let r = r64 as u32;
        if s {
            let v = (!(a ^ b) & (a ^ r)) >> 31;
            self.cpsr = (self.cpsr & !(N | Z | C | V))
                | (r & N)
                | if r == 0 { Z } else { 0 }
                | if r64 >> 32 != 0 { C } else { 0 }
                | (v << 28);
        }
        r
    }

    /// Refill the two-stage pipeline from `v` (N + S fetch).
    fn refill<B: CpuBus>(&mut self, b: &mut B, v: u32) {
        if self.thumb() {
            let pc = v & !1;
            self.r[15] = pc.wrapping_add(4);
            b.note_pc(pc);
            self.pipe[0] = b.fetch16(pc, false) & 0xFFFF;
            self.pipe[1] = b.fetch16(pc.wrapping_add(2), true) & 0xFFFF;
        } else {
            let pc = v & !3;
            self.r[15] = pc.wrapping_add(8);
            b.note_pc(pc);
            self.pipe[0] = b.fetch32(pc, false);
            self.pipe[1] = b.fetch32(pc.wrapping_add(4), true);
        }
        self.flushed = true;
    }

    /// Branch: the fetch already in flight (at r15) is wasted, then the pipeline refills.
    fn set_pc<B: CpuBus>(&mut self, b: &mut B, v: u32) {
        // ROM -> non-ROM branch: the wasted ROM fetch overlaps on the cartridge bus (1 cycle)
        if !self.wasted && (self.r[15] >> 24).wrapping_sub(8) < 6 && (v >> 24).wrapping_sub(8) >= 6 {
            self.wasted = true;
            b.idle(1);
        }
        // non-ROM -> ROM branch: 2 extra cycles (cartridge bus startup) measured vs the reference
        if (self.r[15] >> 24).wrapping_sub(8) >= 6 && (v >> 24).wrapping_sub(8) < 6 { b.idle(2); }
        self.waste(b);
        self.refill(b, v);
    }

    /// The wasted in-flight fetch; loads into PC issue it before their data cycles.
    fn waste<B: CpuBus>(&mut self, b: &mut B) {
        if !self.wasted {
            self.wasted = true;
            if self.thumb() { b.fetch16(self.r[15], true); } else { b.fetch32(self.r[15], true); }
        }
    }

    /// Like `waste`, but a ROM -> non-ROM load into PC overlaps the wasted fetch (1 cycle).
    fn waste_to<B: CpuBus>(&mut self, b: &mut B, dest: Option<u32>) {
        if !self.wasted {
            if let Some(d) = dest {
                if (self.r[15] >> 24).wrapping_sub(8) < 6 && (d >> 24).wrapping_sub(8) >= 6 {
                    self.wasted = true;
                    b.idle(1);
                    return;
                }
            }
            self.waste(b);
        }
    }

    fn exception<B: CpuBus>(&mut self, b: &mut B, mode: u32, vector: u32, lr: u32) {
        let old = self.cpsr;
        self.spsr[bank(mode)] = old;
        self.set_cpsr((old & !0x3F) | mode | FI);
        self.r[14] = lr;
        self.cpsr &= !FT;
        self.wasted = true; b.idle(1);
        self.set_pc(b, vector);
    }

    // ---------------------------------------------------------------- step
    pub fn step_g<B: CpuBus>(&mut self, b: &mut B) {
        if b.halted() {
            b.idle(1);
            return;
        }
        if self.cold {
            self.cold = false;
            let pc = self.r[15].wrapping_sub(8);
            b.note_pc(pc);
            self.refill(b, pc);
        }
        self.flushed = false;
        self.wasted = false;
        self.step_thumb = self.cpsr & FT != 0;
        // The reference runs one more instruction after the IRQ line is seen, then enters (3 cycles).
        let take = self.cpsr & FI == 0 && b.irq_line();
        let op = self.pipe[0];
        self.pipe[0] = self.pipe[1];
        // The opcode fetch for the pipeline happens after the instruction's own bus cycles.
        if self.step_thumb {
            b.note_pc(self.r[15].wrapping_sub(4));
            let r = self.r[15];
            if let Some(w) = b.peek32(r & !3) {
                let h4 = if r & 2 != 0 { w >> 16 } else { w & 0xFFFF };
                let v = if r >> 24 == 3 {
                    // IWRAM code: the two halves come from different prefetched words
                    let h2 = b.peek32((r - 2) & !3).map_or(h4, |x| if (r - 2) & 2 != 0 { x >> 16 } else { x & 0xFFFF });
                    if r & 2 == 0 { h2 | h4 << 16 } else { h4 | h2 << 16 }
                } else { h4 | h4 << 16 };
                b.set_open(v);
            }
            self.exec_thumb(b, op);
            if !self.flushed {
                let nf = b.fetch16(self.r[15], true) & 0xFFFF;
                self.pipe[1] = nf;
                b.note_fetch(self.r[15], nf | nf << 16);
                self.r[15] = self.r[15].wrapping_add(2);
            }
        } else {
            b.note_pc(self.r[15].wrapping_sub(8));
            if let Some(w) = b.peek32(self.r[15]) { b.set_open(w); }
            if self.cond_ok(op >> 28) {
                self.exec_arm(b, op);
            }
            if !self.flushed {
                let nf = b.fetch32(self.r[15], true);
                self.pipe[1] = nf;
                b.note_fetch(self.r[15], nf);
                self.r[15] = self.r[15].wrapping_add(4);
            }
        }
        if take && self.cpsr & FI == 0 && b.irq_line() {
            // next instruction = r15-8 (ARM) / r15-4 (THUMB); LR = next + 4
            let lr = if self.thumb() { self.r[15] } else { self.r[15] - 4 };
            self.step_thumb = self.thumb();
            self.exception(b, M_IRQ, 0x18, lr);
        }
    }

    // ---------------------------------------------------------------- memory helpers
    #[inline]
    fn ld32<B: CpuBus>(&mut self, b: &mut B, a: u32, seq: bool) -> u32 {
        b.read32(a, seq).rotate_right((a & 3) * 8)
    }
    #[inline]
    fn ld16<B: CpuBus>(&mut self, b: &mut B, a: u32, seq: bool) -> u32 {
        (b.read16(a, seq) & 0xFFFF).rotate_right((a & 1) * 8)
    }
    #[inline]
    fn ld16s<B: CpuBus>(&mut self, b: &mut B, a: u32, seq: bool) -> u32 {
        if a & 1 != 0 {
            b.read8(a, seq) as u8 as i8 as u32
        } else {
            b.read16(a, seq) as u16 as i16 as u32
        }
    }
    #[inline]
    fn ld8s<B: CpuBus>(&mut self, b: &mut B, a: u32, seq: bool) -> u32 {
        b.read8(a, seq) as u8 as i8 as u32
    }

}

mod alu;
mod arm;
mod thumb;
#[cfg(test)]
mod tests;
use alu::*;
