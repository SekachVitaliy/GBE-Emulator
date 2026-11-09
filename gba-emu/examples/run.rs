// usage: cargo run --release --example run -- rom.gba [frames] [out.ppm] [keys_hex]
use gba_emu as gba;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let rom = std::fs::read(&a[1]).expect("rom");
    let frames: usize = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(60);
    let out = a.get(3).cloned().unwrap_or("out.ppm".into());
    let keys = a.get(4).cloned().unwrap_or("0".into());
    gba::run(&rom, frames, &keys, &out);
}
