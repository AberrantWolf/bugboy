use std::rc::Rc;
use std::cell::RefCell;

use gb_hw_bus::HardwareBus;
use gb_mem::{MemoryController, RamAddress};
use gb_opcodes::{OpCodes, SecondOpAction, SecondOpRegister, SecondOpType};

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
pub struct DmgCpu {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    sp: RamAddress,
    pc: RamAddress,

    ime: bool, // interrupt master enabled
    halt: bool,
    stop: bool,

    clock: u64,

    mc: Rc<RefCell<MemoryController>>,
    bus: Rc<RefCell<HardwareBus>>,
}

impl DmgCpu {
    pub fn new(bus: Rc<RefCell<HardwareBus>>, mc: Rc<RefCell<MemoryController>>) -> Self {
        DmgCpu {
            a: 0u8,
            b: 0u8,
            c: 0u8,
            d: 0u8,
            e: 0u8,
            f: 0u8,
            h: 0u8,
            l: 0u8,
            sp: RamAddress::new(0xFFFEu16),
            pc: RamAddress::new(0x0100u16),

            ime: true,
            halt: false,
            stop: false,

            clock: 0u64,

            mc: mc,
            bus: bus,
        }
    }

    pub fn tick_op(&mut self) {}

    pub fn get_memory_controller(&self) -> Rc<RefCell<MemoryController>> {
        self.mc.clone()
    }

    fn read_op(&mut self) -> u8 {
        // TODO: cache the operation in the CPU to determine what happens next
        let result = self.mc.borrow().read(self.pc.post_inc(1));
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
    fn make_bc_address(&self) -> RamAddress {
        RamAddress::new((self.b as u16) << 8 | self.c as u16)
    }

    fn make_de_address(&self) -> RamAddress {
        RamAddress::new((self.d as u16) << 8 | self.e as u16)
    }

    fn make_hl_address(&self) -> RamAddress {
        RamAddress::new((self.h as u16) << 8 | self.l as u16)
    }

    fn make_ffc_address(&self, n: u8) -> RamAddress {
        RamAddress::new(0xFF00 | n as u16)
    }

    fn make_ffn_address(&mut self) -> RamAddress {
        let mc = &self.mc.borrow();
        let low = mc.read(self.pc.post_inc(1));
        let high = mc.read(self.pc.post_inc(1));
        RamAddress::new((low as u16) | (high as u16) << 8)
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
    fn read_pc_as_address(&mut self) -> u16 {
        let mc = self.mc.borrow();
        let low = mc.read(self.pc.post_inc(1)) as u16;
        let high = mc.read(self.pc.post_inc(1)) as u16;
        high << 8 | low
    }

    fn do_jump_conditional(&mut self, test: bool) {
        let dest = self.read_pc_as_address();
        if test {
            self.pc.set(dest);
        }
    }

    fn do_jump_relative_conditional(&mut self, test: bool) {
        let offset = self.mc.borrow().read(self.pc.post_inc(1));

        if test {
            self.pc.inc(offset as i8 as u16);
        }
    }

    fn push_address_parts(&mut self, high: u8, low: u8) {
        let mut mc = self.mc.borrow_mut();
        mc.write(self.sp.dec(1), low);
        mc.write(self.sp.dec(1), high);
    }

    fn push_address_u16(&mut self, addr: u16) {
        let high = (addr & 0xFF00 >> 8) as u8;
        let low = (addr & 0x00FF) as u8;
        self.push_address_parts(high, low);
    }

    fn pop_address_parts(&mut self) -> (u8, u8) {
        let mc = self.mc.borrow();
        let high = mc.read(self.sp.post_inc(1));
        let low = mc.read(self.sp.post_inc(1));
        (high, low)
    }

    fn pop_address_u16(&mut self) -> u16 {
        let parts = self.pop_address_parts();
        (parts.0 as u16) << 8 | (parts.1 as u16)
    }

    fn do_call_conditional(&mut self, test: bool) {
        let dest = self.read_pc_as_address();

        if test {
            let addr = self.pc.get();
            self.push_address_u16(addr);
            self.pc.set(dest);
        }
    }

    fn do_return_conditional(&mut self, test: bool) {
        if test {
            let addr = self.pop_address_u16();
            self.pc.set(addr);
        }
    }

    // multibyte ops
    fn hand_rotate_shift_op(&mut self, value: u8, op: SecondOpAction) -> u8 {
        match op {
            SecondOpAction::RLC => return self.do_rlc(value),
            SecondOpAction::RL => return self.do_rl(value),
            SecondOpAction::RRC => return self.do_rrc(value),
            SecondOpAction::RR => return self.do_rr(value),
            SecondOpAction::SLA => return self.do_sla(value),
            SecondOpAction::SRA => return self.do_sra(value),
            SecondOpAction::SRL => return self.do_srl(value),
            SecondOpAction::SWAP => return self.do_swap(value),
        }
    }

    fn decode_and_execute_cb_op(&mut self, sop: u8) {
        let op_type: SecondOpType = SecondOpType::from_u8(sop);
        let action = SecondOpAction::from_u8(sop);
        let register = SecondOpRegister::from_u8(sop);

        let bit_mask = 1 << (action as u8);

        match op_type {
            SecondOpType::BIT_CHECK => {
                let reg_value = match register {
                    SecondOpRegister::A => self.a,
                    SecondOpRegister::B => self.b,
                    SecondOpRegister::C => self.c,
                    SecondOpRegister::D => self.d,
                    SecondOpRegister::E => self.e,
                    SecondOpRegister::H => self.h,
                    SecondOpRegister::L => self.l,
                    SecondOpRegister::mHL => self.mc.borrow().read(self.make_hl_address()),
                };
                self.set_flag_conditional(ZERO_FLAG, (reg_value & bit_mask) == 0);
            }
            SecondOpType::SET => match register {
                SecondOpRegister::A => self.a |= bit_mask,
                SecondOpRegister::B => self.b |= bit_mask,
                SecondOpRegister::C => self.c |= bit_mask,
                SecondOpRegister::D => self.d |= bit_mask,
                SecondOpRegister::E => self.e |= bit_mask,
                SecondOpRegister::H => self.h |= bit_mask,
                SecondOpRegister::L => self.l |= bit_mask,
                SecondOpRegister::mHL => {
                    let mut mc = self.mc.borrow_mut();
                    let hl = self.make_hl_address();
                    let val = mc.read(hl);
                    mc.write(hl, val | bit_mask);
                }
            },
            SecondOpType::RESET => match register {
                SecondOpRegister::A => self.a &= !bit_mask,
                SecondOpRegister::B => self.b &= !bit_mask,
                SecondOpRegister::C => self.c &= !bit_mask,
                SecondOpRegister::D => self.d &= !bit_mask,
                SecondOpRegister::E => self.e &= !bit_mask,
                SecondOpRegister::H => self.h &= !bit_mask,
                SecondOpRegister::L => self.l &= !bit_mask,
                SecondOpRegister::mHL => {
                    let mut mc = self.mc.borrow_mut();
                    let hl = self.make_hl_address();
                    let val = mc.read(hl);
                    mc.write(hl, val & !bit_mask);
                }
            },
            SecondOpType::ROTATE_SHIFT => match register {
                SecondOpRegister::A => {
                    let a = self.a;
                    self.a = self.hand_rotate_shift_op(a, action);
                }
                SecondOpRegister::B => self.b &= !bit_mask,
                SecondOpRegister::C => self.c &= !bit_mask,
                SecondOpRegister::D => self.d &= !bit_mask,
                SecondOpRegister::E => self.e &= !bit_mask,
                SecondOpRegister::H => self.h &= !bit_mask,
                SecondOpRegister::L => self.l &= !bit_mask,
                SecondOpRegister::mHL => {
                    let mut mc = self.mc.borrow_mut();
                    let hl = self.make_hl_address();
                    let val = mc.read(hl);
                    mc.write(hl, val & !bit_mask);
                }
            },
        }
    }

    fn is_flag_set(&self, flag: u8) -> bool {
        (self.f & flag) == flag
    }

    fn do_daa(&mut self) {
        let n = self.is_flag_set(SUBT_FLAG);
        let hc = self.is_flag_set(HALF_CARRY_FLAG);
        let c = self.is_flag_set(CARRY_FLAG);

        let mut temp = self.a as u16;

        if n {
            if hc {
                temp = (temp.wrapping_sub(0x06)) & 0xFF;
            }
            if c {
                temp = temp - 0x06;
            }
        } else {
            if hc || (temp & 0x0f) > 9 {
                temp += 0x06;
            }
            if c || temp > 0x9f {
                temp += 0x60;
            }
        }

        let a = temp as u8;
        self.a = a;
        self.set_flag_conditional(ZERO_FLAG, a == 0);
        self.set_flag_conditional(CARRY_FLAG, temp > 0xFF);
        self.reset_flag(HALF_CARRY_FLAG);
    }

    pub fn tick(&mut self) {
        if self.stop {
            return;
        }

        self.bus.borrow_mut().sync(self.clock);
    }
}
