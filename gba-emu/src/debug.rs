//! Native-only helpers used by examples/ (lockstep logs, reports); not part of the wasm build.
use crate::emu::Emu;

/// Native helper for examples/run.rs: run `frames` frames, write a PPM, print a report.
pub fn run(rom: &[u8], frames: usize, keys: &str, out: &str) {
    // keys: hex mask held for all frames, or a schedule "hexKEYxFRAMES,..." (e.g. "0x20,1x2,0x30")
    let sched: Vec<(u32, usize)> = if std::path::Path::new(keys).is_file() {
        // replay file: "<frame> <keyshex>" lines
        let ev: Vec<(usize, u32)> = std::fs::read_to_string(keys).unwrap().lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()).map(|l| { let mut w = l.split_whitespace(); (w.next().unwrap().parse().unwrap(), u32::from_str_radix(w.next().unwrap(), 16).unwrap()) }).collect();
        ev.iter().enumerate().map(|(i, &(f, k))| (k, ev.get(i + 1).map_or(usize::MAX, |n| n.0) .saturating_sub(f))).collect()
    } else if keys.contains('x') { keys.split(',').map(|t| { let (k, n) = t.split_once('x').unwrap(); (u32::from_str_radix(k, 16).unwrap_or(0), n.parse().unwrap()) }).collect() } else { vec![(u32::from_str_radix(keys, 16).unwrap_or(0), usize::MAX)] };
    let mut sched = sched.into_iter();
    let (mut cur, mut left) = sched.next().unwrap_or((0, usize::MAX));
    let mut m = Emu::new();
    m.bus.rom[..rom.len()].copy_from_slice(rom);
    assert!(m.load_rom(rom.len()));
    let mut samples = 0usize;
    let t = std::time::Instant::now();
    for _ in 0..frames { while left == 0 { (cur, left) = sched.next().unwrap_or((0, usize::MAX)); } left = left.saturating_sub(1); m.keys = cur; m.run_frame(); samples += m.drain_audio() as usize; }
    let mut ppm = format!("P6\n240 160\n255\n").into_bytes();
    let mut nz = std::collections::HashSet::new();
    for &p in &m.fb { ppm.extend_from_slice(&[p as u8, (p >> 8) as u8, (p >> 16) as u8]); nz.insert(p); }
    std::fs::write(out, ppm).unwrap();
    println!("frames={frames} time={:?} audio_pairs={samples} unique_colors={} cycles={} pc={:08x} cpsr={:08x} lr={:08x} save={}\n{}", t.elapsed(), nz.len(), m.bus.cycles, m.cpu.r[15], m.cpu.cpsr, m.cpu.r[14], m.bus.save.data.len(), m.bus.dbg());
}

pub fn trace(rom: &[u8], n: usize) {
    let mut m = Emu::new();
    m.bus.rom[..rom.len()].copy_from_slice(rom);
    m.load_rom(rom.len());
    let mut shown = false;
    for i in 0..n {
        if !shown && m.cpu.r[15] >= 0x0800_0000 { println!("reached ROM at step {i} cycles {} synced {}", m.bus.cycles, m.bus.synced_cycles()); shown = true; }
        println!("{i} pc={:08x} cpsr={:08x}", m.cpu.r[15], m.cpu.cpsr);
        m.cpu.step(&mut m.bus);
        m.bus.sync();
    }
}

pub fn dump(rom: &[u8], frames: usize) {
    let mut m = Emu::new();
    m.bus.rom[..rom.len()].copy_from_slice(rom);
    m.load_rom(rom.len());
    for _ in 0..frames { m.run_frame(); }
    let nz = m.bus.ppu.vram.iter().filter(|&&b| b != 0).count();
    let nzi: Vec<usize> = m.bus.ppu.vram.iter().enumerate().filter(|(_, &b)| b != 0).map(|(i, _)| i).take(4).collect(); println!("first nz vram idx {:?} bg2pa={:x} bg2pd={:x} bg2cnt={:x}", nzi, m.bus.ppu.read_io(0x20), m.bus.ppu.read_io(0x26), m.bus.ppu.read_io(0xC));
    println!("pal[0..8]={:?} vram_nonzero={nz} fbmax={:x}", &m.bus.ppu.pal[..8], m.fb.iter().max().unwrap());
}

/// Per-instruction log for lockstep comparison with the reference: "pc_after cycles_of_instr".
pub fn steps(rom: &[u8], n: usize) {
    let mut m = Emu::new();
    m.warmup_frames = 0;
    m.bus.rom[..rom.len()].copy_from_slice(rom);
    m.load_rom(rom.len());
    while m.cpu.r[15] < 0x0800_0000 { m.cpu.step(&mut m.bus); m.bus.sync(); }
    let mut out = String::new();
    m.bus.trace_mem = std::env::var("MEMLOG").is_ok();
    for _ in 0..n {
        let c0 = m.bus.cycles;
        m.bus.memlog.clear();
        if m.bus.halted() { m.bus.halt_skip(1); } else { m.cpu.step(&mut m.bus); m.bus.sync(); }
        out.push_str(&format!("{:x} {}", m.cpu.r[15], m.bus.cycles - c0));
        for (p, r, sz, w) in &m.bus.memlog { out.push_str(&format!(" @{} {} {} {}", p % 280896, r, sz, *w as u8)); }
        out.push('\n');
        if std::env::var("STEPDBG").is_ok() { eprintln!("  r15={:08x} cpsr={:08x} r0-3={:x?} lr={:x}", m.cpu.r[15], m.cpu.cpsr, &m.cpu.r[0..4], m.cpu.r[14]); }
    }
    print!("{out}");
    if let Ok(a) = std::env::var("DUMPAT") { let a = u32::from_str_radix(&a, 16).unwrap(); for i in 0..8 { let v = m.bus.read32(a + i * 4, false); eprintln!("{:08x}: {:08x}", a + i * 4, v); } eprintln!("r={:x?} cpsr={:x} cyc={} {}", m.cpu.r, m.cpu.cpsr, m.bus.cycles, m.bus.dbg()); }
}
