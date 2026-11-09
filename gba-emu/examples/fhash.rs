// usage: fhash rom replay N [lag]  -> per-frame hash lines (keys delayed by lag frames)
use gba_emu::emu::Emu;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let rom = std::fs::read(&a[1]).unwrap();
    let ev: Vec<(usize, u32)> = std::fs::read_to_string(&a[2]).unwrap().lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()).map(|l| { let mut w = l.split_whitespace(); (w.next().unwrap().parse().unwrap(), u32::from_str_radix(w.next().unwrap(), 16).unwrap()) }).collect();
    let n: usize = a[3].parse().unwrap();
    let lag: usize = a.get(4).map_or(0, |s| s.parse().unwrap());
    let mut m = Emu::new();
    m.bus.rom[..rom.len()].copy_from_slice(&rom);
    m.load_rom(rom.len());
    let keys_at = |f: usize| ev.iter().filter(|e| e.0 <= f).last().map_or(0, |e| e.1);
    for f in 0..n { m.keys = if f >= lag { keys_at(f - lag) } else { 0 }; m.run_frame(); let mut h = 1469598103934665603u64; for &p in &m.fb { h = (h ^ p as u64).wrapping_mul(1099511628211); } println!("{h:016x}"); }
}
