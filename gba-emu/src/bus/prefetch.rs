//! Explicit game-pak prefetch buffer model (see DECISIONS.md).
use super::*;

impl Bus {
    pub(super) fn pf_advance(&mut self, now: u64) {
        while self.pf_flight && self.pf_done <= now {
            self.pf_cnt += 1; self.pf_flight = false;
            if self.pf_cnt < 8 { self.pf_flight = true; self.pf_done += self.rom_s[self.pf_ws] as u64; }
        }
    }

    /// Cost of an opcode fetch of `sz` bytes from ROM with the prefetch buffer enabled.
    pub(super) fn pf_fetch(&mut self, a: u32, sz: u32) -> u64 {
        let ws = ((a >> 24) - 8 >> 1) as usize;
        let (n, sw) = (self.rom_n[ws], self.rom_s[ws]);
        let now = self.cycles;
        #[cfg(not(target_arch = "wasm32"))]
        if self.trace_mem { self.memlog.push((self.ppu_pos + (now - self.synced) as u32, a >> 24, sz, false)); }
        self.pf_advance(now);
        let need = sz / 2;
        if self.pf_valid && a == self.pf_head && self.pf_ws == ws && a & 0x1FFFF != 0 {
            let mut t = now;
            for _ in 0..need {
                self.pf_advance(t);
                debug_assert!(self.pf_cnt <= 8);
                if self.pf_cnt > 0 {
                    if self.pf_cnt == 8 && !self.pf_flight { self.pf_flight = true; self.pf_done = t + sw as u64; }
                    self.pf_cnt -= 1; t += 1;
                } else {
                    t = self.pf_done.max(t + 1);
                    self.pf_advance(t);
                    debug_assert!(self.pf_cnt > 0, "in-flight fetch must deliver a halfword");
                    self.pf_cnt -= 1;
                }
            }
            self.pf_head = a.wrapping_add(sz);
            t - now
        } else {
            let rem = self.pf_done.saturating_sub(now); let wait = if self.pf_flight && (rem < sw as u64) { rem } else { 0 };
            let c = wait + (n + (need - 1) * sw) as u64;
            self.pf_valid = true; self.pf_head = a.wrapping_add(sz); self.pf_cnt = 0; self.pf_ws = ws;
            self.pf_flight = true; self.pf_done = now + c + sw as u64;
            c
        }
    }

    pub fn fetch16(&mut self, a: u32, seq: bool) -> u32 {
        if self.prefetch && !self.in_dma && (8..=0xD).contains(&(a >> 24)) { self.dma_gate(); self.cycles += self.pf_fetch(a, 2); self.rd(a & !1, 2) } else { self.read16(a, seq) }
    }

    pub fn fetch32(&mut self, a: u32, seq: bool) -> u32 {
        if self.prefetch && !self.in_dma && (8..=0xD).contains(&(a >> 24)) { self.dma_gate(); self.cycles += self.pf_fetch(a, 4); self.rd(a & !3, 4) } else { self.read32(a, seq) }
    }
}
