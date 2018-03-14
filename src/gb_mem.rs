use std::fmt;

pub struct MemoryController {
    ram: [u8; 0x10000], //65536 bytes
}

impl fmt::Debug for MemoryController {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MemoryController")
    }
}

impl MemoryController {
    pub fn new() -> Self {
        MemoryController {
            ram: [0u8; 0x10000],
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        self.ram[addr as usize] = val;

        if addr < 0xDE00 && addr > 0xBFFF {
            self.ram[addr as usize + 0x2000] = val;
        }

        if addr < 0xFE00 && addr > 0xDFFF {
            self.ram[addr as usize - 0x2000] = val;
        }
    }
}
