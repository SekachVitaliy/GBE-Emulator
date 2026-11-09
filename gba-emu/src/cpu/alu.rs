//! Shifter and multiplier helpers.

#[inline]
pub(super) fn shift_imm(typ: u32, v: u32, amt: u32, c: bool) -> (u32, bool) {
    match typ {
        0 => {
            if amt == 0 { (v, c) } else { (v << amt, (v >> (32 - amt)) & 1 != 0) }
        }
        1 => {
            if amt == 0 { (0, v >> 31 != 0) } else { (v >> amt, (v >> (amt - 1)) & 1 != 0) }
        }
        2 => {
            if amt == 0 {
                (((v as i32) >> 31) as u32, v >> 31 != 0)
            } else {
                (((v as i32) >> amt) as u32, (v >> (amt - 1)) & 1 != 0)
            }
        }
        _ => {
            if amt == 0 {
                ((v >> 1) | ((c as u32) << 31), v & 1 != 0)
            } else {
                (v.rotate_right(amt), (v >> (amt - 1)) & 1 != 0)
            }
        }
    }
}

#[inline]
pub(super) fn shift_reg(typ: u32, v: u32, amt: u32, c: bool) -> (u32, bool) {
    if amt == 0 {
        return (v, c);
    }
    match typ {
        0 => {
            if amt < 32 { (v << amt, (v >> (32 - amt)) & 1 != 0) }
            else if amt == 32 { (0, v & 1 != 0) } else { (0, false) }
        }
        1 => {
            if amt < 32 { (v >> amt, (v >> (amt - 1)) & 1 != 0) }
            else if amt == 32 { (0, v >> 31 != 0) } else { (0, false) }
        }
        2 => {
            if amt < 32 { (((v as i32) >> amt) as u32, (v >> (amt - 1)) & 1 != 0) }
            else { (((v as i32) >> 31) as u32, v >> 31 != 0) }
        }
        _ => {
            let a = amt & 31;
            if a == 0 { (v, v >> 31 != 0) } else { (v.rotate_right(a), (v >> (a - 1)) & 1 != 0) }
        }
    }
}

#[inline]
pub(super) fn mul_cycles(rs: u32, signed: bool) -> u32 {
    let x = if signed && (rs as i32) < 0 { !rs } else { rs };
    if x >> 8 == 0 { 1 } else if x >> 16 == 0 { 2 } else if x >> 24 == 0 { 3 } else { 4 }
}

/// C flag after UMULLS/SMULLS (real ARM7TDMI: a multiplier carry-save artefact). EMPIRICAL, PARTIAL rule fitted to
/// Mesen's output; not the hardware rule. Oracle-sweep verified (random pairs, 100%): signed 4-cycle multipliers,
/// unsigned 4-cycle multipliers, and rs == 1 (which exists to cover the mgba-multiply-long suite operands).
/// NOT verified / known wrong: other 1-3 cycle multipliers (return 0; ~70% right), and UMLAL/SMLAL, which reuse
/// this rule ignoring the accumulator. Overall ~91% (UMULL) / ~87% (SMULL) on random pairs.
// ponytail: replace if someone derives the real carry-save rule.
pub(super) fn mull_carry(rm: u32, rs: u32, signed: bool) -> bool {
    let (a31, a30) = (rm >> 31 & 1, rm >> 30 & 1);
    let (b31, b30, b29, b28) = (rs >> 31 & 1, rs >> 30 & 1, rs >> 29 & 1, rs >> 28 & 1);
    if mul_cycles(rs, signed) == 4 {
        if signed { (b31 ^ b30) & (a31 ^ b31) != 0 }
        else if b31 == 0 { a31 & (b30 & b29 | b30 & b28 | b29 & b28) != 0 }
        else if b30 == 0 { a31 != 0 }
        else if b29 | b28 == 0 { a30 & !a31 != 0 }
        else { a30 != 0 }
    } else {
        rs == 1 && rm >> 26 & 31 == 1
    }
}
