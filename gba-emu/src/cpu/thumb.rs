use super::*;

impl Cpu {
    // ---------------------------------------------------------------- THUMB
    pub(super) fn thumb_undef<B: CpuBus>(&mut self, b: &mut B) {
        let lr = self.r[15].wrapping_sub(2);
        self.exception(b, M_UND, 0x04, lr);
    }

    pub(super) fn exec_thumb<B: CpuBus>(&mut self, b: &mut B, op: u32) {
        match op >> 11 {
            0..=2 => {
                // shift by immediate
                let rd = (op & 7) as usize;
                let v = self.r[((op >> 3) & 7) as usize];
                let c = self.cpsr & C != 0;
                let (r, c) = shift_imm(op >> 11, v, (op >> 6) & 31, c);
                self.r[rd] = r;
                self.set_nz(r);
                self.set_c(c);
            }
            3 => {
                let rd = (op & 7) as usize;
                let a = self.r[((op >> 3) & 7) as usize];
                let x = if op & (1 << 10) != 0 { (op >> 6) & 7 } else { self.r[((op >> 6) & 7) as usize] };
                self.r[rd] = if op & (1 << 9) != 0 { self.adc(a, !x, 1, true) } else { self.adc(a, x, 0, true) };
            }
            4..=7 => {
                let rd = ((op >> 8) & 7) as usize;
                let imm = op & 0xFF;
                let a = self.r[rd];
                match (op >> 11) & 3 {
                    0 => {
                        self.r[rd] = imm;
                        self.set_nz(imm);
                    }
                    1 => { self.adc(a, !imm, 1, true); }
                    2 => self.r[rd] = self.adc(a, imm, 0, true),
                    _ => self.r[rd] = self.adc(a, !imm, 1, true),
                }
            }
            8 => {
                if op & 0x0400 == 0 { self.thumb_alu(b, op) } else { self.thumb_hi(b, op) }
            }
            9 => {
                // LDR pc-relative
                let rd = ((op >> 8) & 7) as usize;
                let a = (self.r[15] & !3).wrapping_add((op & 0xFF) << 2);
                self.r[rd] = self.ld32(b, a, false);
                b.idle(1);
            }
            10 | 11 => {
                let rm = self.r[((op >> 6) & 7) as usize];
                let a = self.r[((op >> 3) & 7) as usize].wrapping_add(rm);
                let rd = (op & 7) as usize;
                match (op >> 9) & 7 {
                    0 => b.write32(a, self.r[rd], false),
                    1 => b.write16(a, self.r[rd] & 0xFFFF, false),
                    2 => b.write8(a, self.r[rd] & 0xFF, false),
                    3 => { self.r[rd] = self.ld8s(b, a, false); b.idle(1); }
                    4 => { self.r[rd] = self.ld32(b, a, false); b.idle(1); }
                    5 => { self.r[rd] = self.ld16(b, a, false); b.idle(1); }
                    6 => { self.r[rd] = b.read8(a, false) & 0xFF; b.idle(1); }
                    _ => { self.r[rd] = self.ld16s(b, a, false); b.idle(1); }
                }
            }
            12..=15 => {
                let rb = self.r[((op >> 3) & 7) as usize];
                let rd = (op & 7) as usize;
                let imm = (op >> 6) & 31;
                match (op >> 11) & 3 {
                    0 => b.write32(rb.wrapping_add(imm << 2), self.r[rd], false),
                    1 => { let a = rb.wrapping_add(imm << 2); self.r[rd] = self.ld32(b, a, false); b.idle(1); }
                    2 => b.write8(rb.wrapping_add(imm), self.r[rd] & 0xFF, false),
                    _ => { self.r[rd] = b.read8(rb.wrapping_add(imm), false) & 0xFF; b.idle(1); }
                }
            }
            16 | 17 => {
                let rb = self.r[((op >> 3) & 7) as usize];
                let rd = (op & 7) as usize;
                let a = rb.wrapping_add(((op >> 6) & 31) << 1);
                if op & 0x0800 == 0 {
                    b.write16(a, self.r[rd] & 0xFFFF, false);
                } else {
                    self.r[rd] = self.ld16(b, a, false);
                    b.idle(1);
                }
            }
            18 | 19 => {
                let rd = ((op >> 8) & 7) as usize;
                let a = self.r[13].wrapping_add((op & 0xFF) << 2);
                if op & 0x0800 == 0 {
                    b.write32(a, self.r[rd], false);
                } else {
                    self.r[rd] = self.ld32(b, a, false);
                    b.idle(1);
                }
            }
            20 | 21 => {
                let rd = ((op >> 8) & 7) as usize;
                let base = if op & 0x0800 == 0 { self.r[15] & !3 } else { self.r[13] };
                self.r[rd] = base.wrapping_add((op & 0xFF) << 2);
            }
            22 | 23 => {
                if op & 0x0F00 == 0 {
                    let off = (op & 0x7F) << 2;
                    self.r[13] = if op & 0x80 != 0 { self.r[13].wrapping_sub(off) } else { self.r[13].wrapping_add(off) };
                } else if op & 0x0600 == 0x0400 {
                    self.thumb_pushpop(b, op);
                } else {
                    self.thumb_undef(b);
                }
            }
            24 | 25 => self.thumb_block(b, op),
            26 | 27 => {
                let c = (op >> 8) & 15;
                if c == 15 {
                    let lr = self.r[15].wrapping_sub(2);
                    self.exception(b, M_SVC, 0x08, lr);
                } else if c == 14 {
                    self.thumb_undef(b);
                } else if self.cond_ok(c) {
                    let off = (((op & 0xFF) << 24) as i32 >> 23) as u32;
                    let t = self.r[15].wrapping_add(off);
                    self.set_pc(b, t);
                }
            }
            28 => {
                let off = (((op & 0x7FF) << 21) as i32 >> 20) as u32;
                let t = self.r[15].wrapping_add(off);
                self.set_pc(b, t);
            }
            29 => self.thumb_undef(b),
            30 => {
                let off = (((op & 0x7FF) << 21) as i32 >> 9) as u32;
                self.r[14] = self.r[15].wrapping_add(off);
            }
            _ => {
                let t = self.r[14].wrapping_add((op & 0x7FF) << 1);
                self.r[14] = self.r[15].wrapping_sub(2) | 1;
                self.set_pc(b, t);
            }
        }
    }

    pub(super) fn thumb_alu<B: CpuBus>(&mut self, b: &mut B, op: u32) {
        let rd = (op & 7) as usize;
        let rs = self.r[((op >> 3) & 7) as usize];
        let a = self.r[rd];
        let cf = self.cpsr & C != 0;
        let cin = cf as u32;
        let logic = |s: &mut Self, r: u32| { s.r[rd] = r; s.set_nz(r); };
        match (op >> 6) & 15 {
            0 => logic(self, a & rs),
            1 => logic(self, a ^ rs),
            2 | 3 | 4 | 7 => {
                b.idle(1);
                let typ = match (op >> 6) & 15 { 2 => 0, 3 => 1, 4 => 2, _ => 3 };
                let (r, c) = shift_reg(typ, a, rs & 0xFF, cf);
                self.r[rd] = r;
                self.set_nz(r);
                self.set_c(c);
            }
            5 => self.r[rd] = self.adc(a, rs, cin, true),
            6 => self.r[rd] = self.adc(a, !rs, cin, true),
            8 => self.set_nz(a & rs),
            9 => self.r[rd] = self.adc(0, !rs, 1, true),
            10 => { self.adc(a, !rs, 1, true); }
            11 => { self.adc(a, rs, 0, true); }
            12 => logic(self, a | rs),
            13 => {
                b.idle(mul_cycles(a, true));
                logic(self, a.wrapping_mul(rs));
            }
            14 => logic(self, a & !rs),
            _ => logic(self, !rs),
        }
    }

    pub(super) fn thumb_hi<B: CpuBus>(&mut self, b: &mut B, op: u32) {
        let rd = ((op & 7) | ((op >> 4) & 8)) as usize;
        let rs = ((op >> 3) & 15) as usize;
        let s = self.r[rs];
        let d = self.r[rd];
        match (op >> 8) & 3 {
            0 => {
                let v = d.wrapping_add(s);
                if rd == 15 { self.set_pc(b, v) } else { self.r[rd] = v }
            }
            1 => { self.adc(d, !s, 1, true); }
            2 => {
                if rd == 15 { self.set_pc(b, s) } else { self.r[rd] = s }
            }
            _ => {
                if s & 1 == 0 { self.cpsr &= !FT; }
                self.set_pc(b, s);
            }
        }
    }

    pub(super) fn thumb_pushpop<B: CpuBus>(&mut self, b: &mut B, op: u32) {
        let list = op & 0xFF;
        let extra = op & 0x100 != 0;
        let n = list.count_ones() + extra as u32;
        let mut seq = false;
        if op & 0x0800 == 0 {
            // PUSH
            let mut a = self.r[13].wrapping_sub(4 * n);
            self.r[13] = a;
            for i in 0..8 {
                if list & (1 << i) != 0 {
                    b.write32(a, self.r[i], seq);
                    a = a.wrapping_add(4);
                    seq = true;
                }
            }
            if extra {
                b.write32(a, self.r[14], seq);
            }
        } else {
            let mut a = self.r[13];
            if extra { let d = b.peek32(self.r[13].wrapping_add(4 * list.count_ones()) & !3); self.waste_to(b, d); }
            for i in 0..8 {
                if list & (1 << i) != 0 {
                    self.r[i] = b.read32(a, seq);
                    a = a.wrapping_add(4);
                    seq = true;
                }
            }
            if extra {
                let v = b.read32(a, seq);
                a = a.wrapping_add(4);
                self.r[13] = a;
                b.idle(1);
                self.set_pc(b, v);
                return;
            }
            self.r[13] = a;
            b.idle(1);
        }
    }

    pub(super) fn thumb_block<B: CpuBus>(&mut self, b: &mut B, op: u32) {
        let rn = ((op >> 8) & 7) as usize;
        let l = op & 0x0800 != 0;
        let mut list = op & 0xFF;
        let empty = list == 0;
        let n = if empty { 16 } else { list.count_ones() };
        if empty { list = 1 << 15; }
        let base = self.r[rn];
        let wb = base.wrapping_add(4 * n);
        let mut a = base;
        let mut seq = false;
        if l {
            let mut pc_val = None;
            if list & (1 << 15) != 0 { self.waste(b); }
            for i in 0..16usize {
                if list & (1 << i) == 0 { continue; }
                let v = b.read32(a, seq);
                if i == 15 { pc_val = Some(v) } else { self.r[i] = v }
                a = a.wrapping_add(4);
                seq = true;
            }
            b.idle(1);
            if list & (1 << rn) == 0 { self.r[rn] = wb; }
            if let Some(v) = pc_val { self.set_pc(b, v); }
        } else {
            let mut first = true;
            for i in 0..16usize {
                if list & (1 << i) == 0 { continue; }
                let mut v = self.r[i];
                if i == rn && !first { v = wb; }
                if i == 15 { v = v.wrapping_add(2); }
                b.write32(a, v, seq);
                if first { self.r[rn] = wb; }
                first = false;
                a = a.wrapping_add(4);
                seq = true;
            }
        }
    }
}
