use std::rc::Rc;
use gb_mem::MemoryController;
use gb_opcodes as OpCodes;

const ZERO_FLAG: u8 = 1 << 7;
const SUBT_FLAG: u8 = 1 << 6;
const HALF_CARRY_FLAG: u8 = 1 << 5;
const CARRY_FLAG: u8 = 1 << 4;

const VBLANK_IF: u8 = 1;
const LCDC_IF: u8 = 1 << 1;
const TIMER_OVERFLOW_IF: u8 = 1 << 2;
const SERIAL_IO_COMPLETE_IF: u8 = 1 << 3;
const P10_P13_TERM_NEG_EDGE_IF: u8 = 1 << 4;

#[derive(Debug)]
pub struct GbCpu {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,

    sp: u16,
    pc: u16,

    halt: bool,
    stop: bool,

    mc: Rc<MemoryController>,
}

impl GbCpu {
    pub fn new() -> Self {
        GbCpu {
            a: 0u8,
            b: 0u8,
            c: 0u8,
            d: 0u8,
            e: 0u8,
            f: 0u8,
            h: 0u8,
            l: 0u8,
            sp: 0u16,
            pc: 0u16,
            halt: false,
            stop: false,
            mc: Rc::new(MemoryController::new()),
        }
    }

    pub fn get_ram(&self) -> Rc<MemoryController> {
        self.mc.clone()
    }

    fn get_carry(&self) -> u8 {
        if (self.f & CARRY_FLAG) == CARRY_FLAG {
            1u8
        } else {
            0u8
        }
    }

    // Creating addresses by combining registers (&c)
    fn make_bc_address(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }

    fn make_de_address(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }

    fn make_hl_address(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }

    fn make_ffc_address(&self, n: u8) -> u16 {
        0xFF00 | n as u16
    }

    fn make_ffn_address(&mut self) -> u16 {
        let mc = &self.mc;
        let high = self.pc;
        let low = self.pc + 1;
        self.pc += 2;
        (mc.read(high) as u16) | (mc.read(low) as u16) << 8
    }

    // Setting the flag helpers
    fn set_flag(&mut self, mask: u8) {
        self.f |= mask;
    }

    fn reset_flag(&mut self, mask: u8) {
        self.f &= !mask;
    }

    fn set_flag_conditional(&mut self, mask: u8, test: bool) {
        if test {
            self.set_flag(mask);
        } else {
            self.reset_flag(mask);
        }
    }

    fn add(&mut self, a: u8, b: u8) -> u8 {
        let r = self.add_no_zcheck(a, b);
        self.set_flag_conditional(ZERO_FLAG, r as u8 == 0);
        r
    }

    fn add_no_zcheck(&mut self, a: u8, b: u8) -> u8 {
        let r = a + b;
        let hr = (a & 0x0F) + (b & 0x0F);

        self.set_flag_conditional(HALF_CARRY_FLAG, hr > 0x0F);
        self.set_flag_conditional(CARRY_FLAG, r < a); // it wrapped around
        self.reset_flag(SUBT_FLAG);
        r
    }

    fn add_with_carry(&mut self, a: u8, b: u8) -> u8 {
        let carry = self.get_carry();
        let t = a + carry;
        self.add(t, b)
    }

    fn subtract(&mut self, a: u8, b: u8) -> u8 {
        let r = self.subtract_no_zcheck(a, b);
        self.set_flag_conditional(ZERO_FLAG, r == 0);
        r
    }

    fn subtract_no_zcheck(&mut self, a: u8, b: u8) -> u8 {
        let r = a - b;
        let hr = (a & 0x0F) - (b & 0x0F);

        self.set_flag_conditional(HALF_CARRY_FLAG, hr > 0x0F);
        self.set_flag_conditional(CARRY_FLAG, r > a); // it wrapped around
        self.set_flag(SUBT_FLAG);
        r
    }

    fn subtract_with_carry(&mut self, a: u8, b: u8) -> u8 {
        let carry = self.get_carry();
        let n = b + carry;
        self.subtract(a, n)
    }

    fn set_logic_flags(&mut self, result: u8, set_half_carry: bool) {
        self.f &= !(SUBT_FLAG | CARRY_FLAG);
        if set_half_carry {
            self.f |= HALF_CARRY_FLAG;
        } else {
            self.f &= !HALF_CARRY_FLAG;
        }

        self.set_flag_conditional(ZERO_FLAG, result == 0);
    }

    fn add_to_u16(&mut self, a: u8, b: u16) -> u16 {
        self.add(a, b as u8);
        a as u16 + b
    }

    // increment/decrement
    fn increment_16(high: &mut u8, low: &mut u8) {
        // does not affect flags
        if *low == 0xFF {
            *high += 1;
        }
        *low += 1;
    }

    fn decrement_16(high: &mut u8, low: &mut u8) {
        // does not affect flags
        if *low == 0x00 {
            *high -= 1;
        }
        *low -= 1;
    }
}
