//! Core unit tests.
use super::*;
struct M { m: Vec<u8>, cyc: u32 }
impl CpuBus for M {
    fn read8(&mut self, a: u32, _: bool) -> u32 { self.cyc += 1; self.m[a as usize] as u32 }
    fn read16(&mut self, a: u32, _: bool) -> u32 { self.cyc += 1; let a = (a & !1) as usize; u16::from_le_bytes([self.m[a], self.m[a + 1]]) as u32 }
    fn read32(&mut self, a: u32, _: bool) -> u32 { self.cyc += 1; let a = (a & !3) as usize; u32::from_le_bytes(self.m[a..a + 4].try_into().unwrap()) }
    fn write8(&mut self, a: u32, v: u32, _: bool) { self.m[a as usize] = v as u8 }
    fn write16(&mut self, a: u32, v: u32, _: bool) { let a = (a & !1) as usize; self.m[a..a + 2].copy_from_slice(&(v as u16).to_le_bytes()) }
    fn write32(&mut self, a: u32, v: u32, _: bool) { let a = (a & !3) as usize; self.m[a..a + 4].copy_from_slice(&v.to_le_bytes()) }
    fn idle(&mut self, c: u32) { self.cyc += c }
    fn irq_line(&self) -> bool { false }
    fn halted(&self) -> bool { false }
}
fn run(code: &[u32], n: usize) -> Cpu {
    let mut m = M { m: vec![0; 0x10000], cyc: 0 };
    for (i, w) in code.iter().enumerate() { m.m[i * 4..i * 4 + 4].copy_from_slice(&w.to_le_bytes()); }
    let mut c = Cpu::new();
    c.cpsr = 0x1F;
    for _ in 0..n { c.step_g(&mut m); }
    c
}
/// Guard: `impl CpuBus for Bus` must route opcode fetches through Bus::fetch* (prefetch-buffer model);
/// with the trait defaults they would degrade to plain reads, which cost differently.
#[test]
fn cpubus_fetch_uses_prefetch() {
    let run = |fetch: bool| {
        let mut b = Bus::new();
        b.rom_len = 0x1000;
        b.write16(0x0400_0204, 0x4000, false);
        let c0 = b.cycles;
        for k in 0..8u32 {
            let a = 0x0800_0000 + k * 4;
            if fetch { CpuBus::fetch32(&mut b, a, k != 0); } else { CpuBus::read32(&mut b, a, k != 0); }
            CpuBus::idle(&mut b, 6);
        }
        b.cycles - c0
    };
    assert_ne!(run(true), run(false), "CpuBus::fetch32 is not using the prefetch path");
}

#[test]
fn arm_basic() {
    // mov r0,#5 ; adds r1,r0,#-? ; subs r2,r0,r0 ; ldr via pc; bl
    let c = run(&[0xE3A00005, 0xE0901000, 0xE0502000 , 0xE1A03F10 /*lsl r3,r0,pc? */], 3);
    assert_eq!(c.r[0], 5);
    assert_eq!(c.r[1], 10);
    assert_eq!(c.r[2], 0);
    assert!(c.cpsr & (Z | C) == Z | C);
}
#[test]
fn thumb_and_bx() {
    // add r0,pc,#1 ; bx r0 (thumb at 8) ; thumb: mov r1,#7 ; mov r2,#9
    let c = run(&[0xE28F0001, 0xE12FFF10, 0x2209_2107], 4);
    assert!(c.cpsr & FT != 0);
    assert_eq!((c.r[1], c.r[2]), (7, 9));
}
#[test]
fn mode_banks() {
    // mov r13,#1; msr cpsr,#0xD2 (irq); mov r13,#2; msr cpsr,#0x1F; 
    let c = run(&[0xE3A0D001, 0xE321F0D2, 0xE3A0D002, 0xE321F01F], 4);
    assert_eq!(c.r[13], 1);
    assert_eq!(c.bank_sp[2], 2);
}
