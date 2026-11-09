// usage: ppuregs rom frames  -> prints display regs after N frames (keys 0)
use gba_emu::emu::Emu;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let rom = std::fs::read(&a[1]).unwrap();
    let n: usize = a[2].parse().unwrap();
    let mut m = Emu::new();
    m.bus.rom[..rom.len()].copy_from_slice(&rom);
    m.load_rom(rom.len());
    for _ in 0..n { m.run_frame(); }
    for o in (0..0x58).step_by(2) { print!("{:02x}:{:04x} ", o, m.bus.ppu.read_io(o)); if o % 16 == 14 { println!(); } }
    println!();
}
