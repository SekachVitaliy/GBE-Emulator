//! Cartridge backup memory: SRAM, Flash 64K/128K, EEPROM 512B/8K.
#[derive(Clone, Copy, PartialEq)]
pub enum Kind { Sram, Flash64, Flash128, Eeprom }

pub struct Save {
    pub kind: Kind,
    pub data: Vec<u8>,
    // flash command state
    fstate: u8, id_mode: bool, erase: bool, write_next: bool, bank_next: bool, bank: usize,
    // eeprom state
    abits: usize, // 0 = unknown (set by DMA length hint)
    bits: Vec<u8>,
    out: Vec<u8>, out_pos: usize,
}

/// One pass over the ROM for all save-type markers (a 32 MiB ROM was scanned up to 5 times before).
fn scan(rom: &[u8]) -> [bool; 4] {
    let mut f = [false; 4]; // FLASH1M_V, FLASH512_V|FLASH_V, EEPROM_V, (unused)
    for i in 0..rom.len().saturating_sub(8) {
        match rom[i] {
            b'F' if rom[i..].starts_with(b"FLASH") => {
                let r = &rom[i + 5..];
                if r.starts_with(b"1M_V") { f[0] = true } else if r.starts_with(b"512_V") || r.starts_with(b"_V") { f[1] = true }
            }
            b'E' if rom[i..].starts_with(b"EEPROM_V") => f[2] = true,
            _ => {}
        }
        if f[0] { break; }
    }
    f
}

impl Save {
    pub fn detect(rom: &[u8]) -> Save {
        let f = scan(rom);
        let kind = if f[0] { Kind::Flash128 } else if f[1] { Kind::Flash64 } else if f[2] { Kind::Eeprom } else { Kind::Sram }; // SRAM_V or unknown: plain 32K SRAM is the safest default
        Save::new(kind)
    }
    pub fn new(kind: Kind) -> Save {
        let (n, fill) = match kind { Kind::Sram => (0x8000, 0xFF), Kind::Flash64 => (0x10000, 0xFF), Kind::Flash128 => (0x20000, 0xFF), Kind::Eeprom => (0x2000, 0xFF) };
        Save { kind, data: vec![fill; n], fstate: 0, id_mode: false, erase: false, write_next: false, bank_next: false, bank: 0, abits: 0, bits: vec![], out: vec![], out_pos: 0 }
    }

    /// 8-bit read from 0x0E000000 region (offset already masked to 16 bits).
    pub fn read(&self, off: u32) -> u8 {
        match self.kind {
            Kind::Sram => self.data[(off & 0x7FFF) as usize],
            Kind::Flash64 | Kind::Flash128 => {
                if self.id_mode && off < 2 {
                    let id: [u8; 2] = if self.kind == Kind::Flash128 { [0x62, 0x13] } else { [0x32, 0x1B] };
                    return id[off as usize];
                }
                self.data[self.bank * 0x10000 + off as usize]
            }
            Kind::Eeprom => 0xFF,
        }
    }

    pub fn write(&mut self, off: u32, v: u8) {
        match self.kind {
            Kind::Sram => self.data[(off & 0x7FFF) as usize] = v,
            Kind::Flash64 | Kind::Flash128 => self.flash_write(off, v),
            Kind::Eeprom => {}
        }
    }

    fn flash_write(&mut self, off: u32, v: u8) {
        let base = self.bank * 0x10000;
        if self.write_next { self.data[base + off as usize] &= v; self.write_next = false; self.fstate = 0; return; }
        if self.bank_next && off == 0 { self.bank = (v & 1) as usize; self.bank_next = false; self.fstate = 0; return; }
        match (self.fstate, off, v) {
            (0, 0x5555, 0xAA) => self.fstate = 1,
            (1, 0x2AAA, 0x55) => self.fstate = 2,
            (2, 0x5555, cmd) => {
                self.fstate = 0;
                match cmd {
                    0x90 => self.id_mode = true,
                    0xF0 => self.id_mode = false,
                    0x80 => self.erase = true,
                    0xA0 => self.write_next = true,
                    0xB0 if self.kind == Kind::Flash128 => self.bank_next = true,
                    0x10 if self.erase => { self.data.fill(0xFF); self.erase = false; }
                    _ => {}
                }
            }
            (2, _, 0x30) if self.erase => {
                let s = base + (off & 0xF000) as usize;
                self.data[s..s + 0x1000].fill(0xFF);
                self.erase = false; self.fstate = 0;
            }
            (_, _, 0xF0) => { self.id_mode = false; self.fstate = 0; }
            _ => self.fstate = 0,
        }
    }

    /// DMA length hint: 9/73 transfers mean 6-bit addressing, 17/81 mean 14-bit.
    pub fn eeprom_hint(&mut self, count: u32) {
        if self.kind == Kind::Eeprom && self.abits == 0 {
            match count { 9 | 73 => self.abits = 6, 17 | 81 => self.abits = 14, _ => {} }
        }
    }

    pub fn eeprom_read(&mut self) -> u16 {
        if self.out_pos < self.out.len() { let b = self.out[self.out_pos]; self.out_pos += 1; b as u16 } else { 1 }
    }

    pub fn eeprom_write(&mut self, bit: u8) {
        if self.kind != Kind::Eeprom { return; }
        self.bits.push(bit & 1);
        let ab = if self.abits == 0 { 6 } else { self.abits };
        let n = self.bits.len();
        if n < 2 + ab + 1 { return; }
        let addr = self.bits[2..2 + ab].iter().fold(0usize, |a, &b| (a << 1) | b as usize);
        let base = (addr * 8) % self.data.len();
        if self.bits[0] == 1 && self.bits[1] == 1 && n == 2 + ab + 1 {
            // read request
            self.out = vec![0; 4];
            for i in 0..64 { self.out.push((self.data[(base + i / 8) % self.data.len()] >> (7 - i % 8)) & 1); }
            self.out_pos = 0;
            self.bits.clear();
        } else if self.bits[0] == 1 && self.bits[1] == 0 && n == 2 + ab + 64 + 1 {
            for i in 0..64 {
                let (byte, bit) = (base + i / 8, 7 - i % 8);
                let l = self.data.len();
                let d = &mut self.data[byte % l];
                *d = (*d & !(1 << bit)) | (self.bits[2 + ab + i] << bit);
            }
            self.bits.clear();
            self.out.clear(); self.out_pos = 0;
        } else if n > 2 + ab + 64 + 1 || (self.bits[0] == 0) { self.bits.clear(); }
    }
}
