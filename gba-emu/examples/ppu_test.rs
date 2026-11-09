//! Synthetic PPU test: mode 0 tiled BG + sprites + alpha window, dumps PPM to argv[1].
#[path = "../src/ppu.rs"]
mod ppu;
use ppu::Ppu;

fn main() {
    let mut p = Ppu::new();
    // palette: 1=red 2=green 3=blue
    for (i, c) in [0x0000u16, 0x001F, 0x03E0, 0x7C00].iter().enumerate() { p.pal[i*2] = *c as u8; p.pal[i*2+1] = (*c >> 8) as u8; }
    for (i, c) in [0x0000u16, 0x03FF, 0x7FFF].iter().enumerate() { p.pal[0x200+i*2] = *c as u8; p.pal[0x201+i*2] = (*c >> 8) as u8; }
    // BG0 tile 1: checker of colors 1/2 in 4bpp, map at block 8, char base 0
    for r in 0..8 { for b in 0..4 { p.vram[32 + r*4 + b] = if (r/2)%2==0 { 0x21 } else { 0x12 }; } }
    for i in 0..32*32 { p.vram[0x4000 + i*2] = (i % 2) as u8; }
    // sprite tile 0 (obj base 0x10000): solid color 1, 32x32 at (100,50), 1D
    for i in 0..(16*32) { p.vram[0x10000 + i] = 0x11; }
    // semi-transparent sprite
    p.oam[0..8].copy_from_slice(&[50, 0x04, 100, 0x80, 0, 0, 0, 0]);
    p.oam[8..16].copy_from_slice(&[60, 0x04, 110, 0x80, 0, 0, 0, 0]);
    p.write_io(0x00, 0x1100 | 0x40, 0xFFFF); // mode0, BG0, OBJ, 1D
    p.write_io(0x08, 8 << 8, 0xFFFF);
    p.write_io(0x50, 0x0140 | 0x10, 0xFFFF);
    p.write_io(0x52, 0x0808, 0xFFFF);
    p.write_io(0x04, 0x0038 | (100 << 8), 0xFFFF);
    let mut irq = 0;
    for _ in 0..(228 * 1232) { let e = p.tick(1); irq |= e.irq; }
    println!("irq mask {irq}, center px {:08x}", p.fb[60 * 240 + 105]);
    assert_eq!(irq, 7, "vblank|hblank|vcount irqs");
    // mode 3, identity affine: pixel (3,2)=blue
    let mut q = Ppu::new();
    q.write_io(0x00, 0x0403, 0xFFFF);
    q.write_io(0x20, 0x100, 0xFFFF); q.write_io(0x26, 0x100, 0xFFFF);
    q.vram[(2*240+3)*2+1] = 0x7C;
    for _ in 0..(160*1232) { q.tick(1); }
    assert_eq!(q.fb[2*240+3], 0xFFFF0000);
    assert_eq!(q.fb[2*240+4], 0xFF000000);
    let mut s = String::from("P3\n240 160\n255\n");
    for px in &p.fb { s += &format!("{} {} {}\n", px & 255, (px >> 8) & 255, (px >> 16) & 255); }
    if let Some(f) = std::env::args().nth(1) { std::fs::write(f, s).unwrap(); }
}
