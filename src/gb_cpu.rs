use std::rc::Rc;
use gb_mem::MemoryController;
use gb_opcodes::OpCodes;

const ZERO_FLAG: u8 = 1 << 7;
const SUBT_FLAG: u8 = 1 << 6;
const HALF_CARRY_FLAG: u8 = 1 << 5;
const CARRY_FLAG: u8 = 1 << 4;

const VBLANK_IF: u8 = 1;
const LCDC_IF: u8 = 1 << 1;
const TIMER_OVERFLOW_IF: u8 = 1 << 2;
const SERIAL_IO_COMPLETE_IF: u8 = 1 << 3;
const P10_P13_TERM_NEG_EDGE_IF: u8 = 1 << 4;

mod details {
    const PC_MAX: usize = 0xFFFF;

    #[derive(Debug)]
    pub struct ProgramCounter {
        val: usize,
    }

    impl ProgramCounter {
        pub fn new(init: usize) -> Self {
            ProgramCounter { val: init }
        }

        pub fn get(&self) -> usize {
            self.val
        }

        pub fn set(&mut self, val: usize) {
            self.val = val & PC_MAX;
        }

        pub fn inc(&mut self, amt: usize) {
            self.val = self.val.wrapping_add(amt) & PC_MAX;
        }

        pub fn dec(&mut self, val: usize) {
            self.val = self.val.wrapping_sub(val) & PC_MAX;
        }
    }
}

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
    pc: details::ProgramCounter,

    ime: bool, // interrupt master enabled
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
            sp: 0xFFFEu16,
            pc: details::ProgramCounter::new(0x0100usize),
            ime: true,
            halt: false,
            stop: false,
            mc: Rc::new(MemoryController::new()),
        }
    }

    pub fn get_memory_controller(&self) -> Rc<MemoryController> {
        self.mc.clone()
    }

    fn read_op(&mut self) -> u8 {
        // TODO: cache the operation in the CPU to determine what happens next
        let result = self.mc.read(self.pc.get());
        self.pc.inc(1);
        result
    }

    fn get_carry_state(&self) -> u8 {
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
        let pc = self.pc.get();
        let high_addr = pc;
        let low_addr = pc + 1;
        self.pc.inc(2);
        (mc.read(high_addr) as u16) | (mc.read(low_addr) as u16) << 8
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
        let r = a.overflowing_add(b);
        let hr = (a & 0x0F) + (b & 0x0F);

        self.set_flag_conditional(HALF_CARRY_FLAG, hr > 0x0F);
        self.set_flag_conditional(CARRY_FLAG, r.1); // it wrapped around
        self.reset_flag(SUBT_FLAG);
        r.0
    }

    fn add_with_carry(&mut self, a: u8, b: u8) -> u8 {
        let carry = self.get_carry_state();
        let t = a + carry;
        self.add(t, b)
    }

    fn subtract(&mut self, a: u8, b: u8) -> u8 {
        let r = self.subtract_no_zcheck(a, b);
        self.set_flag_conditional(ZERO_FLAG, r == 0);
        r
    }

    fn subtract_no_zcheck(&mut self, a: u8, b: u8) -> u8 {
        let r = a.overflowing_sub(b);
        let hr = (a & 0x0F) - (b & 0x0F);

        self.set_flag_conditional(HALF_CARRY_FLAG, hr > 0x0F);
        self.set_flag_conditional(CARRY_FLAG, r.1); // it wrapped around
        self.set_flag(SUBT_FLAG);
        r.0
    }

    fn subtract_with_carry(&mut self, a: u8, b: u8) -> u8 {
        let carry = self.get_carry_state();
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
    fn increment(&mut self, byte: &mut u8) {
        self.set_flag_conditional(HALF_CARRY_FLAG, (*byte & 0x0F) == 0x0F);
        self.reset_flag(SUBT_FLAG);
        *byte += 1;
        self.set_flag_conditional(ZERO_FLAG, *byte == 0);
    }

    fn decrement(&mut self, byte: &mut u8) {
        self.set_flag(SUBT_FLAG);
        self.set_flag_conditional(HALF_CARRY_FLAG, (*byte & 0x0F) == 0x00);
        *byte -= 1;
        self.set_flag_conditional(ZERO_FLAG, *byte == 0);
    }

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

    // rotation

    // rotate left through self, but still copies leftmost bit to carry
    fn do_rlc(&mut self, value: u8) -> u8 {
        let result = value.rotate_left(1);

        self.set_flag_conditional(CARRY_FLAG, (value & 0b1000_0000) > 0);
        self.set_flag_conditional(ZERO_FLAG, result == 0);
        self.reset_flag(SUBT_FLAG | HALF_CARRY_FLAG);

        result
    }

    // rotate left through carry (and carry into rightmost bit)
    fn do_rl(&mut self, value: u8) -> u8 {
        let carry = self.get_carry_state();
        let result = (value << 1) | carry;

        self.set_flag_conditional(CARRY_FLAG, (value & 0b1000_0000) > 0);
        self.set_flag_conditional(ZERO_FLAG, result == 0);
        self.reset_flag(SUBT_FLAG | HALF_CARRY_FLAG);

        result
    }

    // rotate right through self, but copy rightmost bit to carry
    fn do_rrc(&mut self, value: u8) -> u8 {
        let result = value.rotate_right(1);

        self.set_flag_conditional(CARRY_FLAG, (value & 0b1) > 0);
        self.set_flag_conditional(ZERO_FLAG, result == 0);
        self.reset_flag(SUBT_FLAG | HALF_CARRY_FLAG);

        result
    }

    // rotate right through the carry
    fn do_rr(&mut self, value: u8) -> u8 {
        let carry = self.get_carry_state();
        let result = (value >> 1) | (carry << 7);

        self.set_flag_conditional(CARRY_FLAG, (value & 0b1) == 0b1);
        self.set_flag_conditional(ZERO_FLAG, result == 0);
        self.reset_flag(SUBT_FLAG | HALF_CARRY_FLAG);

        result
    }

    // shift operations
    fn do_sla(&mut self, value: u8) -> u8 {
        let result = value << 1;

        self.set_flag_conditional(CARRY_FLAG, (value & 0b1000_0000) > 0);
        self.set_flag_conditional(ZERO_FLAG, result == 0);
        self.reset_flag(SUBT_FLAG | HALF_CARRY_FLAG);

        result
    }

    fn do_sra(&mut self, value: u8) -> u8 {
        let msb = value & 0b1000_0000;
        let result = value >> 1 | msb;

        self.set_flag_conditional(CARRY_FLAG, (value & 0b1) > 0);
        self.set_flag_conditional(ZERO_FLAG, result == 0);
        self.reset_flag(SUBT_FLAG | HALF_CARRY_FLAG);

        result
    }

    fn do_srl(&mut self, value: u8) -> u8 {
        let result = value >> 1;

        self.set_flag_conditional(CARRY_FLAG, (value & 0b1) > 0);
        self.set_flag_conditional(ZERO_FLAG, result == 0);
        self.reset_flag(SUBT_FLAG | HALF_CARRY_FLAG);

        result
    }

    fn do_swap(&mut self, value: u8) -> u8 {
        let result = (value & 0x0F) << 4 | (value & 0xF0) >> 4;

        self.set_flag_conditional(ZERO_FLAG, result == 0);
        self.reset_flag(SUBT_FLAG | HALF_CARRY_FLAG | CARRY_FLAG);

        result
    }

    // program flow
    fn do_jump_conditional(&mut self, test: bool) {
        let pc = self.pc.get();
        let low = self.mc.read(pc) as u16;
        let high = self.mc.read(pc + 1) as u16;
        self.pc.inc(2);
        if test {
            let dest = high << 8 | low;
            self.pc.set(dest as usize);
        }
    }

    fn do_jump_relative_conditional(&mut self, test: bool) {
        let offset = self.mc.read(self.pc.get());
        self.pc.inc(1);

        if test {
            self.pc.inc(offset as i8 as usize);
        }
    }
}
