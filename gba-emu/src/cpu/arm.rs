use super::*;

impl Cpu {
    // ---------------------------------------------------------------- ARM
    pub(super) fn exec_arm<B: CpuBus>(&mut self, b: &mut B, op: u32) {
        match (op >> 25) & 7 {
            0 => {
                if op & 0x90 == 0x90 {
                    if op & 0x60 == 0 {
                        // multiply / swap
                        if op & 0x0100_0000 == 0 {
                            if op & 0x0080_0000 == 0 { self.arm_mul(b, op) } else { self.arm_mull(b, op) }
                        } else {
                            self.arm_swp(b, op)
                        }
                    } else {
                        self.arm_half(b, op)
                    }
                } else if op & 0x0FFF_FFF0 == 0x012F_FF10 {
                    let t = self.r[(op & 15) as usize];
                    if t & 1 != 0 { self.cpsr |= FT } else { self.cpsr &= !FT }
                    self.set_pc(b, t);
                } else if op & 0x0190_0000 == 0x0100_0000 {
                    self.arm_psr(b, op)
                } else {
                    self.arm_dp(b, op)
                }
            }
            1 => {
                if op & 0x0190_0000 == 0x0100_0000 {
                    self.arm_psr(b, op)
                } else {
                    self.arm_dp(b, op)
                }
            }
            2 => self.arm_sdt(b, op),
            3 => {
                if op & 0x10 != 0 { self.arm_undef(b) } else { self.arm_sdt(b, op) }
            }
            4 => self.arm_block(b, op),
            5 => {
                let off = (((op << 8) as i32) >> 6) as u32;
                if op & (1 << 24) != 0 {
                    self.r[14] = self.r[15].wrapping_sub(4);
                }
                let t = self.r[15].wrapping_add(off);
                self.set_pc(b, t);
            }
            6 => self.arm_undef(b),
            _ => {
                if op & (1 << 24) != 0 {
                    let lr = self.r[15].wrapping_sub(4);
                    self.exception(b, M_SVC, 0x08, lr);
                } else {
                    self.arm_undef(b)
                }
            }
        }
    }

    pub(super) fn arm_undef<B: CpuBus>(&mut self, b: &mut B) {
        let lr = self.r[15].wrapping_sub(4);
        self.exception(b, M_UND, 0x04, lr);
    }

    pub(super) fn arm_dp<B: CpuBus>(&mut self, b: &mut B, op: u32) {
        let opc = (op >> 21) & 15;
        let s = op & (1 << 20) != 0;
        let rn = ((op >> 16) & 15) as usize;
        let rd = ((op >> 12) & 15) as usize;
        let mut carry = self.cpsr & C != 0;
        let mut a = self.r[rn];
        let o2;
        if op & (1 << 25) != 0 {
            let rot = ((op >> 8) & 15) * 2;
            o2 = (op & 0xFF).rotate_right(rot);
            if rot != 0 {
                carry = o2 >> 31 != 0;
            }
        } else {
            let rm = (op & 15) as usize;
            let typ = (op >> 5) & 3;
            if op & 0x10 != 0 {
                b.idle(1);
                let amt = self.r[((op >> 8) & 15) as usize] & 0xFF;
                let mut v = self.r[rm];
                if rm == 15 { v = v.wrapping_add(4); }
                if rn == 15 { a = a.wrapping_add(4); }
                let (x, c) = shift_reg(typ, v, amt, carry);
                o2 = x;
                carry = c;
            } else {
                let (x, c) = shift_imm(typ, self.r[rm], (op >> 7) & 31, carry);
                o2 = x;
                carry = c;
            }
        }
        let wr = !(8..=11).contains(&opc);
        // flags are not touched by ALU when Rd=PC with S (CPSR <- SPSR instead)
        let sf = s && !(wr && rd == 15);
        let cin = (self.cpsr & C != 0) as u32;
        let res = match opc {
            0 | 8 => a & o2,
            1 | 9 => a ^ o2,
            2 | 10 => self.adc(a, !o2, 1, sf),
            3 => self.adc(o2, !a, 1, sf),
            4 | 11 => self.adc(a, o2, 0, sf),
            5 => self.adc(a, o2, cin, sf),
            6 => self.adc(a, !o2, cin, sf),
            7 => self.adc(o2, !a, cin, sf),
            12 => a | o2,
            13 => o2,
            14 => a & !o2,
            _ => !o2,
        };
        if sf && (opc < 2 || opc == 8 || opc == 9 || opc >= 12) {
            self.set_nz(res);
            self.set_c(carry);
        }
        if !wr && rd == 15 {
            self.restore_spsr(); // TSTP/TEQP/CMPP/CMNP
        }
        if wr {
            if rd == 15 {
                if s {
                    self.restore_spsr();
                }
                self.set_pc(b, res);
            } else {
                self.r[rd] = res;
            }
        }
    }

    pub(super) fn arm_psr<B: CpuBus>(&mut self, b: &mut B, op: u32) {
        let spsr = op & (1 << 22) != 0;
        if op & (1 << 21) == 0 {
            // MRS
            let rd = ((op >> 12) & 15) as usize;
            let bk = bank(self.cpsr);
            self.r[rd] = if spsr && bk != 0 { self.spsr[bk] } else { self.cpsr };
            return;
        }
        // MSR
        let v = if op & (1 << 25) != 0 {
            (op & 0xFF).rotate_right(((op >> 8) & 15) * 2)
        } else {
            self.r[(op & 15) as usize]
        };
        let mut mask = 0u32;
        if op & (1 << 19) != 0 { mask |= 0xFF00_0000; }
        if op & (1 << 18) != 0 { mask |= 0x00FF_0000; }
        if op & (1 << 17) != 0 { mask |= 0x0000_FF00; }
        if op & (1 << 16) != 0 { mask |= 0x0000_00FF; }
        let _ = b;
        if spsr {
            let bk = bank(self.cpsr);
            if bk != 0 {
                self.spsr[bk] = (self.spsr[bk] & !mask) | (v & mask);
            }
        } else {
            if self.cpsr & 0x1F == M_USR {
                mask &= 0xFF00_0000;
            }
            let n = (self.cpsr & !mask) | (v & mask);
            self.set_cpsr(n);
        }
    }

    pub(super) fn arm_mul<B: CpuBus>(&mut self, b: &mut B, op: u32) {
        let rd = ((op >> 16) & 15) as usize;
        let rn = ((op >> 12) & 15) as usize;
        let rs = self.r[((op >> 8) & 15) as usize];
        let rm = self.r[(op & 15) as usize];
        let mut res = rm.wrapping_mul(rs);
        let mut m = mul_cycles(rs, true);
        if op & (1 << 21) != 0 {
            res = res.wrapping_add(self.r[rn]);
            m += 1;
        }
        b.idle(m);
        self.r[rd] = res;
        if op & (1 << 20) != 0 {
            self.set_nz(res);
        }
    }

    pub(super) fn arm_mull<B: CpuBus>(&mut self, b: &mut B, op: u32) {
        let hi = ((op >> 16) & 15) as usize;
        let lo = ((op >> 12) & 15) as usize;
        let rs = self.r[((op >> 8) & 15) as usize];
        let rm = self.r[(op & 15) as usize];
        let signed = op & (1 << 22) != 0;
        let mut res: u64 = if signed {
            (rm as i32 as i64).wrapping_mul(rs as i32 as i64) as u64
        } else {
            (rm as u64) * (rs as u64)
        };
        let mut m = mul_cycles(rs, signed) + 1;
        if op & (1 << 21) != 0 {
            let acc = ((self.r[hi] as u64) << 32) | self.r[lo] as u64;
            res = res.wrapping_add(acc);
            m += 1;
        }
        b.idle(m);
        self.r[lo] = res as u32;
        self.r[hi] = (res >> 32) as u32;
        if op & (1 << 20) != 0 {
            self.cpsr = (self.cpsr & !(N | Z | C)) | ((res >> 32) as u32 & N) | if res == 0 { Z } else { 0 } | if mull_carry(rm, rs, signed) { C } else { 0 };
        }
    }

    pub(super) fn arm_swp<B: CpuBus>(&mut self, b: &mut B, op: u32) {
        let rn = ((op >> 16) & 15) as usize;
        let rd = ((op >> 12) & 15) as usize;
        let rm = self.r[(op & 15) as usize];
        let a = self.r[rn];
        let t;
        if op & (1 << 22) != 0 {
            t = b.read8(a, false) & 0xFF;
            b.write8(a, rm & 0xFF, false);
        } else {
            t = self.ld32(b, a, false);
            b.write32(a, rm, false);
        }
        b.idle(1);
        self.r[rd] = t;
    }

    pub(super) fn arm_half<B: CpuBus>(&mut self, b: &mut B, op: u32) {
        let p = op & (1 << 24) != 0;
        let u = op & (1 << 23) != 0;
        let w = op & (1 << 21) != 0;
        let l = op & (1 << 20) != 0;
        let rn = ((op >> 16) & 15) as usize;
        let rd = ((op >> 12) & 15) as usize;
        let sh = (op >> 5) & 3;
        if !l && sh != 1 {
            return self.arm_undef(b);
        }
        let off = if op & (1 << 22) != 0 {
            ((op >> 4) & 0xF0) | (op & 15)
        } else {
            self.r[(op & 15) as usize]
        };
        let base = self.r[rn];
        let moved = if u { base.wrapping_add(off) } else { base.wrapping_sub(off) };
        let addr = if p { moved } else { base };
        if l {
            if rd == 15 { self.waste(b); }
            let v = match sh {
                1 => self.ld16(b, addr, false),
                2 => self.ld8s(b, addr, false),
                _ => self.ld16s(b, addr, false),
            };
            b.idle(1);
            if w || !p { self.r[rn] = moved; }
            if rd == 15 { self.set_pc(b, v) } else { self.r[rd] = v }
        } else {
            let mut v = self.r[rd];
            if rd == 15 { v = v.wrapping_add(4); }
            b.write16(addr, v & 0xFFFF, false);
            if w || !p { self.r[rn] = moved; }
        }
    }

    pub(super) fn arm_sdt<B: CpuBus>(&mut self, b: &mut B, op: u32) {
        let p = op & (1 << 24) != 0;
        let u = op & (1 << 23) != 0;
        let byte = op & (1 << 22) != 0;
        let w = op & (1 << 21) != 0;
        let l = op & (1 << 20) != 0;
        let rn = ((op >> 16) & 15) as usize;
        let rd = ((op >> 12) & 15) as usize;
        let off = if op & (1 << 25) != 0 {
            let c = self.cpsr & C != 0;
            shift_imm((op >> 5) & 3, self.r[(op & 15) as usize], (op >> 7) & 31, c).0
        } else {
            op & 0xFFF
        };
        let base = self.r[rn];
        let moved = if u { base.wrapping_add(off) } else { base.wrapping_sub(off) };
        let addr = if p { moved } else { base };
        if l {
            if rd == 15 { let d = b.peek32(addr & !3).map(|w| w.rotate_right((addr & 3) * 8)); self.waste_to(b, d); }
            let v = if byte { b.read8(addr, false) & 0xFF } else { self.ld32(b, addr, false) };
            b.idle(1);
            if w || !p { self.r[rn] = moved; }
            if rd == 15 { self.set_pc(b, v) } else { self.r[rd] = v }
        } else {
            let mut v = self.r[rd];
            if rd == 15 { v = v.wrapping_add(4); }
            if byte { b.write8(addr, v & 0xFF, false) } else { b.write32(addr, v, false) }
            if w || !p { self.r[rn] = moved; }
        }
    }

    pub(super) fn arm_block<B: CpuBus>(&mut self, b: &mut B, op: u32) {
        let p = op & (1 << 24) != 0;
        let u = op & (1 << 23) != 0;
        let sbit = op & (1 << 22) != 0;
        let w = op & (1 << 21) != 0;
        let l = op & (1 << 20) != 0;
        let rn = ((op >> 16) & 15) as usize;
        let mut list = op & 0xFFFF;
        let empty = list == 0;
        let n = if empty { 16 } else { list.count_ones() };
        let base = self.r[rn];
        let low = match (u, p) {
            (true, false) => base,
            (true, true) => base.wrapping_add(4),
            (false, false) => base.wrapping_sub(4 * n).wrapping_add(4),
            (false, true) => base.wrapping_sub(4 * n),
        };
        let wb = if u { base.wrapping_add(4 * n) } else { base.wrapping_sub(4 * n) };
        if empty { list = 1 << 15; }
        let user = sbit && !(l && list & (1 << 15) != 0);
        let mut addr = low;
        let mut seq = false;
        let mut first = true;
        if l {
            if list & (1 << 15) != 0 { let d = b.peek32(low.wrapping_add(4 * (list.count_ones() - 1)) & !3); self.waste_to(b, d); }
            if w { self.r[rn] = wb; }
            let mut pc_val = None;
            for i in 0..16usize {
                if list & (1 << i) == 0 { continue; }
                let v = b.read32(addr, seq);
                if i == 15 { pc_val = Some(v); }
                else if user { self.set_user_reg(i, v) } else { self.r[i] = v }
                addr = addr.wrapping_add(4);
                seq = true;
            }
            b.idle(1);
            if let Some(v) = pc_val {
                if sbit { self.restore_spsr(); }
                self.set_pc(b, v);
            }
        } else {
            for i in 0..16usize {
                if list & (1 << i) == 0 { continue; }
                let mut v = if user { self.user_reg(i) } else { self.r[i] };
                if i == rn && w && !first { v = wb; }
                if i == 15 { v = v.wrapping_add(4); }
                b.write32(addr, v, seq);
                if first && w { self.r[rn] = wb; }
                first = false;
                addr = addr.wrapping_add(4);
                seq = true;
            }
        }
    }
}
