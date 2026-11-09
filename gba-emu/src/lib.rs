//! Original GBA emulator core. C ABI per gba-eval/spec/ABI.md.
pub mod apu;
pub mod bus;
pub mod cpu;
pub mod emu;
pub mod ppu;
#[cfg(not(target_arch = "wasm32"))]
pub mod debug;
#[cfg(not(target_arch = "wasm32"))]
pub use debug::{dump, run, steps, trace};
use emu::Emu;

// Single global instance; the wasm module is single-threaded, so the unsynchronised cell is sound.
struct Global(std::cell::UnsafeCell<Option<Emu>>);
unsafe impl Sync for Global {}
static EMU: Global = Global(std::cell::UnsafeCell::new(None));

fn e() -> &'static mut Emu {
    // SAFETY: only ever called from the single wasm thread; the ABI never re-enters.
    unsafe { (*EMU.0.get()).get_or_insert_with(Emu::new) }
}

#[no_mangle] pub extern "C" fn emu_init() -> i32 { e(); 1 }
#[no_mangle] pub extern "C" fn emu_rom_buffer() -> *mut u8 { e().rom_ptr() }
#[no_mangle] pub extern "C" fn emu_load_rom(len: i32) -> i32 { e().load_rom(len as usize) as i32 }
#[no_mangle] pub extern "C" fn emu_reset() -> i32 { e().reset() as i32 }
#[no_mangle] pub extern "C" fn emu_set_keys(k: u32) { e().keys = k; }
#[no_mangle] pub extern "C" fn emu_run_frame() { e().run_frame(); }
#[no_mangle] pub extern "C" fn emu_framebuffer() -> *const u32 { e().fb.as_ptr() }
#[no_mangle] pub extern "C" fn emu_audio_buffer() -> *const i16 { e().audio.as_ptr() }
#[no_mangle] pub extern "C" fn emu_audio_samples() -> i32 { e().drain_audio() }
#[no_mangle] pub extern "C" fn emu_audio_rate() -> i32 { emu::AUDIO_RATE }
