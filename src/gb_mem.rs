use std::fmt;

use gb_rom::GbRom;

const ADDR_MAX: u16 = 0xFFFF;

pub const IE_ADDR: RamAddress = RamAddress { val: 0xFFFFu16 };
pub const IF_ADDR: RamAddress = RamAddress { val: 0xFF0Fu16 };

pub const SB_ADDR: RamAddress = RamAddress { val: 0xFF01u16 };
pub const SC_ADDR: RamAddress = RamAddress { val: 0xFF02u16 };

pub fn increment_16(high: &mut u8, low: &mut u8) {
    // does not affect flags
    let over_low = (*low).overflowing_add(1);
    *low = over_low.0;

    if over_low.1 {
        *high = (*high).wrapping_add(1);
    }
}

pub fn decrement_16(high: &mut u8, low: &mut u8) {
    // does not affect flags
    let over_low = (*low).overflowing_sub(1);
    *low = over_low.0;

    if over_low.1 {
        *high = (*high).wrapping_sub(1);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RamAddress {
    val: u16,
}

impl RamAddress {
    pub fn new(init: u16) -> Self {
        RamAddress { val: init }
    }

    pub fn get(&self) -> u16 {
        self.val
    }

    pub fn set(&mut self, val: u16) {
        self.val = val & ADDR_MAX;
    }

    pub fn inc(&mut self, amt: u16) -> Self {
        self.val = self.val.wrapping_add(amt) & ADDR_MAX;
        *self
    }

    pub fn post_inc(&mut self, amt: u16) -> Self {
        let copy = *self;
        self.inc(amt);
        copy
    }

    pub fn dec(&mut self, val: u16) -> Self {
        self.val = self.val.wrapping_sub(val) & ADDR_MAX;
        *self
    }

    pub fn post_dec(&mut self, val: u16) -> Self {
        let copy = *self;
        self.dec(val);
        copy
    }
}

#[test]
fn post_inc_test() {
    let mut ra = RamAddress::new(10);

    assert!(ra.get() == 10);
    let ra2 = ra.post_inc(1);
    assert!(ra2.get() == 10);
    assert!(ra.get() == 11);
}

#[test]
fn post_dec_test() {
    let mut ra = RamAddress::new(10);

    assert!(ra.get() == 10);
    let ra2 = ra.post_dec(1);
    assert!(ra2.get() == 10);
    assert!(ra.get() == 9)
}

enum MemorySection {
    RestartInterrupts = 0x0000,
    Header = 0x0100,
    RomBank0 = 0x0150,
    RomBankN = 0x4000,
    VRam = 0x8000,
    ExternalRam = 0xA000,
    WorkRam0 = 0xC000,
    WorkRamN = 0xD000,
    Echo = 0xE000,
    SpriteAttribute = 0xFE00,
    Unusable = 0xFEA0,
    IORegisters = 0xFF00,
    HighRam = 0xFF80,
    IERegister = 0xFFFF,
}

pub struct MemoryController {
    rom: GbRom,
    ram: [u8; 0x10000], //65536 bytes
}

impl fmt::Debug for MemoryController {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MemoryController")
    }
}

impl MemoryController {
    pub fn new(rom: GbRom) -> Self {
        let mut mc = MemoryController {
            rom: rom,
            ram: [0u8; 0x10000],
        };

        {
            let mut dest = &mut mc.ram[0x000..0x8000];
            mc.rom.copy_current_slice(dest);
        }

        mc
    }

    // Will panic if addr is outside of the size
    pub fn read(&self, addr: RamAddress) -> u8 {
        self.ram[addr.get() as usize]
        //self.rom.read_address(addr)
    }

    // Will panic if addr is outside of the size
    pub fn write(&mut self, addr: RamAddress, val: u8) {
        let idx = addr.get() as usize;

        match idx {
            0x0000...0x00FF => {
                println!("ERROR: trying to write to ROM bank 0: {}", idx);
                // send this on to the ROM as it may cause a bank switch
                return;
            }
            0x4000...0x7FFF => {
                println!("ERROR: trying to write to switchable ROM bank: {}", idx);
                // MAYBE send this to the ROM as well...?
                return;
            }
            0x8000...0x9FFF => {
                // Video RAM...
            }
            0xA000...0xBFFF => {
                // Switchable RAM bank... (on cartridge, if available)
            }
            0xC000...0xDFFF => {
                // Internal RAM
                // 0xD000...0xDFFF is switchable on CGB
                self.ram[idx + 0x2000] = val;
            }
            0xE000...0xFDFF => {
                // mirror RAM -- probly shouldn't use...?
                self.ram[idx - 0x2000] = val;
            }
            0xFE00...0xFE9F => {
                //OAM -- Object attribute memory
            }
            0xFEA0...0xFEFF => {
                // Unusable Memory
                println!("ERROR: trying to write to unusable memory: {}", idx);
                return;
            }
            0xFF00...0xFF7F => {
                // I/O ports
            }
            0xFF80...0xFFFE => {
                // High RAM
            }
            0xFFFF => {
                // Interrupt enable register
            }
            _ => {
                println!("WARNING: Unsupported memory write to {}", idx);
            }
        };

        self.ram[idx] = val;
    }
}
