use gba_emu::*;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let rom = std::fs::read(&a[1]).unwrap();
    let n: usize = a[2].parse().unwrap();
    trace(&rom, n);
}
