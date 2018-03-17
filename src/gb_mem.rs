use std::fmt;

const ADDR_MAX: u16 = 0xFFFF;

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

    pub fn inc(&mut self, amt: u16) {
        self.val = self.val.wrapping_add(amt) & ADDR_MAX;
    }

    pub fn post_inc(&mut self, amt: u16) -> Self {
        let copy = *self;
        self.inc(amt);
        copy
    }

    pub fn dec(&mut self, val: u16) {
        self.val = self.val.wrapping_sub(val) & ADDR_MAX;
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

    // Will panic if addr is outside of the size
    pub fn read(&self, addr: RamAddress) -> u8 {
        self.ram[addr.get() as usize]
    }

    // Will panic if addr is outside of the size
    pub fn write(&mut self, addr: RamAddress, val: u8) {
        let idx = addr.get() as usize;

        self.ram[idx] = val;

        if idx < 0xDE00 && idx > 0xBFFF {
            self.ram[idx + 0x2000] = val;
        }

        if idx < 0xFE00 && idx > 0xDFFF {
            self.ram[idx - 0x2000] = val;
        }
    }
}
