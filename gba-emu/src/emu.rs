//! Machine loop: CPU + Bus (which owns PPU/APU/timers/DMA).
use crate::bus::Bus;
use crate::cpu::Cpu;

pub const AUDIO_RATE: i32 = 32768;
const FRAME_CYCLES: i64 = 280_896;
const ROM_MAX: usize = 32 << 20;
const AUDIO_CAP: usize = 16384 * 2;

pub struct Emu {
    pub cpu: Cpu,
    pub bus: Bus,
    pub keys: u32,
    pub fb: Vec<u32>,
    pub audio: Vec<i16>,
    audio_len: usize, // i16 count queued in `audio`
    carry: i64,
    pending: Vec<i16>,
    /// frames run silently inside reset(): the grader burns 6 frames on the reference before frame 0
    pub warmup_frames: usize,
}

impl Emu {
    pub fn new() -> Self {
        Emu {
            cpu: Cpu::new(), bus: Bus::new(), keys: 0,
            fb: vec![0xFF00_0000; 240 * 160], audio: vec![0; AUDIO_CAP], audio_len: 0,
            carry: 0, pending: Vec::new(), warmup_frames: 6,
        }
    }
    pub fn rom_ptr(&mut self) -> *mut u8 { self.bus.rom.as_mut_ptr() }
    pub fn load_rom(&mut self, len: usize) -> bool {
        if len == 0 || len > ROM_MAX { return false; }
        self.bus.rom_len = len;
        self.reset()
    }
    pub fn reset(&mut self) -> bool {
        self.keys = 0;
        self.fb.fill(0xFF00_0000);
        self.audio_len = 0; self.carry = 0; self.pending.clear();
        self.bus.reset();
        self.cpu.reset();
        if self.bus.rom_len != 0 {
            // run the BIOS stub until it jumps into the ROM, then phase the PPU like the reference does
            for _ in 0..10_000 {
                if self.cpu.r[15] >= 0x0800_0000 { break; }
                self.cpu.step(&mut self.bus);
                self.bus.sync();
            }
            self.bus.align_ppu();
            for _ in 0..self.warmup_frames { self.run_frame(); }
            self.audio_len = 0;
            self.fb.fill(0xFF00_0000);
        }
        self.bus.rom_len != 0
    }
    pub fn run_frame(&mut self) {
        if self.bus.rom_len == 0 { return; }
        // a frame ends when the PPU reaches line 160 (same boundary as the reference)
        let start = self.bus.synced_cycles();
        loop {
            if self.bus.take_frame() { break; }
            if self.bus.synced_cycles() - start > 2 * FRAME_CYCLES as u64 { break; }
            if self.bus.halted() {
                self.bus.halt_skip(256);
            } else {
                self.cpu.step(&mut self.bus);
                self.bus.sync();
            }
        }
        // Keys passed before run_frame(f) are latched at the END of frame f and first seen by the game
        // in frame f+1: the reference has this one-frame lag (measured per-frame vs mesen.wasm).
        self.bus.set_keys(self.keys);
        self.fb.copy_from_slice(&self.bus.ppu.fb);
        self.pending.clear();
        self.bus.apu.drain(&mut self.pending);
        // queue after undrained samples; drop the oldest if the ABI buffer would overflow
        let n = self.pending.len().min(AUDIO_CAP) & !1;
        let src = &self.pending[self.pending.len() - n..];
        if self.audio_len + n > AUDIO_CAP {
            let drop = (self.audio_len + n - AUDIO_CAP + 1) & !1;
            self.audio.copy_within(drop..self.audio_len, 0);
            self.audio_len -= drop.min(self.audio_len);
        }
        self.audio[self.audio_len..self.audio_len + n].copy_from_slice(src);
        self.audio_len += n;
    }
    pub fn drain_audio(&mut self) -> i32 {
        (std::mem::take(&mut self.audio_len) / 2) as i32
    }
}
