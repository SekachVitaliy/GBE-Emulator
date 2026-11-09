//! DMA channels: triggers, start gating, preemption and the transfer loop.
use super::*;

impl Bus {
    pub(super) fn dma_ctrl_write(&mut self, n: usize, old: u16, new: u16) {
        if new & 0x8000 == 0 { return; }
        if old & 0x8000 != 0 {
            // channel rewrites its own control while running: restart once the current transfer ends
            let _ = n; // ponytail: own-control rewrite while running is ignored (reference does not restart)
            return;
        }
        self.dma_relatch(n);
        if new >> 12 & 3 == 0 { self.dma_pending |= 1 << n; self.dma_ready[n] = self.cycles + 2; self.dma_lat[n] = 1; }
    }

    pub(super) fn dma_relatch(&mut self, n: usize) {
        let r = |o: usize| self.io[(0xB0 + 12 * n + o) >> 1] as u32;
        let (sad, dad, cnt) = (r(0) | r(2) << 16, r(4) | r(6) << 16, r(8));
        let d = &mut self.dma[n];
        d.src = sad & if n == 0 { 0x07FF_FFFF } else { 0x0FFF_FFFF };
        d.dst = dad & if n < 3 { 0x07FF_FFFF } else { 0x0FFF_FFFF };
        d.left = Self::dma_count(n, cnt);
    }

    pub(super) fn dma_count(n: usize, cnt: u32) -> u32 { if cnt == 0 { if n == 3 { 0x10000 } else { 0x4000 } } else { cnt & if n == 3 { 0xFFFF } else { 0x3FFF } } }

    pub(super) fn dma_trigger(&mut self, timing: u16) {
        for n in 0..4 {
            let c = self.io[(0xBA + 12 * n) >> 1];
            if c & 0x8000 != 0 && c >> 12 & 3 == timing && (timing != 3 || n == 3) { self.dma_pending |= 1 << n; self.dma_lat[n] = 2; self.dma_ready[n] = self.synced + EV_X; }
        }
    }

    pub(super) fn dma_fifo(&mut self, ch: usize) {
        for n in 1..3 {
            let c = self.io[(0xBA + 12 * n) >> 1];
            if c & 0x8000 != 0 && c >> 12 & 3 == 3 && self.dma[n].dst & !3 == 0x0400_00A0 + 4 * ch as u32 { self.dma_pending |= 1 << n; self.fifo_mask |= 1 << n; self.dma_lat[n] = 2; self.dma_ready[n] = self.synced + EV_X; }
        }
    }

    /// Called before every CPU/DMA bus access: applies the post-halt penalty, ticks peripherals if an event is due
    /// (so a DMA request raised mid-instruction is seen), and lets a pending DMA take the bus once its start time passed.
    #[inline] pub(super) fn dma_gate(&mut self) {
        if self.wake_pen { self.wake_pen = false; self.cycles += 2; }
        if self.cycles >= self.next_ev && self.next_ev != 0 { self.sync_to(self.cycles); self.next_ev = self.synced + self.next_chunk() as u64; }
        if self.dma_pending != 0 && !self.in_dma {
            // the highest-priority pending channel decides when the DMA gets the bus
            let n = self.dma_pending.trailing_zeros() as usize;
            if self.cycles >= self.dma_ready[n] { self.cycles += self.dma_lat[n]; self.dma_run_pending(); }
        }
    }

    /// Run requested channels, lowest number (highest priority) first; channels requested meanwhile are picked up too.
    pub(super) fn dma_run_pending(&mut self) {
        if self.in_dma { return; }
        while self.dma_pending != 0 {
            let n = self.dma_pending.trailing_zeros() as usize;
            self.dma_pending &= !(1 << n);
            let fifo = self.fifo_mask >> n & 1 != 0;
            self.fifo_mask &= !(1 << n);
            if self.io[(0xBA + 12 * n) >> 1] & 0x8000 != 0 { self.run_dma(n, fifo); }
        }
    }

    pub(super) fn dma_preempt(&mut self, n: usize) {
        let (latch, cur) = (self.dma_latch, self.cur_dma);
        while self.dma_pending & ((1u8 << n) - 1) != 0 {
            let m = self.dma_pending.trailing_zeros() as usize;
            self.dma_pending &= !(1 << m);
            let fifo = self.fifo_mask >> m & 1 != 0;
            self.fifo_mask &= !(1 << m);
            if self.io[(0xBA + 12 * m) >> 1] & 0x8000 != 0 { self.run_dma(m, fifo); }
        }
        self.in_dma = true; self.dma_latch = latch; self.cur_dma = cur;
    }

    pub(super) fn run_dma(&mut self, n: usize, fifo: bool) {
        self.in_dma = true; self.cur_dma = n;
        self.dma_latch = self.latches[n];
        let ci = (0xBA + 12 * n) >> 1;
        let ctrl = self.io[ci];
        let word = fifo || ctrl & 0x400 != 0;
        let sz = if word { 4 } else { 2 };
        let (dc, sc) = ((ctrl >> 5 & 3) as u32, (ctrl >> 7 & 3) as u32);
        let mut d = self.dma[n];
        let count = if fifo { 4 } else { d.left };
        if self.is_eeprom(d.dst) { self.save.eeprom_hint(count); }
        for i in 0..count {
            // higher-priority channels (e.g. sound FIFO refills) preempt between units
            if self.cycles - self.eq >= 32 { self.sync_to(self.cycles); }
            if self.dma_pending & ((1u8 << n) - 1) != 0 { self.dma_preempt(n); }
            let seq = i > 0;
            d.src &= !(sz - 1); d.dst &= !(sz - 1);
            let v = if d.src < 0x0200_0000 || d.src >= 0x1000_0000 { self.dma_latch }
                else if word { self.read32(d.src, seq) } else { self.read16(d.src, seq) };
            if d.src >= 0x0200_0000 && d.src < 0x1000_0000 { self.dma_latch = if word { v } else { v | v << 16 }; }
            if word { self.write32(d.dst, v, seq) } else { self.write16(d.dst, v, seq) }
            // observed: Game Pak sources ignore 'fixed' and keep incrementing
            d.src = match if (sc == 2 || sc == 1) && (0x0800_0000..0x0E00_0000).contains(&d.src) { 0 } else { sc } { 1 => d.src.wrapping_sub(sz), 2 => d.src, _ => d.src.wrapping_add(sz) };
            if !fifo { d.dst = match dc { 1 => d.dst.wrapping_sub(sz), 2 => d.dst, _ => d.dst.wrapping_add(sz) }; }
        }
        let timing = ctrl >> 12 & 3;
        if ctrl & 0x4000 != 0 && !fifo { self.iflag |= 0x100 << n; }
        if fifo || (ctrl & 0x200 != 0 && timing != 0) {
            if !fifo {
                let r = |o: usize| self.io[(0xB0 + 12 * n + o) >> 1] as u32;
                d.left = Self::dma_count(n, r(8));
                if dc == 3 { d.dst = (r(4) | r(6) << 16) & if n < 3 { 0x07FF_FFFF } else { 0x0FFF_FFFF }; }
            }
        } else {
            self.io[ci] &= !0x8000;
        }
        self.dma[n] = d;
        self.latches[n] = self.dma_latch;
        self.after_rom_dma = d.src >= 0x0800_0000 && d.src < 0x0E00_0000;
        self.in_dma = false;
    }
}
