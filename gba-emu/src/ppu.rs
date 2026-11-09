//! GBA PPU: scanline renderer. Owns VRAM/PAL/OAM and display IO 0x04000000..0x04000056.
//! Notes: address mirroring and 8-bit write duplication are the bus's job.
//! HBlank begins at cycle 1006 of each 1232-cycle line (960 + 46 delay), the line is rendered then.

pub struct PpuEvents {
    pub irq: u16,
    pub hblank_start: bool,
    pub vblank_start: bool,
}

const TRANSP: u16 = 0x8000;
const HB: u32 = 1006;
const LINE: u32 = 1232;
const SIZES: [[(i32, i32); 4]; 3] = [
    [(8, 8), (16, 16), (32, 32), (64, 64)],
    [(16, 8), (32, 8), (32, 16), (64, 32)],
    [(8, 16), (8, 32), (16, 32), (32, 64)],
];

pub struct Ppu {
    pub vram: Vec<u8>,
    pub pal: Vec<u8>,
    pub oam: Vec<u8>,
    pub fb: Vec<u32>,
    r: [u16; 0x2C],
    line: u32,
    cyc: u32,
    refx: [i32; 2],
    refy: [i32; 2],
    snap: [(i32, i32); 2],
    pending_irq: u16,
}

fn sext28(v: u32) -> i32 {
    ((v << 4) as i32) >> 4
}

#[inline]
fn rgb(c: u16) -> u32 {
    let r = (c & 31) as u32;
    let g = ((c >> 5) & 31) as u32;
    let b = ((c >> 10) & 31) as u32;
    let e = |v: u32| (v << 3) | (v >> 2);
    0xFF00_0000 | (e(b) << 16) | (e(g) << 8) | e(r)
}

#[inline]
fn blend(a: u16, b: u16, ea: u32, eb: u32) -> u16 {
    let ch = |s: u32| (((((a as u32) >> s) & 31) * ea + (((b as u32) >> s) & 31) * eb) >> 4).min(31);
    (ch(0) | (ch(5) << 5) | (ch(10) << 10)) as u16
}

#[inline]
fn fade(a: u16, y: u32, up: bool) -> u16 {
    let ch = |s: u32| {
        let c = ((a as u32) >> s) & 31;
        if up { c + (((31 - c) * y) >> 4) } else { c - ((c * y) >> 4) }
    };
    (ch(0) | (ch(5) << 5) | (ch(10) << 10)) as u16
}

impl Ppu {
    pub fn new() -> Self {
        Ppu {
            vram: vec![0; 96 << 10],
            pal: vec![0; 1024],
            oam: vec![0; 1024],
            fb: vec![0xFF00_0000; 240 * 160],
            r: [0; 0x2C],
            line: 0,
            cyc: 0,
            refx: [0; 2],
            refy: [0; 2],
            snap: [(0, 0); 2],
            pending_irq: 0,
        }
    }

    pub fn reset(&mut self) {
        *self = Ppu::new();
    }

    pub fn read_io(&self, off: u32) -> u16 {
        let i = ((off & !1) >> 1) as usize;
        match i {
            0 => self.r[0],
            1 => self.r[1] & 1,
            2 => {
                let lyc = self.r[2] >> 8;
                let (line, cyc) = self.seen();
                let mut v = self.r[2] & 0xFF38;
                if (160..227).contains(&line) { v |= 1; }
                if cyc >= HB { v |= 2; }
                if line as u16 == lyc { v |= 4; }
                v
            }
            3 => self.vc_line() as u16,
            4..=7 => self.r[i] & if i < 6 { 0xDFFF } else { 0xFFFF },
            0x24 | 0x25 => self.r[i],
            0x28 | 0x29 => self.r[i],
            _ => 0,
        }
    }

    const LAT: u32 = 0;
    #[inline] fn vc_line(&self) -> u32 {
        self.seen().0
    }
    /// (line, cycle) as observed by a CPU read: the access completes a few cycles after the bus
    /// call is made (measured against the reference with lockstep on celeste's VCOUNT polling loop).
    #[inline]
    fn seen(&self) -> (u32, u32) {
        let c = self.cyc + Self::LAT;
        if c >= LINE { (if self.line == 227 { 0 } else { self.line + 1 }, c - LINE) } else { (self.line, c) }
    }

    pub fn write_io(&mut self, off: u32, v: u16, mask: u16) {
        let i = ((off & !1) >> 1) as usize;
        if i >= 0x2C || i == 3 { return; }
        let old = self.r[i];
        let mut nv = (old & !mask) | (v & mask);
        if i == 2 {
            nv = (nv & 0xFF38) | (old & 7);
            // LYC change / enable can create a new vcount match
            let was = self.line as u16 == old >> 8 && old & 0x20 != 0;
            let now = self.line as u16 == nv >> 8 && nv & 0x20 != 0;
            if now && !was { self.pending_irq |= 4; }
        }
        self.r[i] = nv;
        match i {
            0x14 | 0x15 => self.refx[0] = sext28(self.r[0x14] as u32 | (self.r[0x15] as u32) << 16),
            0x16 | 0x17 => self.refy[0] = sext28(self.r[0x16] as u32 | (self.r[0x17] as u32) << 16),
            0x1C | 0x1D => self.refx[1] = sext28(self.r[0x1C] as u32 | (self.r[0x1D] as u32) << 16),
            0x1E | 0x1F => self.refy[1] = sext28(self.r[0x1E] as u32 | (self.r[0x1F] as u32) << 16),
            _ => {}
        }
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

    #[inline]
    fn v8(&self, a: usize) -> u8 {
        let a = a & 0x1FFFF;
        self.vram[if a >= 0x18000 { a - 0x8000 } else { a }]
    }
    #[inline]
    fn v16(&self, a: usize) -> u16 {
        self.v8(a) as u16 | (self.v8(a + 1) as u16) << 8
    }
    #[inline]
    fn pc(&self, i: usize) -> u16 {
        (self.pal[i] as u16 | (self.pal[i + 1] as u16) << 8) & 0x7FFF
    }

    fn render_line(&mut self) {
        let y = self.line as usize;
        let dc = self.r[0];
        let mode = (dc & 7) as u32;
        let mosaic = self.r[0x26];
        let (bgh, bgv) = ((mosaic & 15) as usize + 1, ((mosaic >> 4) & 15) as usize + 1);

        let advance = |s: &mut Ppu| {
            for i in 0..2 {
                s.refx[i] = s.refx[i].wrapping_add(s.r[0x11 + 8 * i] as i16 as i32);
                s.refy[i] = s.refy[i].wrapping_add(s.r[0x13 + 8 * i] as i16 as i32);
            }
        };
        if dc & 0x80 != 0 {
            for p in &mut self.fb[y * 240..y * 240 + 240] { *p = 0xFFFF_FFFF; }
            advance(self);
            return;
        }

        // which BG layers produce pixels
        let mut mask = 0u8;
        match mode {
            0 => mask = 0xF,
            1 => mask = 0x7,
            2 => mask = 0xC,
            3..=5 => mask = 0x4,
            _ => {}
        }
        mask &= ((dc >> 8) & 0xF) as u8;

        let mut bg = [[TRANSP; 240]; 4];
        for i in 0..4usize {
            if mask & (1 << i) == 0 { continue; }
            let cnt = self.r[4 + i];
            let mos = cnt & 0x40 != 0;
            let affine = (mode == 1 && i == 2) || mode == 2 || mode >= 3;
            if !affine {
                let ye = if mos { y - y % bgv } else { y };
                self.render_text(i, ye, &mut bg[i]);
            } else {
                let k = i - 2;
                let (cx, cy) = if mos && bgv > 1 {
                    if y % bgv == 0 { self.snap[k] = (self.refx[k], self.refy[k]); }
                    self.snap[k]
                } else {
                    (self.refx[k], self.refy[k])
                };
                self.render_affine(k, mode, cx, cy, &mut bg[i]);
            }
            if mos && bgh > 1 {
                for x in 0..240 { bg[i][x] = bg[i][x - x % bgh]; }
            }
        }

        // sprites
        let mut ocol = [TRANSP; 240];
        let mut oinf = [0u8; 240];
        let mut owin = [false; 240];
        let obj_on = dc & 0x1000 != 0;
        if obj_on { self.render_sprites(y, mode, &mut ocol, &mut oinf, &mut owin); }

        // compose
        let mut list = [(0u8, 0u8); 4];
        let mut nl = 0;
        for p in 0..4u8 {
            for i in 0..4u8 {
                if mask & (1 << i) != 0 && (self.r[4 + i as usize] & 3) as u8 == p {
                    list[nl] = (p, i);
                    nl += 1;
                }
            }
        }
        let backdrop = self.pc(0);
        let winen = dc & 0xE000 != 0;
        let win_rng = |h: u16, v: u16| -> (usize, usize, bool) {
            let (x1, mut x2) = ((h >> 8) as usize, (h & 0xFF) as usize);
            if x2 > 240 { x2 = 240; }
            let (y1, mut y2) = ((v >> 8) as usize, (v & 0xFF) as usize);
            if y2 > 160 { y2 = 160; }
            // x1 > x2 / y1 > y2 wrap around (verified vs the reference with synthetic ROMs)
            (x1, x2, if y1 <= y2 { y >= y1 && y < y2 } else { y >= y1 || y < y2 })
        };
        let w0 = if dc & 0x2000 != 0 { Some(win_rng(self.r[0x20], self.r[0x22])) } else { None };
        let w1 = if dc & 0x4000 != 0 { Some(win_rng(self.r[0x21], self.r[0x23])) } else { None };
        let (winin, winout) = (self.r[0x24], self.r[0x25]);
        let bld = self.r[0x28];
        let bmode = (bld >> 6) & 3;
        let (t1, t2) = ((bld & 0x3F) as u8, ((bld >> 8) & 0x3F) as u8);
        let eva = ((self.r[0x29] & 31) as u32).min(16);
        let evb = (((self.r[0x29] >> 8) & 31) as u32).min(16);
        let evy = ((self.r[0x2A] & 31) as u32).min(16);

        for x in 0..240usize {
            let mut wm = 0x3Fu8;
            if winen {
                wm = if matches!(w0, Some((a, b, true)) if inw(a, b, x)) {
                    (winin & 0x3F) as u8
                } else if matches!(w1, Some((a, b, true)) if inw(a, b, x)) {
                    ((winin >> 8) & 0x3F) as u8
                } else if dc & 0x8000 != 0 && owin[x] {
                    ((winout >> 8) & 0x3F) as u8
                } else {
                    (winout & 0x3F) as u8
                };
            }
            let ov = ocol[x] != TRANSP && wm & 0x10 != 0;
            let op = oinf[x] & 3;
            let mut c = [(0u16, 0u8); 2];
            let mut n = 0;
            let mut odone = !ov;
            for &(bp, idx) in &list[..nl] {
                if !odone && op <= bp {
                    c[n] = (ocol[x], 4);
                    n += 1;
                    odone = true;
                    if n == 2 { break; }
                }
                if wm & (1 << idx) != 0 {
                    let px = bg[idx as usize][x];
                    if px != TRANSP {
                        c[n] = (px, idx);
                        n += 1;
                        if n == 2 { break; }
                    }
                }
            }
            if n < 2 && !odone { c[n] = (ocol[x], 4); n += 1; }
            if n < 2 { c[n] = (backdrop, 5); n += 1; }
            if n < 2 { c[1] = (backdrop, 5); n = 1; }
            let (top, tid) = c[0];
            let (sec, sid) = c[1];
            let has2 = n == 2;
            let mut out = top;
            if tid == 4 && oinf[x] & 4 != 0 && wm & 0x20 != 0 {
                if has2 && t2 & (1 << sid) != 0 { out = blend(top, sec, eva, evb); }
            } else if wm & 0x20 != 0 && t1 & (1 << tid) != 0 {
                match bmode {
                    1 if has2 && t2 & (1 << sid) != 0 => out = blend(top, sec, eva, evb),
                    2 => out = fade(top, evy, true),
                    3 => out = fade(top, evy, false),
                    _ => {}
                }
            }
            self.fb[y * 240 + x] = rgb(out);
        }
        advance(self);
    }

    fn render_text(&self, i: usize, y: usize, out: &mut [u16; 240]) {
        let cnt = self.r[4 + i];
        let hofs = (self.r[8 + 2 * i] & 0x1FF) as usize;
        let vofs = (self.r[9 + 2 * i] & 0x1FF) as usize;
        let size = cnt >> 14;
        let (wmask, hmask) = (if size & 1 != 0 { 511 } else { 255 }, if size & 2 != 0 { 511 } else { 255 });
        let cbase = ((cnt >> 2) & 3) as usize * 0x4000;
        let sbase = ((cnt >> 8) & 31) as usize * 0x800;
        let bpp8 = cnt & 0x80 != 0;
        let yy = (y + vofs) & hmask;
        let ty = yy >> 3;
        let mut x = 0usize;
        while x < 240 {
            let xx = (x + hofs) & wmask;
            let tx = xx >> 3;
            let blk = match size { 0 => 0, 1 => tx >> 5, 2 => ty >> 5, _ => (tx >> 5) + 2 * (ty >> 5) };
            let e = self.v16(sbase + blk * 0x800 + ((ty & 31) * 32 + (tx & 31)) * 2) as usize;
            let tile = e & 0x3FF;
            let hf = e & 0x400 != 0;
            let py = if e & 0x800 != 0 { 7 - (yy & 7) } else { yy & 7 };
            let pb = (e >> 12) * 16;
            for px in (xx & 7)..8 {
                if x >= 240 { break; }
                let p = if hf { 7 - px } else { px };
                let idx = if bpp8 {
                    self.v8(cbase + tile * 64 + py * 8 + p) as usize
                } else {
                    let b = self.v8(cbase + tile * 32 + py * 4 + (p >> 1)) as usize;
                    if p & 1 != 0 { b >> 4 } else { b & 15 }
                };
                if idx != 0 {
                    out[x] = self.pc(2 * if bpp8 { idx } else { pb + idx });
                }
                x += 1;
            }
        }
    }

    fn render_affine(&self, k: usize, mode: u32, mut cx: i32, mut cy: i32, out: &mut [u16; 240]) {
        let cnt = self.r[6 + k];
        let pa = self.r[0x10 + 8 * k] as i16 as i32;
        let pc = self.r[0x12 + 8 * k] as i16 as i32;
        let frame = if self.r[0] & 0x10 != 0 { 0xA000 } else { 0 };
        let size = 128i32 << (cnt >> 14);
        let wrap = cnt & 0x2000 != 0;
        let cbase = ((cnt >> 2) & 3) as usize * 0x4000;
        let sbase = ((cnt >> 8) & 31) as usize * 0x800;
        for x in 0..240usize {
            let (mut px, mut py) = (cx >> 8, cy >> 8);
            cx = cx.wrapping_add(pa);
            cy = cy.wrapping_add(pc);
            match mode {
                3 | 4 | 5 => {
                    let (w, h) = if mode == 5 { (160, 128) } else { (240, 160) };
                    if px < 0 || py < 0 || px >= w || py >= h { continue; }
                    let (px, py) = (px as usize, py as usize);
                    match mode {
                        3 => out[x] = self.v16((py * 240 + px) * 2) & 0x7FFF,
                        4 => {
                            let idx = self.v8(frame + py * 240 + px) as usize;
                            if idx != 0 { out[x] = self.pc(idx * 2); }
                        }
                        _ => out[x] = self.v16(frame + (py * 160 + px) * 2) & 0x7FFF,
                    }
                }
                _ => {
                    if wrap {
                        px &= size - 1;
                        py &= size - 1;
                    } else if px < 0 || py < 0 || px >= size || py >= size {
                        continue;
                    }
                    let (px, py) = (px as usize, py as usize);
                    let tile = self.v8(sbase + (py >> 3) * (size as usize >> 3) + (px >> 3)) as usize;
                    let idx = self.v8(cbase + tile * 64 + (py & 7) * 8 + (px & 7)) as usize;
                    if idx != 0 { out[x] = self.pc(idx * 2); }
                }
            }
        }
    }

    fn render_sprites(&self, y: usize, mode: u32, col: &mut [u16; 240], inf: &mut [u8; 240], owin: &mut [bool; 240]) {
        let dc = self.r[0];
        let one_d = dc & 0x40 != 0;
        let mut budget: i32 = if dc & 0x20 != 0 { 954 } else { 1210 };
        let ms = self.r[0x26];
        let (omh, omv) = (((ms >> 8) & 15) as i32 + 1, ((ms >> 12) & 15) as i32 + 1);
        let o = &self.oam;
        let rd = |a: usize| o[a] as u16 | (o[a + 1] as u16) << 8;
        for n in 0..128usize {
            let (a0, a1, a2) = (rd(n * 8), rd(n * 8 + 2), rd(n * 8 + 4));
            let affine = a0 & 0x100 != 0;
            if !affine && a0 & 0x200 != 0 { continue; }
            let om = (a0 >> 10) & 3;
            let shape = (a0 >> 14) as usize;
            if om == 3 || shape == 3 { continue; }
            let (w, h) = SIZES[shape][(a1 >> 14) as usize];
            let dbl = affine && a0 & 0x200 != 0;
            let (bw, bh) = if dbl { (w * 2, h * 2) } else { (w, h) };
            let sy = (a0 & 0xFF) as i32;
            let row = (y as i32 - sy) & 0xFF;
            if row >= bh { continue; }
            let cost = if affine { 10 + 2 * bw } else { w };
            let avail = budget;
            budget -= cost;
            if avail <= 0 { break; }
            let maxpx = if affine { ((avail - 10) / 2).clamp(0, bw) } else { avail.min(w) };
            let tile = (a2 & 0x3FF) as usize;
            if mode >= 3 && tile < 512 { continue; }
            let prio = ((a2 >> 10) & 3) as u8;
            let bpp8 = a0 & 0x2000 != 0;
            let pbank = ((a2 >> 12) & 15) as usize * 16;
            let mut sx = (a1 & 0x1FF) as i32;
            if sx >= 256 { sx -= 512; }
            let mos = a0 & 0x1000 != 0 && om != 2;
            let mut r0 = row;
            if mos && omv > 1 {
                let r2 = (y as i32 - (y as i32 % omv) - sy) & 0xFF;
                if r2 < bh { r0 = r2; }
            }
            let (pa, pb, pcc, pd) = if affine {
                let b = ((a1 >> 9) & 31) as usize * 32;
                (rd(b + 6) as i16 as i32, rd(b + 14) as i16 as i32, rd(b + 22) as i16 as i32, rd(b + 30) as i16 as i32)
            } else { (0, 0, 0, 0) };
            for k in 0..maxpx {
                let x = sx + k;
                if x < 0 || x >= 240 { continue; }
                let mut kk = k;
                if mos && omh > 1 { kk = (x - x % omh - sx).clamp(0, bw - 1); }
                let (tx, ty);
                if affine {
                    let (ox, oy) = (kk - bw / 2, r0 - bh / 2);
                    tx = ((pa * ox + pb * oy) >> 8) + w / 2;
                    ty = ((pcc * ox + pd * oy) >> 8) + h / 2;
                    if tx < 0 || ty < 0 || tx >= w || ty >= h { continue; }
                } else {
                    tx = if a1 & 0x1000 != 0 { w - 1 - kk } else { kk };
                    ty = if a1 & 0x2000 != 0 { h - 1 - r0 } else { r0 };
                }
                let (tcx, tcy) = ((tx >> 3) as usize, (ty >> 3) as usize);
                let idx;
                let c;
                if bpp8 {
                    let tn = (tile + if one_d { tcy * (w as usize >> 3) * 2 } else { tcy * 32 } + tcx * 2) & 0x3FF;
                    idx = self.v8(0x10000 + tn * 32 + (ty as usize & 7) * 8 + (tx as usize & 7)) as usize;
                    c = 0x200 + idx * 2;
                } else {
                    let tn = (tile + if one_d { tcy * (w as usize >> 3) } else { tcy * 32 } + tcx) & 0x3FF;
                    let b = self.v8(0x10000 + tn * 32 + (ty as usize & 7) * 4 + ((tx as usize & 7) >> 1)) as usize;
                    idx = if tx & 1 != 0 { b >> 4 } else { b & 15 };
                    c = 0x200 + (pbank + idx) * 2;
                }
                if idx == 0 { continue; }
                let xu = x as usize;
                if om == 2 {
                    owin[xu] = true;
                } else if col[xu] == TRANSP || prio < (inf[xu] & 3) {
                    col[xu] = self.pc(c);
                    inf[xu] = prio | if om == 1 { 4 } else { 0 };
                }
            }
        }
    }
}
#[inline] fn inw(a: usize, b: usize, x: usize) -> bool { if a <= b { x >= a && x < b } else { x >= a || x < b } }
