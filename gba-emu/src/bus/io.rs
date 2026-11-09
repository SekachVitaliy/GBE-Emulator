//! Memory-mapped IO register reads and writes.
use super::*;

impl Bus {
    pub(super) fn io_read16(&mut self, off: u32) -> Option<u16> {
        Some(match off {
            0x10..=0x47 | 0x4C | 0x4E | 0x54..=0x5F | 0x8C..=0x8F | 0xA0..=0xAF | 0xE0..=0xFF => return None, // write-only / unused: open bus
            0..=0x57 => {
                if off == 4 || off == 6 { self.sync_to(self.cycles); }
                let mut v = self.ppu.read_io(off);
                // the reference raises the VBlank flag a few cycles after line 160 starts
                if off == 4 && self.ppu_pos >= 160 * 1232 && self.ppu_pos < 160 * 1232 + VBLANK_DELAY { v &= !1; }
                match off { 0x48 | 0x4A => v & 0x3F3F, 0x50 => v & 0x3FFF, 0x52 => v & 0x1F1F, _ => v }
            }
            0x60..=0xA7 => self.apu.read_io(off) as u16 | (self.apu.read_io(off + 1) as u16) << 8,
            0xB0..=0xDF => match (off - 0xB0) % 12 {
                10 => self.io[off as usize >> 1] & if off < 0xDE { 0xF7E0 } else { 0xFFE0 },
                8 => 0,
                _ => return None,
            },
            0x100..=0x10F => {
                self.next_ev = 0; self.sync();
                let t = &self.timers[((off - 0x100) >> 2) as usize];
                if off & 2 == 0 { if t.on && t.ctrl & 4 == 0 { t.counter.wrapping_sub((READ_ADJ >> PRESCALE[(t.ctrl & 3) as usize]) as u16) } else { t.counter } } else { t.ctrl & 0xC7 }
            }
            0x130 => !(self.keys as u16) & 0x3FF,
            0x200 => self.ie,
            0x202 => { self.next_ev = 0; self.sync(); self.iflag }
            0x204 => self.waitcnt,
            0x208 => self.ime,
            0x300 => self.postflg as u16,
            0x206 | 0x20A | 0x302.. | 0x136 | 0x142 | 0x15A => 0,
            _ => self.io[off as usize >> 1],
        })
    }

    pub(super) fn io_write16(&mut self, off: u32, v: u32, mask: u32) {
        let (m, v) = (mask as u16, (v & mask) as u16);
        let merge = |old: u16| (old & !m) | v;
        match off {
            0..=0x57 => self.ppu.write_io(off, v, m),
            0x60..=0x9F => {
                if m & 0xFF != 0 { self.apu.write_io(off, v as u8); }
                if m & 0xFF00 != 0 { self.apu.write_io(off + 1, (v >> 8) as u8); }
            }
            0xA0..=0xA7 => {
                let ch = ((off - 0xA0) >> 2) as usize;
                let sh = (off & 2) * 8;
                let wm = (m as u32) << sh;
                self.fifo_buf[ch] = (self.fifo_buf[ch] & !wm) | ((v as u32) << sh);
                if off & 2 != 0 && m & 0xFF00 != 0 { self.apu.fifo_write(ch, self.fifo_buf[ch]); }
            }
            0xB0..=0xDF => {
                let i = off as usize >> 1;
                let old = self.io[i];
                self.io[i] = merge(old);
                if (off - 0xB0) % 12 == 10 { self.dma_ctrl_write(((off - 0xB0) / 12) as usize, old, self.io[i]); }
            }
            0x100..=0x10F => {
                let n = ((off - 0x100) >> 2) as usize;
                if off & 2 == 0 { self.timers[n].reload = merge(self.timers[n].reload); }
                else {
                    let t = &mut self.timers[n];
                    let c = merge(t.ctrl) & 0xC7;
                    let on = c & 0x80 != 0;
                    if on && !t.on { t.counter = t.reload; t.acc = 0; t.delay = START_DELAY; }
                    t.ctrl = c; t.on = on;
                }
            }
            0x132 => { self.io[0x99] = merge(self.io[0x99]); self.check_keypad(); }
            0x200 => { self.ie = merge(self.ie) & 0x3FFF; self.upd_cond(self.cycles); }
            0x202 => { self.iflag &= !v; self.upd_cond(self.cycles); }
            0x204 => self.set_waitcnt(merge(self.waitcnt)),
            0x208 => self.ime = merge(self.ime) & 1,
            0x300 => {
                if m & 0xFF != 0 { self.postflg = v as u8 & 1; }
                if m & 0xFF00 != 0 { self.halt = true; }
            }
            0x302.. => {}
            _ => self.io[off as usize >> 1] = merge(self.io[off as usize >> 1]),
        }
    }
}
