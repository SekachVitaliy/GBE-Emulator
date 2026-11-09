//! APU: PSG ch1-4, Direct Sound A/B, mixer. Event-driven and lazily ticked (see `Apu::tick`).
//! Output is point-sampled once per window (512 cycles >> SOUNDBIAS resolution), i.e. 32768 Hz << res,
//! then passed through a DC blocker. No box filtering.

const WIN: u32 = 512;
const HP: f32 = 0.9962; // DC-blocker pole, fitted against the reference's slow droop on flat tops
const FS_PERIOD: u32 = 32768; // 512 Hz frame sequencer
const DUTY: [u8; 4] = [0b0000_0001, 0b1000_0001, 0b1000_0111, 0b0111_1110];

// Readable bits per 16-bit register at offset 0x60 + 2*i.
const RMASK: [u16; 24] = [
    0x007F, 0xFFC0, 0x4000, 0, 0xFFC0, 0, 0x4000, 0, 0x00E0, 0xE000, 0x4000, 0, 0xFF00, 0, 0x40FF, 0,
    0xFF77, 0x770F, 0x0080, 0, 0xC3FE, 0, 0, 0,
];

#[derive(Default, Clone, Copy)]
struct Chan {
    on: bool,
    timer: u32,
    pos: u32,
    len: u32,
    vol: u32,
    env_t: u32,
    sh_freq: u32,
    sw_t: u32,
    sw_on: bool,
    lfsr: u32,
}

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
    pub fn new() -> Self {
        let mut a = Apu {
            r: [0; 0x30], wave: [[0; 16]; 2], ch: [Chan::default(); 4], fifo: [Fifo::default(); 2],
            fs_step: 0, fs_left: FS_PERIOD, win_left: WIN, hx: [512.0; 2], hy: [0.0; 2], cur: [512; 2], dirty: true, pend: 0, next_ev: 0, out: Vec::new(),
        };
        a.reset();
        a
    }
    pub fn reset(&mut self) {
        self.r = [0; 0x30];
        self.r[0x28] = 0x00;
        self.r[0x29] = 0x02;
        self.wave = [[0; 16]; 2];
        self.ch = [Chan::default(); 4];
        self.fifo = [Fifo::default(); 2];
        self.fs_step = 0;
        self.fs_left = FS_PERIOD;
        self.win_left = WIN;
        self.hx = [512.0; 2];
        self.hy = [0.0; 2];
        self.cur = [512; 2];
        self.dirty = true;
        self.pend = 0;
        self.next_ev = 0;
        self.out.clear();
    }

    fn r16(&self, o: usize) -> u32 { self.r[o] as u32 | (self.r[o + 1] as u32) << 8 }
    fn master(&self) -> bool { self.r[0x24] & 0x80 != 0 }
    fn freq(&self, i: usize) -> u32 { self.r16([0x04, 0x0C, 0x14][i]) & 0x7FF }
    fn len_en(&self, i: usize) -> bool { self.r[[0x05, 0x0D, 0x15, 0x1D][i]] & 0x40 != 0 }
    fn env_reg(&self, i: usize) -> u32 { (self.r16([0x02, 0x08, 0, 0x18][i]) >> 8) & 0xFF }
    fn dac(&self, i: usize) -> bool {
        if i == 2 { self.r[0x10] & 0x80 != 0 } else { self.env_reg(i) & 0xF8 != 0 }
    }
    fn period(&self, i: usize) -> u32 {
        match i {
            0 | 1 => (2048 - self.freq(i)) * 16,
            2 => (2048 - self.freq(2)) * 8,
            _ => {
                let h = self.r[0x1C] as u32;
                let rr = h & 7;
                let base = if rr == 0 { 16 } else { 32 * rr };
                base << ((h >> 4) + 1).min(30)
            }
        }
    }

    fn trigger(&mut self, i: usize) {
        let dac = self.dac(i);
        let per = self.period(i);
        let env = self.env_reg(i);
        let max_len = if i == 2 { 256 } else { 64 };
        let sweep = self.r[0x00] as u32;
        let f = self.freq(0);
        let c = &mut self.ch[i];
        if c.len == 0 { c.len = max_len; }
        c.on = dac;
        c.timer = per;
        c.pos = 0;
        c.vol = env >> 4;
        c.env_t = if env & 7 == 0 { 8 } else { env & 7 };
        if i == 3 { c.lfsr = 0x7FFF; }
        if i == 0 {
            c.sh_freq = f;
            let (p, s) = ((sweep >> 4) & 7, sweep & 7);
            c.sw_t = if p == 0 { 8 } else { p };
            c.sw_on = p != 0 || s != 0;
            if s != 0 && self.sweep_calc() > 2047 { self.ch[0].on = false; }
        }
        self.dirty = true;
    }

    fn sweep_calc(&self) -> u32 {
        let s = self.r[0] as u32;
        let d = self.ch[0].sh_freq >> (s & 7);
        if s & 8 != 0 { self.ch[0].sh_freq.wrapping_sub(d) } else { self.ch[0].sh_freq + d }
    }

    fn frame_step(&mut self) {
        let st = self.fs_step;
        self.fs_step = (st + 1) & 7;
        if st & 1 == 0 {
            for i in 0..4 {
                if self.len_en(i) && self.ch[i].len > 0 {
                    self.ch[i].len -= 1;
                    if self.ch[i].len == 0 { self.ch[i].on = false; }
                }
            }
        }
        if st == 2 || st == 6 {
            let sw = self.r[0] as u32;
            if self.ch[0].sw_t > 0 { self.ch[0].sw_t -= 1; }
            if self.ch[0].sw_t == 0 {
                let p = (sw >> 4) & 7;
                self.ch[0].sw_t = if p == 0 { 8 } else { p };
                if self.ch[0].sw_on && p != 0 {
                    let n = self.sweep_calc();
                    if n > 2047 { self.ch[0].on = false; }
                    else if sw & 7 != 0 {
                        self.ch[0].sh_freq = n;
                        self.r[4] = n as u8;
                        self.r[5] = (self.r[5] & !7) | (n >> 8) as u8;
                        if self.sweep_calc() > 2047 { self.ch[0].on = false; }
                    }
                }
            }
        }
        if st == 7 {
            for i in [0usize, 1, 3] {
                let e = self.env_reg(i);
                let p = e & 7;
                if p == 0 { continue; }
                let c = &mut self.ch[i];
                c.env_t = c.env_t.saturating_sub(1);
                if c.env_t == 0 {
                    c.env_t = p;
                    if e & 8 != 0 { if c.vol < 15 { c.vol += 1; } } else if c.vol > 0 { c.vol -= 1; }
                }
            }
        }
        self.dirty = true;
    }

    fn step_chan(&mut self, i: usize) {
        let per = self.period(i);
        match i {
            0 | 1 => { let c = &mut self.ch[i]; c.pos = (c.pos + 1) & 7; c.timer += per; }
            2 => { let c = &mut self.ch[2]; c.pos = (c.pos + 1) & 63; c.timer += per; }
            _ => {
                let w7 = self.r[0x1C] & 8 != 0;
                let c = &mut self.ch[3];
                let x = (c.lfsr ^ (c.lfsr >> 1)) & 1;
                c.lfsr = (c.lfsr >> 1) | (x << 14);
                if w7 { c.lfsr = (c.lfsr & !0x40) | (x << 6); }
                c.timer += per;
            }
        }
        self.dirty = true;
    }

    fn chan_out(&self, i: usize) -> u32 {
        let c = &self.ch[i];
        if !c.on { return 0; }
        match i {
            0 | 1 => {
                let d = (self.r[[0x02, 0x08][i]] >> 6) as usize;
                if DUTY[d] >> c.pos & 1 != 0 { c.vol } else { 0 }
            }
            2 => {
                let v = self.r[0x13];
                let bank = if self.r[0x10] & 0x20 != 0 { (c.pos >> 5) as usize } else { ((self.r[0x10] >> 6) & 1) as usize };
                let idx = (c.pos & 31) as usize;
                let b = self.wave[bank][idx >> 1];
                let s = if idx & 1 == 0 { b >> 4 } else { b & 15 } as u32;
                if v & 0x80 != 0 { s * 3 / 4 } else {
                    match (v >> 5) & 3 { 0 => 0, 1 => s, 2 => s >> 1, _ => s >> 2 }
                }
            }
            _ => if c.lfsr & 1 == 0 { c.vol } else { 0 },
        }
    }

    fn recompute(&mut self) {
        self.dirty = false;
        if !self.master() { self.cur = [512, 512]; return; }
        let cl = self.r16(0x20);
        let ch = self.r16(0x22);
        let outs = [self.chan_out(0), self.chan_out(1), self.chan_out(2), self.chan_out(3)];
        let bias = (self.r16(0x28) & 0x3FE) as i32;
        for side in 0..2 {
            let (vol, en_sh) = if side == 0 { ((cl >> 4) & 7, 12) } else { (cl & 7, 8) };
            let mut s = 0u32;
            for i in 0..4 { if cl >> (en_sh + i) & 1 != 0 { s += outs[i as usize]; } }
            let scale = [1, 2, 4, 4][(ch & 3) as usize];
            let mut v = (s * (vol + 1) * scale / 4) as i32;
            for f in 0..2 {
                let en = if side == 0 { 9 + 4 * f } else { 8 + 4 * f };
                if ch >> en & 1 != 0 {
                    let full = ch >> (2 + f) & 1 != 0;
                    v += self.fifo[f as usize].cur as i32 * if full { 2 } else { 1 };
                }
            }
            self.cur[side as usize] = (v + bias).clamp(0, 0x3FF);
        }
    }

    /// Lazy: nothing observable changes before the next channel/frame-sequencer/sample-window event,
    /// so cycles are banked and applied in one go at the event or before any state access.
    #[inline]
    pub fn tick(&mut self, cycles: u32) {
        self.pend += cycles;
        if self.pend >= self.next_ev { self.flush(); }
    }

    fn flush(&mut self) {
        let n = std::mem::take(&mut self.pend);
        if n > 0 { self.run(n); }
        let mut e = self.win_left.min(self.fs_left);
        if self.master() { for c in &self.ch { if c.on { e = e.min(c.timer.max(1)); } } }
        self.next_ev = e.max(1);
    }

    fn run(&mut self, cycles: u32) {
        let mut rem = cycles;
        let master = self.master();
        while rem > 0 {
            if self.dirty { self.recompute(); }
            let mut dt = rem.min(self.win_left).min(self.fs_left);
            if master {
                for c in &self.ch { if c.on && c.timer < dt { dt = c.timer.max(1); } }
            }
            rem -= dt;
            self.win_left -= dt;
            self.fs_left -= dt;
            if master {
                for i in 0..4 {
                    if self.ch[i].on {
                        self.ch[i].timer -= dt.min(self.ch[i].timer);
                        if self.ch[i].timer == 0 { self.step_chan(i); }
                    }
                }
            }
            if self.fs_left == 0 { self.fs_left = FS_PERIOD; if master { self.frame_step(); } }
            if self.win_left == 0 {
                // bias resolution bits: 9..6-bit, output rate 32768<<res (Mesen emits at that rate)
                let res = (self.r[0x29] >> 6) as u32;
                self.win_left = WIN >> res;
                for s in 0..2 {
                    let x = ((self.cur[s] >> res) << res) as f32;
                    // DC blocker (matches Mesen's slow droop on flat tops)
                    let y = x - self.hx[s] + HP * self.hy[s];
                    self.hx[s] = x;
                    self.hy[s] = y;
                    self.out.push((y * 64.0).clamp(-32768.0, 32767.0) as i16);
                }
            }
        }
    }

    pub fn read_io(&self, off: u32) -> u8 {
        if (0x90..0xA0).contains(&off) {
            return self.wave[1 - ((self.r[0x10] >> 6) & 1) as usize][(off - 0x90) as usize];
        }
        let o = off.wrapping_sub(0x60) as usize;
        if o >= 0x2A { return 0; }
        let m = RMASK[o >> 1];
        let mut v = self.r[o] & (m >> (8 * (o & 1))) as u8;
        if o == 0x24 {
            v |= (0..4).fold(0, |a, i| a | (self.ch[i].on as u8) << i);
        }
        v
    }

    pub fn write_io(&mut self, off: u32, v: u8) {
        self.flush();
        self.next_ev = 0;
        if (0x90..0xA0).contains(&off) {
            let bank = 1 - ((self.r[0x10] >> 6) & 1) as usize;
            self.wave[bank][(off - 0x90) as usize] = v;
            return;
        }
        let o = off.wrapping_sub(0x60) as usize;
        if o >= 0x2A { return; }
        if o == 0x24 {
            let was = self.master();
            self.r[o] = v & 0x80;
            if was && v & 0x80 == 0 {
                for b in &mut self.r[..0x22] { *b = 0; }
                self.ch = [Chan::default(); 4];
            } else if !was && v & 0x80 != 0 {
                self.fs_step = 0;
            }
            self.dirty = true;
            return;
        }
        if o == 0x25 || o == 0x26 || o == 0x27 { return; }
        if o < 0x22 && !self.master() { return; }
        match o {
            0x23 => {
                self.r[o] = v & 0x77;
                if v & 0x08 != 0 { self.fifo[0] = Fifo::default(); }
                if v & 0x80 != 0 { self.fifo[1] = Fifo::default(); }
            }
            0x05 | 0x0D | 0x15 | 0x1D => {
                self.r[o] = v & 0x7F;
                if v & 0x80 != 0 { self.trigger(match o { 0x05 => 0, 0x0D => 1, 0x15 => 2, _ => 3 }); }
            }
            _ => {
                self.r[o] = v;
                match o {
                    0x02 => self.ch[0].len = 64 - (v as u32 & 63),
                    0x08 => self.ch[1].len = 64 - (v as u32 & 63),
                    0x12 => self.ch[2].len = 256 - v as u32,
                    0x18 => self.ch[3].len = 64 - (v as u32 & 63),
                    _ => {}
                }
                if matches!(o, 0x02 | 0x03 | 0x08 | 0x09 | 0x10 | 0x18 | 0x19) {
                    // DAC off kills the channel
                    for i in 0..4 { if !self.dac(i) { self.ch[i].on = false; } }
                }
            }
        }
        self.dirty = true;
    }

    pub fn fifo_write(&mut self, ch: usize, word: u32) {
        let f = &mut self.fifo[ch & 1];
        for b in 0..4 {
            if f.n < 32 { f.buf[(f.rd + f.n) & 31] = (word >> (8 * b)) as i8; f.n += 1; }
        }
    }

    pub fn timer_overflow(&mut self, timer: usize, count: u32) -> u8 {
        let mut need = 0;
        let sel = self.r16(0x22);
        for f in 0..2 {
            if (sel >> (10 + 4 * f) & 1) as usize != timer { continue; }
            self.flush();
            for _ in 0..count {
                let q = &mut self.fifo[f];
                if q.n > 0 { q.cur = q.buf[q.rd]; q.rd = (q.rd + 1) & 31; q.n -= 1; self.dirty = true; }
            }
            if self.fifo[f].n <= 16 { need |= 1 << f; }
        }
        need
    }

    pub fn drain(&mut self, out: &mut Vec<i16>) {
        self.flush();
        out.append(&mut self.out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w16(a: &mut Apu, off: u32, v: u16) { a.write_io(off, v as u8); a.write_io(off + 1, (v >> 8) as u8); }

    #[test]
    fn square_ch2_frequency_and_off_when_master_disabled() {
        let mut a = Apu::new();
        a.write_io(0x84, 0x80);
        w16(&mut a, 0x80, 0xFF77);
        w16(&mut a, 0x68, 0xF080); // vol 15, duty 50%
        w16(&mut a, 0x6C, 0x8000 | 1024); // 131072/(2048-1024)=128 Hz
        assert_eq!(a.read_io(0x84) & 2, 2);
        a.tick(16_777_216 / 10);
        let mut o = Vec::new();
        a.drain(&mut o);
        let l: Vec<i16> = o.iter().step_by(2).copied().collect();
        let mid = 0i16; // DC-blocked output is centred on zero
        let edges = l.windows(2).filter(|w| w[0] <= mid && w[1] > mid).count();
        assert!((12..=14).contains(&edges), "edges {edges} len {} first {:?}", l.len(), &l[..300].iter().step_by(10).collect::<Vec<_>>()); // ~12.8 cycles in 0.1 s
        a.write_io(0x84, 0);
        assert_eq!(a.read_io(0x84) & 0x8F, 0);
        assert_eq!(a.read_io(0x69), 0);
    }

    #[test]
    fn length_expires_and_fifo_requests_dma() {
        let mut a = Apu::new();
        a.write_io(0x84, 0x80);
        w16(&mut a, 0x62, 0xF03F); // len 1 -> 1/256 s
        w16(&mut a, 0x64, 0xC000 | 1000);
        assert_eq!(a.read_io(0x84) & 1, 1);
        a.tick(100_000);
        assert_eq!(a.read_io(0x84) & 1, 0);
        w16(&mut a, 0x82, 0x0304);
        for _ in 0..8 { a.fifo_write(0, 0x7F7F7F7F); }
        let mut m = 0;
        for _ in 0..16 { m = a.timer_overflow(0, 1); }
        assert_eq!(m & 1, 1);
        a.tick(1024);
        let mut o = Vec::new();
        a.drain(&mut o);
        assert!(o.iter().any(|&s| s > 1000));
    }

    #[test]
    fn noise_and_wave_produce_sound() {
        let mut a = Apu::new();
        a.write_io(0x84, 0x80);
        w16(&mut a, 0x80, 0xFF77);
        w16(&mut a, 0x78, 0xF000);
        w16(&mut a, 0x7C, 0x8011);
        a.tick(5000);
        for i in 0..16 { a.write_io(0x90 + i, 0xF0); } // bank 1 (not playing)
        w16(&mut a, 0x70, 0x0080 | 0x40); // select bank 1, enable
        w16(&mut a, 0x72, 0x2000);
        w16(&mut a, 0x74, 0x8400);
        let mut o = Vec::new();
        a.tick(20000);
        a.drain(&mut o);
        assert!(o.iter().any(|&s| s != 0));
    }
}
