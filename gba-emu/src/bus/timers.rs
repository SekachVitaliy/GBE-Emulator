//! Timer scheduling and ticking.
use super::*;

impl Bus {
    pub(super) fn next_timer_event(&self) -> u32 {
        let mut m = 256u32;
        for (i, t) in self.timers.iter().enumerate() {
            if !t.on || (t.ctrl & 4 != 0 && i > 0) { continue; }
            let sh = PRESCALE[(t.ctrl & 3) as usize];
            let need = (((0x10000 - t.counter as u32) << sh).saturating_sub(t.acc)) + t.delay;
            m = m.min(need.max(1));
        }
        m
    }

    pub(super) fn timers_tick(&mut self, cyc: u32) {
        let mut carry = 0u32;
        let mut fifo_req = 0u8;
        for i in 0..4 {
            let t = &mut self.timers[i];
            if !t.on { carry = 0; continue; }
            let ticks = if t.ctrl & 4 != 0 && i > 0 { carry } else {
                let sh = PRESCALE[(t.ctrl & 3) as usize];
                let skip = t.delay.min(cyc);
                t.delay -= skip;
                t.acc += cyc - skip;
                let k = t.acc >> sh;
                t.acc &= (1 << sh) - 1;
                k
            };
            carry = 0;
            let total = t.counter as u32 + ticks;
            if total < 0x10000 { t.counter = total as u16; continue; }
            let period = 0x10000 - t.reload as u32;
            let over = total - 0x10000;
            let n = 1 + over / period;
            t.counter = (t.reload as u32 + over % period) as u16;
            carry = n;
            if t.ctrl & 0x40 != 0 { self.iflag |= 8 << i; }
            if i < 2 { fifo_req |= self.apu.timer_overflow(i, n); }
        }
        for ch in 0..2 { if fifo_req >> ch & 1 != 0 { self.dma_fifo(ch); } }
    }
}
