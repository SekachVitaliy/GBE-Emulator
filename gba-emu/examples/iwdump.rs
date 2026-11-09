// usage: iwdump rom replay N out.bin -> IWRAM after N frames
use gba_emu::emu::Emu;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let rom = std::fs::read(&a[1]).unwrap();
    let ev: Vec<(usize, u32)> = std::fs::read_to_string(&a[2]).unwrap().lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()).map(|l| { let mut w = l.split_whitespace(); (w.next().unwrap().parse().unwrap(), u32::from_str_radix(w.next().unwrap(), 16).unwrap()) }).collect();
    let n: usize = a[3].parse().unwrap();
    let mut m = Emu::new();
    m.bus.rom[..rom.len()].copy_from_slice(&rom);
    m.load_rom(rom.len());
    let (mut k, mut cur) = (0, 0);
    for f in 0..n { while k < ev.len() && ev[k].0 <= f { cur = ev[k].1; k += 1; } m.keys = cur; m.run_frame(); }
    let mut out = vec![];
    for i in 0..0x8000u32 { out.push(m.bus.read8(0x0300_0000 + i, false) as u8); }
    std::fs::write(&a[4], out).unwrap();
}
