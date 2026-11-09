//! ABI / determinism / golden-frame tests. Native, no mesen.wasm; ROM-based tests skip if the corpus is absent.
use gba_emu::emu::Emu;

fn rom(rel: &str) -> Option<Vec<u8>> {
    std::fs::read(format!("{}/../gba-eval/corpus/roms/{rel}", env!("CARGO_MANIFEST_DIR"))).ok()
}
fn fnv(h: &mut u64, b: &[u8]) { for &x in b { *h = (*h ^ x as u64).wrapping_mul(0x100_0000_01b3); } }
fn boot(r: &[u8]) -> Emu {
    let mut m = Emu::new();
    m.bus.rom[..r.len()].copy_from_slice(r);
    assert!(m.load_rom(r.len()));
    m
}
/// (framebuffer hash, audio hash, total audio pairs) over `n` frames with `keys` held.
fn run(m: &mut Emu, n: usize, keys: u32) -> (u64, u64, usize) {
    let (mut fh, mut ah, mut pairs) = (0xcbf2_9ce4_8422_2325u64, 0xcbf2_9ce4_8422_2325u64, 0);
    for _ in 0..n {
        m.keys = keys;
        m.run_frame();
        let fb: Vec<u8> = m.fb.iter().flat_map(|p| p.to_le_bytes()).collect();
        fnv(&mut fh, &fb);
        let k = m.drain_audio() as usize;
        pairs += k;
        let au: Vec<u8> = m.audio[..k * 2].iter().flat_map(|s| s.to_le_bytes()).collect();
        fnv(&mut ah, &au);
    }
    (fh, ah, pairs)
}

macro_rules! need { ($p:expr) => { match rom($p) { Some(r) => r, None => { eprintln!("skip: {} missing", $p); return; } } }; }

fn load_vs_reset(path: &str, frames: usize) {
    let r = need!(path);
    let mut m = boot(&r);
    let a = run(&mut m, frames, 0);
    assert!(m.reset());
    let b = run(&mut m, frames, 0);
    assert_eq!(a, b, "{path}: reset() must reproduce load_rom() exactly");
}
#[test] fn reset_matches_load_armwrestler() { load_vs_reset("test/armwrestler.gba", 60); }
// Regression test: Bus::reset must clear all DMA/prefetch state (fixed; kept as a guard).
#[test] fn reset_matches_load_bulletgba() { load_vs_reset("homebrew/bulletgba.gba", 120); }

#[test] fn audio_pairs_per_frame_soundbias_res0_2() { bias_pairs(0..3); }
// res 3 = 4389 pairs/frame but emu.rs AUDIO_CAP holds 4096: samples are truncated (emu.rs/bus-agent-r3).
#[test] fn audio_pairs_per_frame_soundbias_res3() { bias_pairs(3..4); }
fn bias_pairs(rs: std::ops::Range<u32>) {
    let r = need!("test/armwrestler.gba");
    for res in rs {
        let mut m = boot(&r);
        m.bus.write16(0x0400_0088, (res << 14) | 0x200, false);
        m.run_frame(); m.drain_audio();
        let frames = 8;
        let (_, _, pairs) = run(&mut m, frames, 0);
        let want = 280_896.0 / (512 >> res) as f64 * frames as f64;
        assert!((pairs as f64 - want).abs() < 4.0 * frames as f64, "res {res}: {pairs} pairs, want ~{want}");
    }
}

#[test]
fn padded_roms_are_equivalent() {
    let r = need!("test/armwrestler.gba");
    let base = run(&mut boot(&r), 30, 0);
    let mut p = r.clone(); p.push(0);
    assert_eq!(run(&mut boot(&p), 30, 0), base, "ROM+1 byte");
    let mut big = r.clone(); big.resize(32 << 20, 0);
    assert_eq!(run(&mut boot(&big), 30, 0), base, "32 MiB padded");
}

#[test]
fn bad_rom_lengths_fail_cleanly() {
    let mut m = Emu::new();
    assert!(!m.load_rom(0));
    assert!(!m.load_rom((32 << 20) + 1));
    assert!(!m.load_rom(usize::MAX));
    m.run_frame(); // no ROM: must not trap
}

// Golden hashes (fb, audio) so silent behaviour changes fail loudly. Regenerate with GOLDEN=1 cargo test --release --test abi -- --nocapture.
fn golden(path: &str, frames: usize, want: Option<(u64, u64)>) {
    let r = need!(path);
    let (f, a, _) = run(&mut boot(&r), frames, 0);
    if std::env::var("GOLDEN").is_ok() { println!("GOLDEN {path} {frames} ({f:#018x}, {a:#018x})"); return; }
    assert_eq!(Some((f, a)), want, "{path}: golden frame/audio hash changed");
}
#[test] fn golden_armwrestler() { golden("test/armwrestler.gba", 90, GOLD[0]); }
#[test] fn golden_jsmolka_memory() { golden("test/jsmolka/memory.gba", 90, GOLD[1]); }
#[test] fn golden_mgba_suite() { golden("test/mgba-suite.gba", 120, GOLD[2]); }
const GOLD: [Option<(u64, u64)>; 3] = [
    Some((0x7f21e991368ab52f, 0x2f4d19dd857da925)),
    Some((0xb889dcbb09df37a5, 0x2f4d19dd857da925)),
    Some((0x82d19597b269473d, 0x13caf73acb55e915)),
];
