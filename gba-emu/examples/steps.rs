fn main() { let a: Vec<String> = std::env::args().collect(); gba_emu::steps(&std::fs::read(&a[1]).unwrap(), a[2].parse().unwrap()); }
